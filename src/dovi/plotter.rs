use std::fmt::Write;
use std::ops::Range;
use std::path::PathBuf;

use anyhow::{Result, bail};
use dolby_vision::rpu::dovi_rpu::DoviRpu;
use dolby_vision::rpu::extension_metadata::blocks::{
    ExtMetadataBlockLevel2, ExtMetadataBlockLevel8,
};
use dolby_vision::rpu::utils::parse_rpu_file;
use dolby_vision::utils::{nits_to_pq, pq_to_nits};
use plotters::coord::ranged1d::{KeyPointHint, NoDefaultFormatting, Ranged, ValueFormatter};
use plotters::coord::types::RangedCoordusize;
use plotters::prelude::{
    AreaSeries, BitMapBackend, Cartesian2d, ChartBuilder, ChartContext, IntoDrawingArea,
    LineSeries, PathElement, SeriesLabelPosition, WHITE,
};
use plotters::style::{BLACK, Color, IntoTextStyle, RGBColor, ShapeStyle};

use super::input_from_either;
use super::rpu_info::{AggregateStats, RpusListSummary, SummaryTrimsStats};
use crate::commands::PlotArgs;

#[cfg(not(feature = "system-font"))]
const NOTO_SANS_REGULAR: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/NotoSans-Regular.ttf"
));

const MAX_COLOR: RGBColor = RGBColor(65, 105, 225);
const AVERAGE_COLOR: RGBColor = RGBColor(75, 0, 130);
const COLORS: [RGBColor; 8] = [
    RGBColor(220, 38, 38),  // red
    RGBColor(234, 179, 8),  // yellow
    RGBColor(34, 197, 94),  // green
    RGBColor(34, 211, 238), // cyan
    RGBColor(59, 130, 246), // blue
    RGBColor(236, 72, 153), // magenta
    RGBColor(249, 115, 22), // orange
    RGBColor(139, 92, 246), // purple
];

pub struct Plotter {
    input: PathBuf,
}

pub struct PlotCoord {
    key_points: Vec<f64>,
    range: Range<f64>,
    mapper: fn(&f64, (i32, i32)) -> i32,
    formatter: fn(&f64) -> String,
}

#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlotType {
    /// L1 Dynamic Brightness
    L1,
    /// L2 Trims
    L2,
    /// L8 Trims (CM v4.0 RPU required)
    L8,
    /// L8 Saturation Vectors (CM v4.0 RPU required)
    L8Saturation,
    /// L8 Hue Vectors (CM v4.0 RPU required)
    L8Hue,
}

#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrimParameter {
    Slope,
    Offset,
    Power,
    Chroma,
    Saturation,
    MS,
    Mid,
    Clip,
}

struct Series<'a, T> {
    identifier: &'static str,
    stats: &'a AggregateStats,
    mapper: fn(&T) -> f64,
}

impl Plotter {
    pub fn plot(args: PlotArgs) -> Result<()> {
        #[cfg(not(feature = "system-font"))]
        {
            let res = plotters::style::register_font(
                "sans-serif",
                plotters::style::FontStyle::Normal,
                NOTO_SANS_REGULAR,
            );

            if res.is_err() {
                bail!("Failed registering font!");
            }
        }

        let PlotArgs {
            input,
            input_pos,
            output,
            title,
            start: start_arg,
            end: end_arg,
            plot_type,
            target_nits_str,
            trims: trim_params,
        } = args;

        let target_nits = target_nits_str.parse::<u16>()?;

        let output = output.unwrap_or(PathBuf::from(plot_type.default_output(target_nits)));
        let title = title.unwrap_or(plot_type.default_title(target_nits).to_string());

        let input = input_from_either("info", input, input_pos)?;
        let plotter = Plotter { input };

        println!("Parsing RPU file...");
        let orig_rpus = parse_rpu_file(plotter.input)?;

        // inclusive range, end must be last RPU index
        let start = start_arg.unwrap_or(0);
        let end = end_arg.unwrap_or(orig_rpus.len() - 1);
        let rpus = &orig_rpus[start..=end];

        println!("Plotting...");
        let summary = plot_type.summary(rpus, target_nits)?;

        if plot_type.requires_dmv2() && !summary.dmv2 {
            bail!(
                "Cannot generate {}: CM v4.0 RPU is required",
                plot_type.name()
            );
        }

        let x_spec = 0..rpus.len();

        let root = BitMapBackend::new(&output, (3000, 1200)).into_drawing_area();
        root.fill(&WHITE)?;
        let root = root
            .margin(30, 30, 60, 60)
            .titled(&title, ("sans-serif", 40))?;

        let mut chart = ChartBuilder::on(&root)
            .x_label_area_size(60)
            .y_label_area_size(60)
            .margin_top(90)
            .build_cartesian_2d(x_spec, PlotCoord::from(plot_type))?;

        chart
            .configure_mesh()
            .bold_line_style(BLACK.mix(0.10))
            .light_line_style(BLACK.mix(0.01))
            .label_style(("sans-serif", 22))
            .axis_desc_style(("sans-serif", 24))
            .x_desc("frames")
            .x_max_light_lines(1)
            .x_labels(24)
            .y_desc(plot_type.y_desc())
            .draw()?;

        plot_type.draw_series(&mut chart, &summary, trim_params)?;

        chart
            .configure_series_labels()
            .border_style(BLACK)
            .position(SeriesLabelPosition::LowerLeft)
            .label_font(("sans-serif", 24))
            .background_style(WHITE)
            .draw()?;

        let mut chart_caption = String::new();
        let l6_meta_str = if let Some(l6) = summary.l6_meta.as_ref() {
            if l6.len() > 2 {
                let l6_list = l6[..2].join(". ");

                format!("{l6_list}. Total different: {}", l6.len())
            } else {
                l6.join(". ")
            }
        } else {
            String::from("None")
        };

        write!(
            chart_caption,
            "Frames: {}. {}. Scenes: {}. DM version: {}.",
            summary.count, summary.profiles_str, summary.scene_count, summary.dm_version_str,
        )?;

        if let Some((dmv1_count, dmv2_count)) = summary.dm_version_counts {
            write!(
                chart_caption,
                " v2.9 count: {dmv1_count}, v4.0 count: {dmv2_count}"
            )?;
        }

        let caption_md = if let Some(l9_mdp) = &summary.l9_mdp {
            format!("{} - {}", summary.rpu_mastering_meta_str, l9_mdp.join(", "))
        } else {
            summary.rpu_mastering_meta_str
        };

        let caption_style = ("sans-serif", 24).into_text_style(&root);
        root.draw_text(&chart_caption, &caption_style, (60, 10))?;
        root.draw_text(&caption_md, &caption_style, (60, 35))?;
        root.draw_text(
            &format!("L6 metadata: {l6_meta_str}"),
            &caption_style,
            (60, 60),
        )?;

        let mut right_captions = vec![format!("L5 offsets: {}", summary.l5_str)];
        if !summary.l2_trims.is_empty() {
            right_captions.push(format!("L2 trims: {}", summary.l2_trims.join(", ")));
        }
        if let Some(l8_trims) = summary.l8_trims.filter(|v| !v.is_empty()) {
            right_captions.push(format!("L8 trims: {}", l8_trims.join(", ")));
        }

        let pos_x = right_captions
            .iter()
            .filter_map(|c| root.estimate_text_size(c, &caption_style).ok())
            .map(|(size, _)| size)
            .max()
            .map_or(0, |max_size| (root.dim_in_pixel().0 - max_size) as i32);
        let mut pos_y = 60;

        for caption in right_captions.iter().rev() {
            root.draw_text(caption, &caption_style, (pos_x, pos_y))?;
            pos_y -= 25;
        }

        root.present()?;

        println!("Done.");

        Ok(())
    }

    fn draw_l1_series(
        chart: &mut ChartContext<BitMapBackend, Cartesian2d<RangedCoordusize, PlotCoord>>,
        summary: &RpusListSummary,
    ) -> Result<()> {
        let data = &summary.l1_data;
        let l1_stats = &summary.l1_stats;

        let max_series_label = format!(
            "Maximum (MaxCLL: {:.2} nits, avg: {:.2} nits)",
            l1_stats.maxcll, l1_stats.maxcll_avg,
        );
        let avg_series_label = format!(
            "Average (MaxFALL: {:.2} nits, avg: {:.2} nits)",
            l1_stats.maxfall, l1_stats.maxfall_avg,
        );

        let max_series = AreaSeries::new(
            (0..).zip(data.iter()).map(|(x, y)| (x, y.max)),
            0.0,
            MAX_COLOR.mix(0.25),
        )
        .border_style(MAX_COLOR);
        let avg_series = AreaSeries::new(
            (0..).zip(data.iter()).map(|(x, y)| (x, y.avg)),
            0.0,
            AVERAGE_COLOR.mix(0.50),
        )
        .border_style(AVERAGE_COLOR);
        let min_series = AreaSeries::new(
            (0..).zip(data.iter()).map(|(x, y)| (x, y.min)),
            0.0,
            BLACK.mix(0.50),
        )
        .border_style(BLACK);

        chart
            .draw_series(max_series)?
            .label(max_series_label)
            .legend(|(x, y)| {
                PathElement::new(
                    vec![(x, y), (x + 20, y)],
                    ShapeStyle {
                        color: MAX_COLOR.to_rgba(),
                        filled: false,
                        stroke_width: 2,
                    },
                )
            });
        chart
            .draw_series(avg_series)?
            .label(avg_series_label)
            .legend(|(x, y)| {
                PathElement::new(
                    vec![(x, y), (x + 20, y)],
                    ShapeStyle {
                        color: AVERAGE_COLOR.to_rgba(),
                        filled: false,
                        stroke_width: 2,
                    },
                )
            });
        chart
            .draw_series(min_series)?
            .label(format!("Minimum (max: {:.06} nits)", l1_stats.max_min_nits,))
            .legend(|(x, y)| {
                PathElement::new(
                    vec![(x, y), (x + 20, y)],
                    ShapeStyle {
                        color: BLACK.to_rgba(),
                        filled: false,
                        stroke_width: 2,
                    },
                )
            });

        Ok(())
    }

    fn draw_l2_series(
        chart: &mut ChartContext<BitMapBackend, Cartesian2d<RangedCoordusize, PlotCoord>>,
        summary: &RpusListSummary,
        mut trim_params: Option<Vec<TrimParameter>>,
    ) -> Result<()> {
        let data = summary.l2_data.as_ref().unwrap();
        let stats = summary.l2_stats.as_ref().unwrap();

        let default_params = TrimParameter::default_l2_params();

        // Remove invalid params
        if let Some(trims) = trim_params.as_mut() {
            trims.retain(|p| default_params.contains(p));
        }

        let effective_params = if trim_params.as_ref().is_some_and(|v| !v.is_empty()) {
            trim_params.as_deref().unwrap()
        } else {
            default_params
        };

        let series = effective_params
            .iter()
            .map(|param| param.l2_series_config(stats))
            .collect::<Vec<_>>();

        Self::draw_line_series(chart, data, &series)
    }

    fn draw_l8_trims_series(
        chart: &mut ChartContext<BitMapBackend, Cartesian2d<RangedCoordusize, PlotCoord>>,
        summary: &RpusListSummary,
        trim_params: Option<Vec<TrimParameter>>,
    ) -> Result<()> {
        let data = summary.l8_data.as_ref().unwrap();
        let stats = summary.l8_stats_trims.as_ref().unwrap();

        let default_params = TrimParameter::default_l8_params();
        let effective_params = if trim_params.as_ref().is_some_and(|v| !v.is_empty()) {
            trim_params.as_deref().unwrap()
        } else {
            default_params
        };

        let series = effective_params
            .iter()
            .map(|param| param.l8_series_config(stats))
            .collect::<Vec<_>>();

        Self::draw_line_series(chart, data, &series)
    }

    fn draw_l8_saturation_series(
        chart: &mut ChartContext<BitMapBackend, Cartesian2d<RangedCoordusize, PlotCoord>>,
        summary: &RpusListSummary,
    ) -> Result<()> {
        let data = summary.l8_data.as_ref().unwrap();
        let stats = summary.l8_stats_saturation.as_ref().unwrap();

        let series: [Series<ExtMetadataBlockLevel8>; 6] = [
            Series {
                identifier: "red",
                stats: &stats.red,
                mapper: |e| e.saturation_vector_field0 as f64,
            },
            Series {
                identifier: "yellow",
                stats: &stats.yellow,
                mapper: |e| e.saturation_vector_field1 as f64,
            },
            Series {
                identifier: "green",
                stats: &stats.green,
                mapper: |e| e.saturation_vector_field2 as f64,
            },
            Series {
                identifier: "cyan",
                stats: &stats.cyan,
                mapper: |e| e.saturation_vector_field3 as f64,
            },
            Series {
                identifier: "blue",
                stats: &stats.blue,
                mapper: |e| e.saturation_vector_field4 as f64,
            },
            Series {
                identifier: "magenta",
                stats: &stats.magenta,
                mapper: |e| e.saturation_vector_field5 as f64,
            },
        ];

        Self::draw_line_series(chart, data, &series)
    }

    fn draw_l8_hue_series(
        chart: &mut ChartContext<BitMapBackend, Cartesian2d<RangedCoordusize, PlotCoord>>,
        summary: &RpusListSummary,
    ) -> Result<()> {
        let data = summary.l8_data.as_ref().unwrap();
        let stats = summary.l8_stats_hue.as_ref().unwrap();

        let series: [Series<ExtMetadataBlockLevel8>; 6] = [
            Series {
                identifier: "red",
                stats: &stats.red,
                mapper: |e| e.hue_vector_field0 as f64,
            },
            Series {
                identifier: "yellow",
                stats: &stats.yellow,
                mapper: |e| e.hue_vector_field1 as f64,
            },
            Series {
                identifier: "green",
                stats: &stats.green,
                mapper: |e| e.hue_vector_field2 as f64,
            },
            Series {
                identifier: "cyan",
                stats: &stats.cyan,
                mapper: |e| e.hue_vector_field3 as f64,
            },
            Series {
                identifier: "blue",
                stats: &stats.blue,
                mapper: |e| e.hue_vector_field4 as f64,
            },
            Series {
                identifier: "magenta",
                stats: &stats.magenta,
                mapper: |e| e.hue_vector_field5 as f64,
            },
        ];

        Self::draw_line_series(chart, data, &series)
    }

    fn draw_line_series<T>(
        chart: &mut ChartContext<BitMapBackend, Cartesian2d<RangedCoordusize, PlotCoord>>,
        data: &[T],
        series: &[Series<T>],
    ) -> Result<()> {
        for (series, color) in series.iter().zip(COLORS.iter()) {
            let Series {
                identifier,
                stats,
                mapper,
            } = series;

            let label = format!(
                "{identifier} (min: {:.0}, max: {:.0}, avg: {:.0})",
                stats.min, stats.max, stats.avg
            );
            let line_series =
                LineSeries::new((0..).zip(data.iter()).map(|(x, y)| (x, mapper(y))), color);

            chart
                .draw_series(line_series)?
                .label(label)
                .legend(move |(x, y)| {
                    PathElement::new(
                        vec![(x, y), (x + 20, y)],
                        ShapeStyle {
                            color: color.to_rgba(),
                            filled: false,
                            stroke_width: 2,
                        },
                    )
                });
        }

        Ok(())
    }
}

impl PlotType {
    pub fn name(&self) -> &str {
        match self {
            Self::L1 => "L1 Dynamic Brightness",
            Self::L2 => "L2 Trims",
            Self::L8 => "L8 Trims",
            Self::L8Saturation => "L8 Saturation Vectors",
            Self::L8Hue => "L8 Hue Vectors",
        }
    }

    pub fn default_title(&self, target_nits: u16) -> String {
        match self {
            Self::L1 => format!("Dolby Vision {}", self.name()),
            _ => format!("Dolby Vision {} ({} nits)", self.name(), target_nits),
        }
    }

    pub fn default_output(&self, target_nits: u16) -> String {
        match self {
            Self::L1 => "L1_plot.png".to_string(),
            Self::L2 => format!("L2_plot-{}.png", target_nits),
            Self::L8 => format!("L8-trims_plot-{}.png", target_nits),
            Self::L8Saturation => format!("L8-saturation_plot-{}.png", target_nits),
            Self::L8Hue => format!("L8-hue_plot-{}.png", target_nits),
        }
    }

    pub fn y_desc(&self) -> &str {
        match self {
            Self::L1 => "nits (cd/m²)",
            _ => "",
        }
    }

    pub fn requires_dmv2(&self) -> bool {
        !matches!(self, Self::L1 | Self::L2)
    }

    pub fn summary(&self, rpus: &[DoviRpu], target_nits: u16) -> Result<RpusListSummary> {
        match self {
            Self::L1 => RpusListSummary::new(rpus),
            Self::L2 => RpusListSummary::with_l2_data(rpus, target_nits),
            Self::L8 => RpusListSummary::with_l8_trims_data(rpus, target_nits),
            Self::L8Saturation => RpusListSummary::with_l8_saturation_data(rpus, target_nits),
            Self::L8Hue => RpusListSummary::with_l8_hue_data(rpus, target_nits),
        }
    }

    pub fn draw_series(
        &self,
        chart: &mut ChartContext<BitMapBackend, Cartesian2d<RangedCoordusize, PlotCoord>>,
        summary: &RpusListSummary,
        trim_params: Option<Vec<TrimParameter>>,
    ) -> Result<()> {
        match self {
            Self::L1 => Plotter::draw_l1_series(chart, summary),
            Self::L2 => Plotter::draw_l2_series(chart, summary, trim_params),
            Self::L8 => Plotter::draw_l8_trims_series(chart, summary, trim_params),
            Self::L8Saturation => Plotter::draw_l8_saturation_series(chart, summary),
            Self::L8Hue => Plotter::draw_l8_hue_series(chart, summary),
        }
    }
}

impl TrimParameter {
    pub const fn identifier(&self) -> &'static str {
        match self {
            Self::Slope => "slope (gain)",
            Self::Offset => "offset (lift)",
            Self::Power => "power (gamma)",
            Self::Chroma => "chroma (weight)",
            Self::Saturation => "saturation (gain)",
            Self::MS => "ms (weight)",
            Self::Mid => "mid (contrast)",
            Self::Clip => "clip (trim)",
        }
    }

    pub const fn param_stats<'a>(&self, stats: &'a SummaryTrimsStats) -> &'a AggregateStats {
        match self {
            Self::Slope => &stats.slope,
            Self::Offset => &stats.offset,
            Self::Power => &stats.power,
            Self::Chroma => &stats.chroma,
            Self::Saturation => &stats.saturation,
            Self::MS => &stats.ms_weight,
            Self::Mid => stats.target_mid_contrast.as_ref().unwrap(),
            Self::Clip => stats.clip_trim.as_ref().unwrap(),
        }
    }

    pub const fn default_l2_params() -> &'static [Self] {
        &[
            Self::Slope,
            Self::Offset,
            Self::Power,
            Self::Chroma,
            Self::Saturation,
            Self::MS,
        ]
    }

    pub const fn default_l8_params() -> &'static [Self] {
        &[
            Self::Slope,
            Self::Offset,
            Self::Power,
            Self::Chroma,
            Self::Saturation,
            Self::MS,
            Self::Mid,
            Self::Clip,
        ]
    }

    const fn l2_series_config<'a>(
        &self,
        stats: &'a SummaryTrimsStats,
    ) -> Series<'a, ExtMetadataBlockLevel2> {
        let mapper: fn(&ExtMetadataBlockLevel2) -> f64 = match self {
            Self::Slope => |e| e.trim_slope as f64,
            Self::Offset => |e| e.trim_offset as f64,
            Self::Power => |e| e.trim_power as f64,
            Self::Chroma => |e| e.trim_chroma_weight as f64,
            Self::Saturation => |e| e.trim_saturation_gain as f64,
            Self::MS => |e| e.ms_weight as f64,
            _ => unreachable!(),
        };

        Series {
            identifier: self.identifier(),
            stats: self.param_stats(stats),
            mapper,
        }
    }

    const fn l8_series_config<'a>(
        &self,
        stats: &'a SummaryTrimsStats,
    ) -> Series<'a, ExtMetadataBlockLevel8> {
        let mapper: fn(&ExtMetadataBlockLevel8) -> f64 = match self {
            Self::Slope => |e| e.trim_slope as f64,
            Self::Offset => |e| e.trim_offset as f64,
            Self::Power => |e| e.trim_power as f64,
            Self::Chroma => |e| e.trim_chroma_weight as f64,
            Self::Saturation => |e| e.trim_saturation_gain as f64,
            Self::MS => |e| e.ms_weight as f64,
            Self::Mid => |e| e.target_mid_contrast as f64,
            Self::Clip => |e| e.clip_trim as f64,
        };

        Series {
            identifier: self.identifier(),
            stats: self.param_stats(stats),
            mapper,
        }
    }
}

impl From<PlotType> for PlotCoord {
    fn from(plot_type: PlotType) -> Self {
        match plot_type {
            PlotType::L1 => Self {
                key_points: vec![
                    nits_to_pq(0.01),
                    nits_to_pq(0.1),
                    nits_to_pq(0.5),
                    nits_to_pq(1.0),
                    nits_to_pq(2.5),
                    nits_to_pq(5.0),
                    nits_to_pq(10.0),
                    nits_to_pq(25.0),
                    nits_to_pq(50.0),
                    nits_to_pq(100.0),
                    nits_to_pq(200.0),
                    nits_to_pq(400.0),
                    nits_to_pq(600.0),
                    nits_to_pq(1000.0),
                    nits_to_pq(2000.0),
                    nits_to_pq(4000.0),
                    nits_to_pq(10000.0),
                ],
                range: 0_f64..1.0_f64,
                mapper: |value, limit| {
                    let size = limit.1 - limit.0;
                    (*value * size as f64) as i32 + limit.0
                },
                formatter: |value| {
                    let nits = (pq_to_nits(*value) * 1000.0).round() / 1000.0;
                    format!("{nits}")
                },
            },
            PlotType::L2 | PlotType::L8 => Self {
                key_points: vec![
                    0.0, 512.0, 1024.0, 1536.0, 2048.0, 2560.0, 3072.0, 3584.0, 4096.0,
                ],
                range: 0_f64..4096.0_f64,
                mapper: |value, limit| {
                    let norm = value / 4096.0;
                    let size = limit.1 - limit.0;
                    (norm * size as f64).round() as i32 + limit.0
                },
                formatter: |value| format!("{value}"),
            },
            PlotType::L8Saturation | PlotType::L8Hue => Self {
                key_points: vec![0.0, 32.0, 64.0, 96.0, 128.0, 160.0, 192.0, 224.0, 256.0],
                range: 0_f64..256.0_f64,
                mapper: |value, limit| {
                    let norm = value / 256.0;
                    let size = limit.1 - limit.0;
                    (norm * size as f64).round() as i32 + limit.0
                },
                formatter: |value| format!("{value}"),
            },
        }
    }
}

impl Ranged for PlotCoord {
    type FormatOption = NoDefaultFormatting;
    type ValueType = f64;

    fn map(&self, value: &f64, limit: (i32, i32)) -> i32 {
        (self.mapper)(value, limit)
    }

    fn key_points<Hint: KeyPointHint>(&self, _hint: Hint) -> Vec<f64> {
        self.key_points.clone()
    }

    fn range(&self) -> Range<f64> {
        self.range.clone()
    }
}

impl ValueFormatter<f64> for PlotCoord {
    fn format_ext(&self, value: &f64) -> String {
        (self.formatter)(value)
    }
}
