use clap::{Args, ValueHint};
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct EditorArgs {
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
        id = "json",
        long,
        short = 'j',
        help = "Sets the edit JSON file to use",
        value_hint = ValueHint::FilePath
    )]
    pub json_file: PathBuf,

    #[arg(
        long,
        short = 'o',
        help = "Modified RPU output file location",
        value_hint = ValueHint::FilePath
    )]
    pub rpu_out: Option<PathBuf>,
}
