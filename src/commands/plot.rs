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

    #[arg(long, help = "Plot L2 trims metadata instead of L1 dynamic brightness")]
    pub l2: bool,
}
