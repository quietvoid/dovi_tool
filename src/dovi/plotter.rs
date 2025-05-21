use std::fmt::Write;
use std::ops::Range;
use std::path::PathBuf;

#[cfg(not(feature = "system-font"))]
use anyhow::bail;

use anyhow::Result;
use plotters::coord::ranged1d::{KeyPointHint, NoDefaultFormatting, Ranged, ValueFormatter};
use plotters::coord::types::RangedCoordusize;
use plotters::prelude::{
    AreaSeries, BitMapBackend, Cartesian2d, ChartBuilder, ChartContext, IntoDrawingArea,
    LineSeries, PathElement, SeriesLabelPosition, WHITE,
};
use plotters::style::{BLACK, Color, IntoTextStyle, RGBColor, ShapeStyle};

use dolby_vision::rpu::utils::parse_rpu_file;
use dolby_vision::utils::{nits_to_pq, pq_to_nits};

use super::input_from_either;
use super::rpu_info::{L2Data, RpusListSummary};
use crate::commands::PlotArgs;

#[cfg(not(feature = "system-font"))]
const NOTO_SANS_REGULAR: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/NotoSans-Regular.ttf"
));

const MAX_COLOR: RGBColor = RGBColor(65, 105, 225);
const AVERAGE_COLOR: RGBColor = RGBColor(75, 0, 130);

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
            l2,
        } = args;

        let (level, y_desc) = if l2 { (2, "") } else { (1, "nits (cd/m²)") };

        let output = output.unwrap_or(PathBuf::from(format!("L{level}_plot.png")));
        let title = title.unwrap_or(format!("Dolby Vision L{level} plot"));

        let input = input_from_either("info", input, input_pos)?;
        let plotter = Plotter { input };

        println!("Parsing RPU file...");
        let orig_rpus = parse_rpu_file(plotter.input)?;

        // inclusive range, end must be last RPU index
        let start = start_arg.unwrap_or(0);
        let end = end_arg.unwrap_or(orig_rpus.len() - 1);
        let rpus = &orig_rpus[start..=end];

        let x_spec = 0..rpus.len();

        let root = BitMapBackend::new(&output, (3000, 1200)).into_drawing_area();
        root.fill(&WHITE)?;
        let root = root
            .margin(30, 30, 60, 60)
            .titled(&title, ("sans-serif", 40))?;

        println!("Plotting...");
        let summary = if l2 {
            RpusListSummary::with_l2_data(rpus)?
        } else {
            RpusListSummary::new(rpus)?
        };

        let mut chart = ChartBuilder::on(&root)
            .x_label_area_size(60)
            .y_label_area_size(60)
            .margin_top(90)
            .build_cartesian_2d(x_spec, PqCoord::for_level(level))?;

        chart
            .configure_mesh()
            .bold_line_style(BLACK.mix(0.10))
            .light_line_style(BLACK.mix(0.01))
            .label_style(("sans-serif", 22))
            .axis_desc_style(("sans-serif", 24))
            .x_desc("frames")
            .x_max_light_lines(1)
            .x_labels(24)
            .y_desc(y_desc)
            .draw()?;

        if l2 {
            Self::draw_l2_series(&mut chart, &summary)?;
        } else {
            Self::draw_l1_series(&mut chart, &summary)?;
        }

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
        let l2_stats = summary.l2_stats.as_ref().unwrap();

        type Series = (&'static str, fn(&L2Data) -> f64, (f64, f64, f64), RGBColor);
        let series: [Series; 6] = [
            (
                "slope (gain)",
                |e| e.0 as f64,
                l2_stats.slope,
                RGBColor(96, 158, 232), // blue
            ),
            (
                "offset (lift)",
                |e| e.1 as f64,
                l2_stats.offset,
                RGBColor(230, 110, 132), // pink
            ),
            (
                "power (gamma)",
                |e| e.2 as f64,
                l2_stats.power,
                RGBColor(236, 162, 75), // orange
            ),
            (
                "chroma (weight)",
                |e| e.3 as f64,
                l2_stats.chroma,
                RGBColor(115, 187, 190), // cyan
            ),
            (
                "saturation (gain)",
                |e| e.4 as f64,
                l2_stats.saturation,
                RGBColor(144, 106, 252), // purple
            ),
            (
                "ms (weight)",
                |e| e.5 as f64,
                l2_stats.ms_weight,
                RGBColor(243, 205, 95), // yellow
            ),
        ];

        for (name, field_extractor, stats, color) in series.iter() {
            let label = format!(
                "{name} (min: {:.0}, max: {:.0}, avg: {:.0})",
                stats.0, stats.1, stats.2
            );
            let series = LineSeries::new(
                (0..).zip(data.iter()).map(|(x, y)| (x, field_extractor(y))),
                color,
            );
            let shape_style = ShapeStyle {
                color: color.to_rgba(),
                filled: false,
                stroke_width: 2,
            };

            chart
                .draw_series(series)?
                .label(label)
                .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], shape_style));
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

impl PqCoord {
    pub fn for_level(level: u8) -> PqCoord {
        if level == 2 {
            PqCoord {
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
            }
        } else {
            PqCoord {
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
            }
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
