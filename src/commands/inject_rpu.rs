use clap::{Args, ValueHint};
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct InjectRpuArgs {
    #[clap(
        name = "input",
        help = "Sets the input HEVC file to use",
        long,
        short = 'i',
        conflicts_with = "input_pos",
        required_unless_present = "input_pos",
        value_hint = ValueHint::FilePath,
    )]
    pub input: Option<PathBuf>,

    #[clap(
        name = "input_pos",
        help = "Sets the input HEVC file to use (positional)",
        conflicts_with = "input",
        required_unless_present = "input",
        value_hint = ValueHint::FilePath
    )]
    pub input_pos: Option<PathBuf>,

    #[clap(long, short = 'r', help = "Sets the input RPU file to use", value_hint = ValueHint::FilePath)]
    pub rpu_in: PathBuf,

    #[clap(
        long,
        short = 'o',
        help = "Output HEVC file location",
        value_hint = ValueHint::FilePath
    )]
    pub output: Option<PathBuf>,

    #[clap(
        long,
        takes_value = false,
        help = "Disable adding AUD NALUs between frames"
    )]
    pub no_add_aud: bool,
}
