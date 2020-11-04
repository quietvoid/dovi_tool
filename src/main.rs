use regex::Regex;
use std::path::PathBuf;
use structopt::StructOpt;

mod dovi;
use dovi::{demuxer::Demuxer, Format};

#[derive(StructOpt, Debug)]
#[structopt(name = "dovi_tool", about = "Stuff about Dolby Vision")]
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
        help = "BL output file location",
        parse(from_os_str)
    )]
    bl_out: Option<PathBuf>,

    #[structopt(
        short = "o",
        long,
        help = "EL output file location",
        parse(from_os_str)
    )]
    el_out: Option<PathBuf>,
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

    match input_format(&input) {
        Ok(format) => {
            let bl_out = match opt.bl_out {
                Some(path) => path,
                None => PathBuf::from("BL.hevc"),
            };

            let el_out = match opt.el_out {
                Some(path) => path,
                None => PathBuf::from("EL.hevc"),
            };

            let parser = Demuxer::new(format, input, bl_out, el_out);
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
