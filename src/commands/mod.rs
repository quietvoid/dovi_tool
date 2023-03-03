use clap::Parser;

use dolby_vision::rpu::ConversionMode;

mod convert;
mod demux;
mod editor;
mod export;
mod extract_rpu;
mod generate;
mod info;
mod inject_rpu;
mod mux;
mod plot;

pub use convert::ConvertArgs;
pub use demux::DemuxArgs;
pub use editor::EditorArgs;
pub use export::ExportArgs;
pub use extract_rpu::ExtractRpuArgs;
pub use generate::{ArgHdr10PlusPeakBrightnessSource, GenerateArgs};
pub use info::InfoArgs;
pub use inject_rpu::InjectRpuArgs;
pub use mux::MuxArgs;
pub use plot::PlotArgs;

#[derive(Parser, Debug)]
pub enum Command {
    #[command(about = "Converts RPU within a single layer HEVC file")]
    Convert(ConvertArgs),

    #[command(
        about = "Demuxes single track dual layer Dolby Vision into Base layer and Enhancement layer files"
    )]
    Demux(DemuxArgs),

    #[command(about = "Edits a binary RPU according to a JSON config")]
    Editor(EditorArgs),

    #[command(about = "Exports a binary RPU file to JSON for simpler analysis")]
    Export(ExportArgs),

    #[command(about = "Extracts Dolby Vision RPU from an HEVC file")]
    ExtractRpu(ExtractRpuArgs),

    #[command(about = "Interleaves RPU NAL units between slices in an HEVC encoded bitstream")]
    InjectRpu(InjectRpuArgs),

    #[command(about = "Generates a binary RPU from different sources")]
    Generate(GenerateArgs),

    #[command(about = "Prints the parsed RPU data as JSON for a specific frame")]
    Info(InfoArgs),

    #[command(about = "Interleaves the enhancement layer into a base layer HEVC bitstream")]
    Mux(MuxArgs),

    #[command(about = "Plot the L1 dynamic brightness metadata")]
    Plot(PlotArgs),
}

#[derive(clap::ValueEnum, Debug, Copy, Clone)]
pub enum ConversionModeCli {
    #[value(name = "0")]
    Lossless = 0,
    #[value(name = "1")]
    ToMel,
    #[value(name = "2")]
    To81,
    #[value(name = "3")]
    Profile5To81,
    #[value(name = "4")]
    To84,
    #[value(name = "5")]
    To81MappingPreserved,
}

impl From<ConversionModeCli> for ConversionMode {
    fn from(mode: ConversionModeCli) -> ConversionMode {
        match mode {
            ConversionModeCli::Lossless => ConversionMode::Lossless,
            ConversionModeCli::ToMel => ConversionMode::ToMel,
            ConversionModeCli::To81 | ConversionModeCli::Profile5To81 => ConversionMode::To81,
            ConversionModeCli::To84 => ConversionMode::To84,
            ConversionModeCli::To81MappingPreserved => ConversionMode::To81MappingPreserved,
        }
    }
}
