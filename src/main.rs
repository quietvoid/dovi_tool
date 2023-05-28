use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, ValueHint};

#[cfg(test)]
mod tests;

mod commands;
use commands::{Command, ConversionModeCli};

mod dovi;
use dovi::{
    converter::Converter,
    demuxer::Demuxer,
    editor::{EditConfig, Editor},
    exporter::Exporter,
    generator::Generator,
    muxer::Muxer,
    plotter::Plotter,
    rpu_extractor::RpuExtractor,
    rpu_info::RpuInfo,
    rpu_injector::RpuInjector,
    CliOptions, WriteStartCodePreset,
};

#[derive(Parser, Debug)]
#[command(
    name = env!("CARGO_PKG_NAME"),
    about = "CLI tool combining multiple utilities for working with Dolby Vision",
    author = "quietvoid",
    version = option_env!("VERGEN_GIT_DESCRIBE").unwrap_or(env!("CARGO_PKG_VERSION"))
)]
struct Opt {
    #[arg(
        id = "mode",
        short = 'm',
        long,
        help = "Sets the mode for RPU processing. See --help for more info",
        long_help = "Sets the mode for RPU processing.\n  \
                     Mode 0: Parses the RPU, rewrites it untouched\n  \
                     Mode 1: Converts the RPU to be MEL compatible\n  \
                     Mode 2: Converts the RPU to be profile 8.1 compatible. Removes mapping\n  \
                     Mode 3: Converts profile 5 to 8.1\n  \
                     Mode 4: Converts to profile 8.4\n  \
                     Mode 5: Converts to profile 8.1, preserving luma/chroma mapping",
        value_enum
    )]
    mode: Option<ConversionModeCli>,

    #[arg(
        long,
        short = 'c',
        help = "Set active area offsets to 0 (meaning no letterbox bars)"
    )]
    crop: bool,

    #[arg(long, help = "Ignore HDR10+ metadata when writing the output HEVC.")]
    drop_hdr10plus: bool,

    #[arg(
        long,
        help = "Sets the edit JSON config file to use",
        value_hint = ValueHint::FilePath
    )]
    edit_config: Option<PathBuf>,

    #[arg(
        value_enum,
        long,
        help = "Start code to use when writing HEVC",
        default_value = "four"
    )]
    start_code: WriteStartCodePreset,

    #[command(subcommand)]
    cmd: Command,
}

fn main() -> Result<()> {
    let opt = Opt::parse();

    let edit_config = opt
        .edit_config
        .as_ref()
        .map(EditConfig::from_path)
        .and_then(Result::ok);

    let mut cli_options = CliOptions {
        mode: opt.mode,
        crop: opt.crop,
        discard_el: false,
        drop_hdr10plus: opt.drop_hdr10plus,
        edit_config,
        start_code: opt.start_code,
    };

    // Set mode 0 by default if cropping, otherwise it has no effect
    if cli_options.mode.is_none() && cli_options.crop {
        cli_options.mode = Some(ConversionModeCli::Lossless);
    }

    match opt.cmd {
        Command::Demux(args) => Demuxer::demux(args, cli_options),
        Command::Editor(args) => Editor::edit(args),
        Command::Convert(args) => Converter::convert(args, cli_options),
        Command::ExtractRpu(args) => RpuExtractor::extract_rpu(args, cli_options),
        Command::InjectRpu(args) => RpuInjector::inject_rpu(args, cli_options),
        Command::Info(args) => RpuInfo::info(args),
        Command::Generate(args) => Generator::generate(args),
        Command::Export(args) => Exporter::export(args),
        Command::Mux(args) => Muxer::mux_el(args, cli_options),
        Command::Plot(args) => Plotter::plot(args),
    }
}
