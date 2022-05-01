use clap::Parser;

mod convert;
mod demux;
mod editor;
mod export;
mod extract_rpu;
mod generate;
mod info;
mod inject_rpu;
mod mux;

pub use convert::ConvertArgs;
pub use demux::DemuxArgs;
pub use editor::EditorArgs;
pub use export::ExportArgs;
pub use extract_rpu::ExtractRpuArgs;
pub use generate::GenerateArgs;
pub use info::InfoArgs;
pub use inject_rpu::InjectRpuArgs;
pub use mux::MuxArgs;

#[derive(Parser, Debug)]
pub enum Command {
    #[clap(about = "Converts RPU within a single layer HEVC file")]
    Convert(ConvertArgs),

    #[clap(
        about = "Demuxes single track dual layer Dolby Vision into Base layer and Enhancement layer files"
    )]
    Demux(DemuxArgs),

    #[clap(about = "Edits a binary RPU according to a JSON config")]
    Editor(EditorArgs),

    #[clap(about = "Exports a binary RPU file to JSON for simpler analysis")]
    Export(ExportArgs),

    #[clap(about = "Extracts Dolby Vision RPU from an HEVC file")]
    ExtractRpu(ExtractRpuArgs),

    #[clap(about = "Interleaves RPU NAL units between slices in an HEVC encoded bitstream")]
    InjectRpu(InjectRpuArgs),

    #[clap(about = "Generates a binary RPU from different sources")]
    Generate(GenerateArgs),

    #[clap(about = "Prints the parsed RPU data as JSON for a specific frame")]
    Info(InfoArgs),

    #[clap(about = "Interleaves the enhancement layer into a base layer HEVC bitstream")]
    Mux(MuxArgs),
}
