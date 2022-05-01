use clap::{Args, ValueHint};
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct ConvertArgs {
    #[clap(
        name = "input",
        help = "Sets the input HEVC file to use, or piped with -",
        long,
        short = 'i',
        conflicts_with = "input_pos",
        required_unless_present = "input_pos",
        value_hint = ValueHint::FilePath,
    )]
    pub input: Option<PathBuf>,

    #[clap(
        name = "input_pos",
        help = "Sets the input HEVC file to use, or piped with - (positional)",
        conflicts_with = "input",
        required_unless_present = "input",
        value_hint = ValueHint::FilePath
    )]
    pub input_pos: Option<PathBuf>,

    #[clap(
        long,
        short = 'o',
        help = "Converted single layer output file location",
        value_hint = ValueHint::FilePath
    )]
    pub output: Option<PathBuf>,

    #[clap(short = 'd', long, help = "Discard the EL stream")]
    pub discard: bool,
}
