pub mod demuxer;
mod io;
pub mod rpu_extractor;

mod rpu;

use indicatif::{ProgressBar, ProgressStyle};
use std::fs::File;
use std::path::PathBuf;

use super::bitvec_reader::BitVecReader;
use super::bitvec_writer::BitVecWriter;
#[derive(Debug, PartialEq)]
pub enum Format {
    Raw,
    RawStdin,
    Matroska,
}

pub fn clear_start_code_emulation_prevention_3_byte(data: &[u8]) -> Vec<u8> {
    data.iter()
        .enumerate()
        .filter_map(|(index, value)| {
            if index > 2
                && index < data.len() - 2
                && data[index - 2] == 0
                && data[index - 1] == 0
                && data[index] <= 3
            {
                None
            } else {
                Some(*value)
            }
        })
        .collect::<Vec<u8>>()
}

pub fn add_start_code_emulation_prevention_3_byte(data: &mut Vec<u8>) {
    let mut count = data.len();
    let mut i = 0;

    while i < count {
        if i > 2 && i < count - 2 && data[i - 2] == 0 && data[i - 1] == 0 && data[i] <= 3 {
            data.insert(i, 3);
            count += 1;
        }

        i += 1;
    }
}

pub fn initialize_progress_bar(format: &Format, input: &PathBuf) -> ProgressBar {
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
