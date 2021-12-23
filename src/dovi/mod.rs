pub mod converter;
pub mod demuxer;
pub mod editor;
pub mod exporter;
pub mod generator;
pub mod rpu_extractor;
pub mod rpu_info;
pub mod rpu_injector;

mod io;

#[cfg(test)]
mod tests;

use indicatif::{ProgressBar, ProgressStyle};
use std::io::{stdout, BufReader, Read, Write};
use std::{fs::File, io::BufWriter, path::Path};

use anyhow::{bail, Result};

use super::bitvec_writer::BitVecWriter;

use dolby_vision::rpu;

use super::input_format;
use hevc_parser::{
    hevc::{Frame, NAL_AUD},
    HevcParser, NALUStartCode,
};
use rpu::dovi_rpu::DoviRpu;

const OUT_NAL_HEADER: &[u8] = &[0, 0, 0, 1];
const HDR10PLUS_SEI_HEADER: &[u8] = &[78, 1, 4];

#[derive(Debug, PartialEq)]
pub enum Format {
    Raw,
    RawStdin,
    Matroska,
}

#[derive(Debug)]
pub struct CliOptions {
    pub mode: Option<u8>,
    pub crop: bool,
    pub discard_el: bool,
    pub drop_hdr10plus: bool,
}

pub fn initialize_progress_bar(format: &Format, input: &Path) -> Result<ProgressBar> {
    let pb: ProgressBar;
    let bytes_count;

    if let Format::RawStdin = format {
        pb = ProgressBar::hidden();
    } else {
        let file = File::open(input).expect("No file found");

        //Info for indicatif ProgressBar
        let file_meta = file.metadata()?;
        bytes_count = file_meta.len() / 100_000_000;

        pb = ProgressBar::new(bytes_count);
        pb.set_style(
            ProgressStyle::default_bar().template("[{elapsed_precise}] {bar:60.cyan} {percent}%"),
        );
    }

    Ok(pb)
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

pub fn parse_rpu_file(input: &Path) -> Result<Option<Vec<DoviRpu>>> {
    println!("Parsing RPU file...");
    stdout().flush().ok();

    let rpu_file = File::open(input)?;
    let metadata = rpu_file.metadata()?;

    // Should never be this large, avoid mistakes
    if metadata.len() > 250_000_000 {
        bail!("Input file probably too large");
    }

    let mut reader = BufReader::new(rpu_file);

    // Should be small enough to fit in the memory
    let mut data = vec![0; metadata.len() as usize];
    reader.read_exact(&mut data)?;

    let mut offsets = Vec::with_capacity(200_000);
    let mut parser = HevcParser::with_nalu_start_code(NALUStartCode::Length4);

    parser.get_offsets(&data, &mut offsets);

    let count = offsets.len();
    let last = *offsets.last().unwrap();
    let mut warned = false;

    let rpus: Vec<DoviRpu> = offsets
        .iter()
        .enumerate()
        .map(|(index, offset)| {
            let size = if offset == &last {
                data.len() - offset
            } else {
                offsets[index + 1] - offset
            };

            let start = *offset;
            let end = start + size;

            DoviRpu::parse_unspec62_nalu(&data[start..end])
        })
        .enumerate()
        .filter_map(|(i, res)| {
            if let Err(e) = &res {
                if !warned {
                    println!("Error parsing frame {}: {}", i, e);
                    warned = true;
                }
            }

            res.ok()
        })
        .collect();

    if count > 0 && rpus.len() == count {
        Ok(Some(rpus))
    } else if count == 0 {
        bail!("No RPU found");
    } else {
        bail!(
            "Number of valid RPUs different from total: expected {} got {}",
            count,
            rpus.len()
        );
    }
}

pub fn write_rpu_file(output_path: &Path, data: Vec<Vec<u8>>) -> Result<()> {
    println!("Writing RPU file...");
    let mut writer = BufWriter::with_capacity(
        100_000,
        File::create(output_path).expect("Can't create file"),
    );

    for encoded_rpu in data {
        writer.write_all(OUT_NAL_HEADER)?;

        // Remove 0x7C01
        writer.write_all(&encoded_rpu[2..])?;
    }

    writer.flush()?;

    Ok(())
}

pub fn _get_aud(frame: &Frame) -> Vec<u8> {
    let pic_type: u8 = match &frame.frame_type {
        2 => 0,
        1 => 1,
        0 => 2,
        _ => 7,
    };

    let mut data = OUT_NAL_HEADER.to_vec();
    let mut writer = BitVecWriter::new();

    // forbidden_zero_bit
    writer.write(false);

    writer.write_n(&(NAL_AUD).to_be_bytes(), 6);
    writer.write_n(&(0_u8).to_be_bytes(), 6);
    writer.write_n(&(0_u8).to_be_bytes(), 3);

    writer.write_n(&pic_type.to_be_bytes(), 3);

    data.extend_from_slice(writer.as_slice());

    data
}
