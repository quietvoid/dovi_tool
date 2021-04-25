pub mod converter;
pub mod demuxer;
pub mod editor;
pub mod rpu_extractor;

mod io;
mod rpu;

use hevc_parser::HevcParser;
use rpu::{parse_dovi_rpu, DoviRpu};

use indicatif::{ProgressBar, ProgressStyle};
use std::io::{BufReader, Read, Write};
use std::{fs::File, io::BufWriter, path::Path};

use super::bitvec_reader::BitVecReader;
use super::bitvec_writer::BitVecWriter;
use super::input_format;

const OUT_NAL_HEADER: &[u8] = &[0, 0, 0, 1];

#[derive(Debug, PartialEq)]
pub enum Format {
    Raw,
    RawStdin,
    Matroska,
}

#[derive(Debug)]
pub struct RpuOptions {
    pub mode: Option<u8>,
    pub crop: bool,
    pub discard_el: bool,
}

pub fn initialize_progress_bar(format: &Format, input: &Path) -> ProgressBar {
    let pb: ProgressBar;
    let bytes_count;

    if let Format::RawStdin = format {
        pb = ProgressBar::hidden();
    } else {
        let file = File::open(input).expect("No file found");

        //Info for indicatif ProgressBar
        let file_meta = file.metadata();
        bytes_count = file_meta.unwrap().len() / 100_000_000;

        pb = ProgressBar::new(bytes_count);
        pb.set_style(
            ProgressStyle::default_bar().template("[{elapsed_precise}] {bar:60.cyan} {percent}%"),
        );
    }

    pb
}

impl std::fmt::Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Format::Matroska => write!(f, "Matroska file"),
            Format::Raw => write!(f, "HEVC file"),
            Format::RawStdin => write!(f, "HEVC pipe"),
        }
    }
}

pub fn parse_rpu_file(input: &Path) -> Option<Vec<DoviRpu>> {
    println!("Parsing RPU file...");

    let rpu_file = File::open(input).unwrap();
    let metadata = rpu_file.metadata().unwrap();

    // Should never be this large, avoid mistakes
    if metadata.len() > 250_000_000 {
        panic!("Input file probably too large");
    }

    let mut reader = BufReader::new(rpu_file);

    // Should be small enough to fit in the memory
    let mut data = vec![0; metadata.len() as usize];
    reader.read_exact(&mut data).unwrap();

    let mut offsets = Vec::with_capacity(200_000);
    let mut parser = HevcParser::default();

    parser.get_offsets(&data, &mut offsets);

    let count = offsets.len();
    let last = *offsets.last().unwrap();

    let rpus: Vec<DoviRpu> = offsets
        .iter()
        .enumerate()
        .map(|(index, offset)| {
            let size = if offset == &last {
                data.len() - offset - 1
            } else {
                let size = if index == count - 1 {
                    last - offset
                } else {
                    offsets[index + 1] - offset
                };

                match &data[offset + size - 1..offset + size + 3] {
                    [0, 0, 0, 1] => size - 2,
                    _ => size,
                }
            };

            let start = *offset + 1;
            let end = start + size;

            parse_dovi_rpu(&data[start..end])
        })
        .filter_map(Result::ok)
        .collect();

    if count > 0 && rpus.len() == count {
        Some(rpus)
    } else if count == 0 {
        panic!("No RPU found");
    } else {
        panic!("Number of valid RPUs different from total");
    }
}

pub fn write_rpu_file(output_path: &Path, rpus: &mut Vec<DoviRpu>) -> Result<(), std::io::Error> {
    println!("Writing RPU file...");
    let mut writer = BufWriter::with_capacity(
        100_000,
        File::create(output_path).expect("Can't create file"),
    );

    for rpu in rpus.iter_mut() {
        let data = rpu.write_rpu_data();

        writer.write_all(OUT_NAL_HEADER)?;

        // Remove 0x7C01
        writer.write_all(&data[2..])?;
    }

    writer.flush()?;

    Ok(())
}
