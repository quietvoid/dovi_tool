use clap::{Args, ValueHint};
use std::path::PathBuf;

use crate::dovi::generator::GeneratorProfile;

#[derive(Args, Debug, Default)]
pub struct GenerateArgs {
    #[arg(
        id = "json",
        long,
        short = 'j',
        help = "Sets the generator config JSON file to use",
        conflicts_with = "xml",
        required_unless_present = "xml",
        value_hint = ValueHint::FilePath
    )]
    pub json_file: Option<PathBuf>,

    #[arg(
        long,
        short = 'o',
        help = "Generated RPU output file location",
        value_hint = ValueHint::FilePath
    )]
    pub rpu_out: Option<PathBuf>,

    #[arg(
        id = "hdr10plus-json",
        long,
        help = "HDR10+ JSON file to generate from",
        conflicts_with = "madvr-file",
        value_hint = ValueHint::FilePath,
    )]
    pub hdr10plus_json: Option<PathBuf>,

    #[arg(
        short = 'x',
        long,
        help = "XML metadata file to generate from",
        conflicts_with_all = &["json", "hdr10plus-json", "madvr-file"],
        required_unless_present = "json",
        value_hint = ValueHint::FilePath
    )]
    pub xml: Option<PathBuf>,

    #[arg(long, help = "Canvas width for L5 metadata generation")]
    pub canvas_width: Option<u16>,

    #[arg(long, help = "Canvas height for L5 metadata generation")]
    pub canvas_height: Option<u16>,

    #[arg(
        id = "madvr-file",
        long,
        help = "madVR measurement file to generate from",
        value_hint = ValueHint::FilePath
    )]
    pub madvr_file: Option<PathBuf>,

    #[arg(
        long,
        help = "madVR source: use custom per-frame target nits if available"
    )]
    pub use_custom_targets: bool,

    #[arg(
        value_enum,
        short = 'p',
        long,
        help = "Dolby Vision profile to generate"
    )]
    pub profile: Option<GeneratorProfile>,

    #[arg(long, help = "Set scene cut flag for every frame")]
    pub long_play_mode: Option<bool>,
}
