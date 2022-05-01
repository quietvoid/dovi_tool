use clap::{Args, ValueHint};
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct EditorArgs {
    #[clap(
        name = "input",
        help = "Sets the input RPU file to use",
        long,
        short = 'i',
        conflicts_with = "input_pos",
        required_unless_present = "input_pos",
        value_hint = ValueHint::FilePath,
    )]
    pub input: Option<PathBuf>,

    #[clap(
        name = "input_pos",
        help = "Sets the input RPU file to use (positional)",
        conflicts_with = "input",
        required_unless_present = "input",
        value_hint = ValueHint::FilePath
    )]
    pub input_pos: Option<PathBuf>,

    #[clap(
        name = "json",
        long,
        short = 'j',
        help = "Sets the edit JSON file to use",
        value_hint = ValueHint::FilePath
    )]
    pub json_file: PathBuf,

    #[clap(
        long,
        short = 'o',
        help = "Modified RPU output file location",
        value_hint = ValueHint::FilePath
    )]
    pub rpu_out: Option<PathBuf>,
}
