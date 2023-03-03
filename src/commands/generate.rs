use clap::{Args, ValueHint};
use hdr10plus::metadata::PeakBrightnessSource;
use std::path::PathBuf;

use crate::dovi::generator::GeneratorProfile;

#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgHdr10PlusPeakBrightnessSource {
    /// The max value from the histogram measurements
    Histogram,
    /// The last percentile in the histogram, usually 99.98% brightness percentile
    Histogram99,
    /// The max value in `maxscl`
    MaxScl,
    /// The luminance calculated from the `maxscl` components
    /// Assumed BT.2020 primaries
    MaxSclLuminance,
}

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
        value_enum,
        long,
        help = "HDR10+: How to extract the peak brightness for the metadata",
        default_value = "histogram"
    )]
    pub hdr10plus_peak_source: Option<ArgHdr10PlusPeakBrightnessSource>,

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

impl From<ArgHdr10PlusPeakBrightnessSource> for PeakBrightnessSource {
    fn from(e: ArgHdr10PlusPeakBrightnessSource) -> Self {
        match e {
            ArgHdr10PlusPeakBrightnessSource::Histogram => Self::Histogram,
            ArgHdr10PlusPeakBrightnessSource::Histogram99 => Self::Histogram99,
            ArgHdr10PlusPeakBrightnessSource::MaxScl => Self::MaxScl,
            ArgHdr10PlusPeakBrightnessSource::MaxSclLuminance => Self::MaxSclLuminance,
        }
    }
}
