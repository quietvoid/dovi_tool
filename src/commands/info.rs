use clap::{Args, ValueHint};
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct InfoArgs {
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
        id = "frame",
        long,
        short = 'f',
        help = "Frame number to show info for"
    )]
    pub frame: Option<usize>,

    #[arg(id = "summary", long, short = 's', help = "Show the RPU summary")]
    pub summary: bool,

    #[arg(
        id = "qpfile",
        help = "set the path of the qpfile to create",
        long,
        short = 'q',
        conflicts_with = "frame",
        value_hint = ValueHint::FilePath,
    )]
    pub qpfile: Option<PathBuf>,
}
