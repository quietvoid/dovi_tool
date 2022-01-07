use clap::Parser;
use regex::Regex;
use std::path::Path;

use anyhow::{bail, format_err, Result};
use bitvec_helpers::bitvec_writer;

mod commands;
use commands::Command;

mod dovi;
use dovi::{
    converter::Converter, demuxer::Demuxer, editor::Editor, exporter::Exporter,
    generator::Generator, rpu_extractor::RpuExtractor, rpu_info::RpuInfo,
    rpu_injector::RpuInjector, CliOptions, Format,
};

#[derive(Parser, Debug)]
#[clap(name = env!("CARGO_PKG_NAME"), about = "Stuff about Dolby Vision", author = "quietvoid", version = env!("CARGO_PKG_VERSION"))]
struct Opt {
    #[clap(
        name = "mode",
        short = 'm',
        long,
        help = "Sets the mode for RPU processing. --help for more info",
        long_help = "Sets the mode for RPU processing.\nMode 1: Converts the RPU to be MEL compatible\nMode 2: Converts the RPU to be profile 8.1 compatible\nMode 3: Converts profile 5 to 8.1"
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

    let res = match opt.cmd {
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
        } => RpuInjector::inject_rpu(input, rpu_in, output, cli_options),
        Command::Info { input, frame } => RpuInfo::info(input, frame),
        Command::Generate { .. } => {
            let mut generator = Generator::from_command(opt.cmd)?;
            generator.generate()
        }
        Command::Export { input, output } => Exporter::export(input, output),
    };

    if let Err(e) = res {
        println!("Error: {:?}", e);
    }

    Ok(())
}

pub fn input_format(input: &Path) -> Result<Format> {
    let regex = Regex::new(r"\.(hevc|.?265|mkv)")?;
    let file_name = match input.file_name() {
        Some(file_name) => file_name
            .to_str()
            .ok_or_else(|| format_err!("Invalid file name"))?,
        None => "",
    };

    if file_name == "-" {
        Ok(Format::RawStdin)
    } else if regex.is_match(file_name) && input.is_file() {
        if file_name.ends_with(".mkv") {
            Ok(Format::Matroska)
        } else {
            Ok(Format::Raw)
        }
    } else if file_name.is_empty() {
        bail!("Missing input.")
    } else if !input.is_file() {
        bail!("Input file doesn't exist.")
    } else {
        bail!("Invalid input file type.")
    }
}
