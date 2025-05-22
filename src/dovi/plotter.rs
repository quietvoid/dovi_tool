use std::fmt::Write;
use std::ops::Range;
use std::path::PathBuf;

use super::input_from_either;
use super::rpu_info::RpusListSummary;
use crate::commands::PlotArgs;
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

#[cfg(not(feature = "system-font"))]
const NOTO_SANS_REGULAR: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/NotoSans-Regular.ttf"
));

const MAX_COLOR: RGBColor = RGBColor(65, 105, 225);
const AVERAGE_COLOR: RGBColor = RGBColor(75, 0, 130);
const COLORS: [RGBColor; 6] = [
    RGBColor(220, 38, 38),  // red
    RGBColor(234, 179, 8),  // yellow
    RGBColor(34, 197, 94),  // green
    RGBColor(34, 211, 238), // cyan
    RGBColor(59, 130, 246), // blue
    RGBColor(236, 72, 153), // magenta
];

type Series<T> = (&'static str, fn(&T) -> f64, (f64, f64, f64));

#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlotType {
    /// L1 Dynamic Brightness
    L1,
    /// L2 Trims
    L2,
    /// L8 Trims (CM v4.0 RPU required)
    L8T,
    /// L8 Saturation Vectors (CM v4.0 RPU required)
    L8S,
    /// L8 Hue Vectors (CM v4.0 RPU required)
    L8H,
}

impl PlotType {
    pub fn name(&self) -> &str {
        match self {
            PlotType::L1 => "L1 Dynamic Brightness",
            PlotType::L2 => "L2 Trims",
            PlotType::L8T => "L8 Trims",
            PlotType::L8S => "L8 Saturation Vectors",
            PlotType::L8H => "L8 Hue Vectors",
        }
    }

    pub fn default_title(&self) -> String {
        format!("Dolby Vision {}", self.name())
    }

    pub fn default_output(&self) -> &str {
        match self {
            PlotType::L1 => "L1_plot.png",
            PlotType::L2 => "L2_plot.png",
            PlotType::L8T => "L8-trims_plot.png",
            PlotType::L8S => "L8-saturation_plot.png",
            PlotType::L8H => "L8-hue_plot.png",
        }
    }

    pub fn y_desc(&self) -> &str {
        match self {
            PlotType::L1 => "nits (cd/m²)",
            _ => "",
        }
    }

    pub fn requires_dmv2(&self) -> bool {
        !matches!(self, PlotType::L1 | PlotType::L2)
    }

    pub fn summary(&self, rpus: &[DoviRpu]) -> Result<RpusListSummary> {
        match self {
            PlotType::L1 => RpusListSummary::new(rpus),
            PlotType::L2 => RpusListSummary::with_l2_data(rpus),
            PlotType::L8T => RpusListSummary::with_l8_trims_data(rpus),
            PlotType::L8S => RpusListSummary::with_l8_saturation_data(rpus),
            PlotType::L8H => RpusListSummary::with_l8_hue_data(rpus),
        }
    }

    pub fn draw_series(
        &self,
        chart: &mut ChartContext<BitMapBackend, Cartesian2d<RangedCoordusize, PqCoord>>,
        summary: &RpusListSummary,
    ) -> Result<()> {
        match self {
            PlotType::L1 => Plotter::draw_l1_series(chart, summary),
            PlotType::L2 => Plotter::draw_l2_series(chart, summary),
            PlotType::L8T => Plotter::draw_l8_trims_series(chart, summary),
            PlotType::L8S => Plotter::draw_l8_saturation_series(chart, summary),
            PlotType::L8H => Plotter::draw_l8_hue_series(chart, summary),
        }
    }
}

pub struct Plotter {
    input: PathBuf,
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
        } = args;

        let output = output.unwrap_or(PathBuf::from(plot_type.default_output()));
        let title = title.unwrap_or(plot_type.default_title().to_string());

        let input = input_from_either("info", input, input_pos)?;
        let plotter = Plotter { input };

        println!("Parsing RPU file...");
        let orig_rpus = parse_rpu_file(plotter.input)?;

        // inclusive range, end must be last RPU index
        let start = start_arg.unwrap_or(0);
        let end = end_arg.unwrap_or(orig_rpus.len() - 1);
        let rpus = &orig_rpus[start..=end];

        println!("Plotting...");
        let summary = plot_type.summary(rpus)?;

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
            .build_cartesian_2d(x_spec, PqCoord::from(plot_type))?;

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

        plot_type.draw_series(&mut chart, &summary)?;

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

        let caption_style = ("sans-serif", 24).into_text_style(&root);
        root.draw_text(&chart_caption, &caption_style, (60, 10))?;
        root.draw_text(&summary.rpu_mastering_meta_str, &caption_style, (60, 35))?;
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
        chart: &mut ChartContext<BitMapBackend, Cartesian2d<RangedCoordusize, PqCoord>>,
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
            (0..).zip(data.iter()).map(|(x, y)| (x, y.1)),
            0.0,
            MAX_COLOR.mix(0.25),
        )
        .border_style(MAX_COLOR);
        let avg_series = AreaSeries::new(
            (0..).zip(data.iter()).map(|(x, y)| (x, y.2)),
            0.0,
            AVERAGE_COLOR.mix(0.50),
        )
        .border_style(AVERAGE_COLOR);
        let min_series = AreaSeries::new(
            (0..).zip(data.iter()).map(|(x, y)| (x, y.0)),
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
        chart: &mut ChartContext<BitMapBackend, Cartesian2d<RangedCoordusize, PqCoord>>,
        summary: &RpusListSummary,
    ) -> Result<()> {
        let data = summary.l2_data.as_ref().unwrap();
        let stats = summary.l2_stats.as_ref().unwrap();

        let series: [Series<ExtMetadataBlockLevel2>; 6] = [
            ("slope (gain)", |e| e.trim_slope as f64, stats.slope),
            ("offset (lift)", |e| e.trim_offset as f64, stats.offset),
            ("power (gamma)", |e| e.trim_power as f64, stats.power),
            (
                "chroma (weight)",
                |e| e.trim_chroma_weight as f64,
                stats.chroma,
            ),
            (
                "saturation (gain)",
                |e| e.trim_saturation_gain as f64,
                stats.saturation,
            ),
            ("ms (weight)", |e| e.ms_weight as f64, stats.ms_weight),
        ];

        Self::draw_line_series(chart, data, &series)
    }

    fn draw_l8_trims_series(
        chart: &mut ChartContext<BitMapBackend, Cartesian2d<RangedCoordusize, PqCoord>>,
        summary: &RpusListSummary,
    ) -> Result<()> {
        let data = summary.l8_data.as_ref().unwrap();
        let stats = summary.l8_stats_trims.as_ref().unwrap();

        let series: [Series<ExtMetadataBlockLevel8>; 6] = [
            ("slope (gain)", |e| e.trim_slope as f64, stats.slope),
            ("offset (lift)", |e| e.trim_offset as f64, stats.offset),
            ("power (gamma)", |e| e.trim_power as f64, stats.power),
            (
                "chroma (weight)",
                |e| e.trim_chroma_weight as f64,
                stats.chroma,
            ),
            (
                "saturation (gain)",
                |e| e.trim_saturation_gain as f64,
                stats.saturation,
            ),
            ("ms (weight)", |e| e.ms_weight as f64, stats.ms_weight),
        ];

        Self::draw_line_series(chart, data, &series)
    }

    fn draw_l8_saturation_series(
        chart: &mut ChartContext<BitMapBackend, Cartesian2d<RangedCoordusize, PqCoord>>,
        summary: &RpusListSummary,
    ) -> Result<()> {
        let data = summary.l8_data.as_ref().unwrap();
        let stats = summary.l8_stats_saturation.as_ref().unwrap();

        let series: [Series<ExtMetadataBlockLevel8>; 6] = [
            ("red", |e| e.saturation_vector_field0 as f64, stats.red),
            (
                "yellow",
                |e| e.saturation_vector_field1 as f64,
                stats.yellow,
            ),
            ("green", |e| e.saturation_vector_field2 as f64, stats.green),
            ("cyan", |e| e.saturation_vector_field3 as f64, stats.cyan),
            ("blue", |e| e.saturation_vector_field4 as f64, stats.blue),
            (
                "magenta",
                |e| e.saturation_vector_field5 as f64,
                stats.magenta,
            ),
        ];

        Self::draw_line_series(chart, data, &series)
    }

    fn draw_l8_hue_series(
        chart: &mut ChartContext<BitMapBackend, Cartesian2d<RangedCoordusize, PqCoord>>,
        summary: &RpusListSummary,
    ) -> Result<()> {
        let data = summary.l8_data.as_ref().unwrap();
        let stats = summary.l8_stats_hue.as_ref().unwrap();

        let series: [Series<ExtMetadataBlockLevel8>; 6] = [
            ("red", |e| e.hue_vector_field0 as f64, stats.red),
            ("yellow", |e| e.hue_vector_field1 as f64, stats.yellow),
            ("green", |e| e.hue_vector_field2 as f64, stats.green),
            ("cyan", |e| e.hue_vector_field3 as f64, stats.cyan),
            ("blue", |e| e.hue_vector_field4 as f64, stats.blue),
            ("magenta", |e| e.hue_vector_field5 as f64, stats.magenta),
        ];

        Self::draw_line_series(chart, data, &series)
    }

    fn draw_line_series<T>(
        chart: &mut ChartContext<BitMapBackend, Cartesian2d<RangedCoordusize, PqCoord>>,
        data: &[T],
        series: &[Series<T>],
    ) -> Result<()> {
        for ((name, field_extractor, stats), color) in series.iter().zip(COLORS.iter()) {
            let label = format!(
                "{name} (min: {:.0}, max: {:.0}, avg: {:.0})",
                stats.0, stats.1, stats.2
            );
            let series = LineSeries::new(
                (0..).zip(data.iter()).map(|(x, y)| (x, field_extractor(y))),
                color,
            );

            chart
                .draw_series(series)?
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

pub struct PqCoord {
    key_points: Vec<f64>,
    range: Range<f64>,
    mapper: fn(&f64, (i32, i32)) -> i32,
    formatter: fn(&f64) -> String,
}

impl From<PlotType> for PqCoord {
    fn from(plot_type: PlotType) -> Self {
        match plot_type {
            PlotType::L1 => PqCoord {
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
            PlotType::L2 | PlotType::L8T => PqCoord {
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
            PlotType::L8S | PlotType::L8H => PqCoord {
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

impl Ranged for PqCoord {
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

impl ValueFormatter<f64> for PqCoord {
    fn format_ext(&self, value: &f64) -> String {
        (self.formatter)(value)
    }
}
