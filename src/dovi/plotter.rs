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
    PathElement, SeriesLabelPosition, WHITE,
};
use plotters::style::{BLACK, Color, IntoTextStyle, RGBColor, ShapeStyle};

use dolby_vision::rpu::utils::parse_rpu_file;
use dolby_vision::utils::{nits_to_pq, pq_to_nits};

use super::input_from_either;
use super::rpu_info::RpusListSummary;
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
        } = args;

        let output = output.unwrap_or(PathBuf::from("L1_plot.png"));
        let title = title.unwrap_or(String::from("Dolby Vision L1 plot"));

        let input = input_from_either("info", input, input_pos)?;
        let plotter = Plotter { input };

        println!("Parsing RPU file...");
        let rpus = parse_rpu_file(plotter.input)?;

        let x_spec = 0..rpus.len();

        let root = BitMapBackend::new(&output, (3000, 1200)).into_drawing_area();
        root.fill(&WHITE)?;
        let root = root
            .margin(30, 30, 60, 60)
            .titled(&title, ("sans-serif", 40))?;

        println!("Plotting...");
        let summary = RpusListSummary::new(&rpus)?;

        let mut chart = ChartBuilder::on(&root)
            .x_label_area_size(60)
            .y_label_area_size(60)
            .margin_top(90)
            .build_cartesian_2d(x_spec, PqCoord {})?;

        chart
            .configure_mesh()
            .bold_line_style(BLACK.mix(0.10))
            .light_line_style(BLACK.mix(0.01))
            .label_style(("sans-serif", 22))
            .axis_desc_style(("sans-serif", 24))
            .x_desc("frames")
            .x_max_light_lines(1)
            .x_labels(24)
            .y_desc("nits (cd/m²)")
            .draw()?;

        Self::draw_l1_series(&mut chart, &summary)?;
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

        if !summary.l2_trims.is_empty() {
            let caption = format!("L2 trims: {}", summary.l2_trims.join(", "));
            let pos = (
                (root.dim_in_pixel().0 - root.estimate_text_size(&caption, &caption_style)?.0)
                    as i32,
                60,
            );
            root.draw_text(&caption, &caption_style, pos)?;
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
}

pub struct PqCoord {}

impl Ranged for PqCoord {
    type FormatOption = NoDefaultFormatting;
    type ValueType = f64;

    fn map(&self, value: &f64, limit: (i32, i32)) -> i32 {
        let size = limit.1 - limit.0;
        (*value * size as f64) as i32 + limit.0
    }

    fn key_points<Hint: KeyPointHint>(&self, _hint: Hint) -> Vec<f64> {
        vec![
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
        ]
    }

    fn range(&self) -> Range<f64> {
        0_f64..1.0_f64
    }
}

impl ValueFormatter<f64> for PqCoord {
    fn format_ext(&self, value: &f64) -> String {
        let nits = (pq_to_nits(*value) * 1000.0).round() / 1000.0;
        format!("{nits}")
    }
}
