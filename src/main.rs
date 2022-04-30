use clap::Parser;

use anyhow::Result;
use bitvec_helpers::bitvec_writer;

mod commands;
use commands::Command;

mod dovi;
use dovi::{
    converter::Converter, demuxer::Demuxer, editor::Editor, exporter::Exporter,
    generator::Generator, muxer::Muxer, rpu_extractor::RpuExtractor, rpu_info::RpuInfo,
    rpu_injector::RpuInjector, CliOptions,
};

#[derive(Parser, Debug)]
#[clap(name = env!("CARGO_PKG_NAME"), about = "Stuff about Dolby Vision", author = "quietvoid", version = env!("CARGO_PKG_VERSION"))]
struct Opt {
    #[clap(
        name = "mode",
        short = 'm',
        long,
        help = "Sets the mode for RPU processing. See --help for more info",
        long_help = "Sets the mode for RPU processing.\n  \
                     Mode 1: Converts the RPU to be MEL compatible\n  \
                     Mode 2: Converts the RPU to be profile 8.1 compatible\n  \
                     Mode 3: Converts profile 5 to 8.1"
    )]
    mode: Option<u8>,

    #[clap(
        long,
        short = 'c',
        help = "Set active area offsets to 0 (meaning no letterbox bars)"
    )]
    crop: bool,

    #[clap(long, help = "Ignore HDR10+ metadata when writing the output HEVC.")]
    drop_hdr10plus: bool,

    #[clap(subcommand)]
    cmd: Command,
}

fn main() -> Result<()> {
    let opt = Opt::parse();

    let mut cli_options = CliOptions {
        mode: opt.mode,
        crop: opt.crop,
        discard_el: false,
        drop_hdr10plus: opt.drop_hdr10plus,
    };

    // Set mode 0 by default if cropping, otherwise it has no effect
    if cli_options.mode.is_none() && cli_options.crop {
        cli_options.mode = Some(0);
    }

    match opt.cmd {
        Command::Demux {
            input,
            stdin,
            bl_out,
            el_out,
            el_only,
        } => Demuxer::demux(input, stdin, bl_out, el_out, el_only, cli_options),
        Command::Editor {
            input,
            json_file,
            rpu_out,
        } => Editor::edit(input, json_file, rpu_out),
        Command::Convert {
            input,
            stdin,
            output,
            discard,
        } => {
            cli_options.discard_el = discard;
            Converter::convert(input, stdin, output, cli_options)
        }
        Command::ExtractRpu {
            input,
            stdin,
            rpu_out,
        } => RpuExtractor::extract_rpu(input, stdin, rpu_out, cli_options),
        Command::InjectRpu {
            input,
            rpu_in,
            output,
            no_add_aud,
        } => RpuInjector::inject_rpu(input, rpu_in, output, no_add_aud, cli_options),
        Command::Info { input, frame } => RpuInfo::info(input, frame),
        Command::Generate { .. } => {
            let mut generator = Generator::from_command(opt.cmd)?;
            generator.generate()
        }
        Command::Export { input, output } => Exporter::export(input, output),
        Command::Mux {
            bl,
            el,
            output,
            no_add_aud,
            eos_before_el,
            discard,
        } => {
            cli_options.discard_el = discard;
            Muxer::mux_el(bl, el, output, no_add_aud, eos_before_el, cli_options)
        }
    }
}
