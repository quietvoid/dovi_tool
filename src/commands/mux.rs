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

    #[arg(
        long,
        help = "Write the EOS/EOB NALUs before the EL. Defaults to false. See --help for more info",
        long_help = "Write the EOS/EOB NALUs before the EL. Defaults to false.\n\
                     In the case of the last frame containing EOS/EOB NALUs, they are written after the EL by default.\n\n\
                     This behaviour is different from yusesope and MakeMKV's mux, but conforms to the HEVC spec.\n\
                     To match their behaviour, enable the --eos-before-el flag."
    )]
    pub eos_before_el: bool,

    #[arg(
        short = 'd',
        long,
        help = "Discard the EL video NALUs, keeping only the RPU"
    )]
    pub discard: bool,
}
