use regex::Regex;
use std::path::PathBuf;
use structopt::StructOpt;

mod dovi;
use dovi::parser::{Format, Parser};

#[derive(StructOpt, Debug)]
#[structopt(
    name = "hdr10plus_parser",
    about = "Parses HDR10+ dynamic metadata in HEVC video files"
)]
struct Opt {
    #[structopt(
        name = "input",
        short = "i",
        long,
        help = "Sets the input file to use",
        long,
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

    #[structopt(
        short = "o",
        long,
        help = "Sets the output JSON file to use",
        parse(from_os_str)
    )]
    output: Option<PathBuf>,

    #[structopt(long, help = "Checks if input file contains dynamic metadata")]
    verify: bool,

    #[structopt(
        long,
        help = "Force only one metadata profile, avoiding mixing different profiles (fix for x265 segfault)"
    )]
    force_single_profile: bool,
}

fn main() -> std::io::Result<()> {
    let opt = Opt::from_args();

    let input = match opt.input {
        Some(input) => input,
        None => match opt.stdin {
            Some(stdin) => stdin,
            None => PathBuf::new(),
        },
    };

    let verify = opt.verify || opt.output.is_none();

    match input_format(&input) {
        Ok(format) => {
            let parser = Parser::new(format, input, opt.output, verify, opt.force_single_profile);
            parser.process_file();
        }
        Err(msg) => println!("{}", msg),
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
