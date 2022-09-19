use std::io::Write;
use std::path::PathBuf;
use std::{fs::File, io::BufWriter, path::Path};

use anyhow::{bail, Result};
use indicatif::{ProgressBar, ProgressStyle};

use dolby_vision::rpu::dovi_rpu::DoviRpu;

use hevc_parser::hevc::{NALUnit, NAL_UNSPEC62};
use hevc_parser::io::{IoFormat, StartCodePreset};

use self::editor::EditConfig;
use super::commands::ConversionModeCli;

pub mod converter;
pub mod demuxer;
pub mod editor;
pub mod exporter;
pub mod generator;
pub mod muxer;
pub mod rpu_extractor;
pub mod rpu_info;
pub mod rpu_injector;

mod general_read_write;
mod hdr10plus_utils;

#[derive(Debug, Clone)]
pub struct CliOptions {
    pub mode: Option<ConversionModeCli>,
    pub crop: bool,
    pub discard_el: bool,
    pub drop_hdr10plus: bool,
    pub edit_config: Option<EditConfig>,
    pub start_code: WriteStartCodePreset,
}

#[derive(clap::ArgEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteStartCodePreset {
    Four,
    AnnexB,
}

pub fn initialize_progress_bar<P: AsRef<Path>>(format: &IoFormat, input: P) -> Result<ProgressBar> {
    let pb: ProgressBar;
    let bytes_count;

    if let IoFormat::RawStdin = format {
        pb = ProgressBar::hidden();
    } else {
        let file = File::open(input).expect("No file found");

        //Info for indicatif ProgressBar
        let file_meta = file.metadata()?;
        bytes_count = file_meta.len() / 100_000_000;

        pb = ProgressBar::new(bytes_count);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:60.cyan} {percent}%")?,
        );
    }

    Ok(pb)
}

pub fn write_rpu_file<P: AsRef<Path>>(output_path: P, data: Vec<Vec<u8>>) -> Result<()> {
    println!("Writing RPU file...");
    let mut writer = BufWriter::with_capacity(
        100_000,
        File::create(output_path).expect("Can't create file"),
    );

    for encoded_rpu in data {
        // Remove 0x7C01
        NALUnit::write_with_preset(
            &mut writer,
            &encoded_rpu[2..],
            WriteStartCodePreset::Four.into(),
            NAL_UNSPEC62,
            true,
        )?;
    }

    writer.flush()?;

    Ok(())
}

pub fn convert_encoded_from_opts(opts: &CliOptions, data: &[u8]) -> Result<Vec<u8>> {
    let mut dovi_rpu = DoviRpu::parse_unspec62_nalu(data)?;

    // Config overrides manual arguments
    if let Some(edit_config) = &opts.edit_config {
        edit_config.execute_single_rpu(&mut dovi_rpu)?;
    } else {
        if let Some(mode) = opts.mode {
            dovi_rpu.convert_with_mode(mode)?;
        }

        if opts.crop {
            dovi_rpu.crop()?;
        }
    }

    dovi_rpu.write_hevc_unspec62_nalu()
}

pub fn input_from_either(cmd: &str, in1: Option<PathBuf>, in2: Option<PathBuf>) -> Result<PathBuf> {
    match in1 {
        Some(in1) => Ok(in1),
        None => match in2 {
            Some(in2) => Ok(in2),
            None => bail!("No input file provided. See `dovi_tool {} --help`", cmd),
        },
    }
}

impl From<WriteStartCodePreset> for StartCodePreset {
    fn from(p: WriteStartCodePreset) -> Self {
        match p {
            WriteStartCodePreset::Four => StartCodePreset::Four,
            WriteStartCodePreset::AnnexB => StartCodePreset::AnnexB,
        }
    }
}
