use regex::Regex;
use std::path::PathBuf;
use structopt::StructOpt;

mod bits;
use bits::{bitvec_reader, bitvec_writer};

mod dovi;
use dovi::{demuxer::Demuxer, rpu_extractor::RpuExtractor, Format};

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(
        name = "mode",
        short = "m",
        long,
        help = "Sets the mode for RPU processing. --help for more info",
        long_help = "Sets the mode for RPU processing.\nMode 1: Converts the RPU to be MEL compatible\nMode 2: Converts the RPU to be profile 8.1 compatible"
    )]
    mode: Option<u8>,

    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt, Debug)]
#[structopt(name = "dovi_tool", about = "Stuff about Dolby Vision")]
enum Command {
    Demux {
        #[structopt(
            name = "input",
            short = "i",
            long,
            help = "Sets the input file to use",
            conflicts_with = "stdin",
            parse(from_os_str)
        )]
        input: Option<PathBuf>,

        #[structopt(
            help = "Uses stdin as input data",
            conflicts_with = "input",
            parse(from_os_str)
        )]
        stdin: Option<PathBuf>,

        #[structopt(long, help = "BL output file location", parse(from_os_str))]
        bl_out: Option<PathBuf>,

        #[structopt(long, help = "EL output file location", parse(from_os_str))]
        el_out: Option<PathBuf>,
    },

    ExtractRpu {
        #[structopt(
            name = "input",
            short = "i",
            long,
            help = "Sets the input file to use",
            conflicts_with = "stdin",
            parse(from_os_str)
        )]
        input: Option<PathBuf>,

        #[structopt(
            help = "Uses stdin as input data",
            conflicts_with = "input",
            parse(from_os_str)
        )]
        stdin: Option<PathBuf>,

        #[structopt(long, help = "RPU output file location", parse(from_os_str))]
        rpu_out: Option<PathBuf>,
    },
}

fn main() -> std::io::Result<()> {
    let opt = Opt::from_args();

    match opt.cmd {
        Command::Demux {
            input,
            stdin,
            bl_out,
            el_out,
        } => {
            demux(input, stdin, bl_out, el_out, opt.mode);
        }
        Command::ExtractRpu {
            input,
            stdin,
            rpu_out,
        } => {
            extract_rpu(input, stdin, rpu_out, opt.mode);
        }
    }

    Ok(())
}

fn input_format(input: &PathBuf) -> Result<Format, &str> {
    let regex = Regex::new(r"\.(hevc|.?265|mkv)").unwrap();
    let file_name = match input.file_name() {
        Some(file_name) => file_name.to_str().unwrap(),
        None => "",
    };

    if file_name == "-" {
        Ok(Format::RawStdin)
    } else if regex.is_match(file_name) && input.is_file() {
        if file_name.contains("mkv") {
            Ok(Format::Matroska)
        } else {
            Ok(Format::Raw)
        }
    } else if file_name == "" {
        Err("Missing input.")
    } else if !input.is_file() {
        Err("Input file doesn't exist.")
    } else {
        Err("Invalid input file type.")
    }
}

fn demux(
    input: Option<PathBuf>,
    stdin: Option<PathBuf>,
    bl_out: Option<PathBuf>,
    el_out: Option<PathBuf>,
    mode: Option<u8>,
) {
    let input = match input {
        Some(input) => input,
        None => match stdin {
            Some(stdin) => stdin,
            None => PathBuf::new(),
        },
    };

    match input_format(&input) {
        Ok(format) => {
            let bl_out = match bl_out {
                Some(path) => path,
                None => PathBuf::from("BL.hevc"),
            };

            let el_out = match el_out {
                Some(path) => path,
                None => PathBuf::from("EL.hevc"),
            };

            let demuxer = Demuxer::new(format, input, bl_out, el_out);
            demuxer.process_input(mode);
        }
        Err(msg) => println!("{}", msg),
    }
}

fn extract_rpu(
    input: Option<PathBuf>,
    stdin: Option<PathBuf>,
    rpu_out: Option<PathBuf>,
    mode: Option<u8>,
) {
    let input = match input {
        Some(input) => input,
        None => match stdin {
            Some(stdin) => stdin,
            None => PathBuf::new(),
        },
    };

    match input_format(&input) {
        Ok(format) => {
            let rpu_out = match rpu_out {
                Some(path) => path,
                None => PathBuf::from("RPU.bin"),
            };

            let parser = RpuExtractor::new(format, input, rpu_out);
            parser.process_input(mode);
        }
        Err(msg) => println!("{}", msg),
    }
}
