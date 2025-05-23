use crate::dovi::plotter::{PlotType, TrimsFilter};
use clap::{Args, ValueHint};
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct PlotArgs {
    #[arg(
        id = "input",
        help = "Sets the input RPU file to use",
        long,
        short = 'i',
        conflicts_with = "input_pos",
        required_unless_present = "input_pos",
        value_hint = ValueHint::FilePath,
    )]
    pub input: Option<PathBuf>,

    #[arg(
        id = "input_pos",
        help = "Sets the input RPU file to use (positional)",
        conflicts_with = "input",
        required_unless_present = "input",
        value_hint = ValueHint::FilePath
    )]
    pub input_pos: Option<PathBuf>,

    #[arg(
        long,
        short = 'o',
        help = "Output PNG image file location",
        value_hint = ValueHint::FilePath
    )]
    pub output: Option<PathBuf>,

    #[arg(long, short = 't', help = "Title to use at the top")]
    pub title: Option<String>,

    #[arg(long, short = 's', help = "Set frame range start")]
    pub start: Option<usize>,

    #[arg(long, short = 'e', help = "Set frame range end (inclusive)")]
    pub end: Option<usize>,

    #[arg(
        long,
        short = 'p',
        help = "Sets the DV metadata level to plot",
        value_enum,
        default_value = "l1"
    )]
    pub plot_type: PlotType,

    #[arg(
        long = "target-nits",
        help = "Target brightness in nits for L2/L8 plots",
        default_value = "100",
        value_parser = ["100", "300", "600", "1000"]
    )]
    pub target_nits_str: String,

    #[arg(
        long,
        help = "Parameters to exclude from L2/L8 trims plots\nSupports multiple occurrences prefixed by --trims-filter or delimited by ','",
        value_enum,
        value_delimiter = ','
    )]
    pub trims_filter: Vec<TrimsFilter>,
}
