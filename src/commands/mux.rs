use clap::{Args, ValueHint};
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct MuxArgs {
    #[arg(
        id = "bl",
        long,
        short = 'b',
        help = "Sets the base layer HEVC file to use",
        value_hint = ValueHint::FilePath
    )]
    pub bl: PathBuf,

    #[arg(
        id = "el",
        long,
        short = 'e',
        help = "Sets the input enhancement layer HEVC file to use",
        value_hint = ValueHint::FilePath
    )]
    pub el: PathBuf,

    #[arg(
        long,
        short = 'o',
        help = "Output BL+EL+RPU HEVC file location",
        value_hint = ValueHint::FilePath
    )]
    pub output: Option<PathBuf>,

    #[arg(long, num_args = 0, help = "Disable adding AUD NALUs between frames")]
    pub no_add_aud: bool,

    #[arg(long, help = "Removes EOS/EOB NALUs from both BL and EL, if present")]
    pub remove_eos: bool,

    #[arg(
        short = 'd',
        long,
        help = "Discard the EL video NALUs, keeping only the RPU"
    )]
    pub discard: bool,

    #[arg(long, help = "Deprecated, enabled by default")]
    pub eos_before_el: bool,
}
