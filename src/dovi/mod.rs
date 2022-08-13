use std::convert::TryInto;
use std::io::Write;
use std::path::PathBuf;
use std::{fs::File, io::BufWriter, path::Path};

use anyhow::{bail, Result};
use indicatif::{ProgressBar, ProgressStyle};

use dolby_vision::rpu::dovi_rpu::DoviRpu;

use hevc_parser::hevc::{NALUnit, SeiMessage, NAL_UNSPEC62, USER_DATA_REGISTERED_ITU_T_35};
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
            ProgressStyle::default_bar().template("[{elapsed_precise}] {bar:60.cyan} {percent}%"),
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

pub fn is_st2094_40_sei(sei_payload: &[u8]) -> Result<bool> {
    if sei_payload.len() >= 4 {
        let sei = SeiMessage::from_bytes(sei_payload)?;

        if sei.payload_type == USER_DATA_REGISTERED_ITU_T_35 {
            // FIXME: Not sure why 4 bytes..
            let itu_t35_bytes = &sei_payload[4..];

            if itu_t35_bytes.len() >= 7 {
                let itu_t_t35_country_code = itu_t35_bytes[0];
                let itu_t_t35_terminal_provider_code =
                    u16::from_be_bytes(itu_t35_bytes[1..3].try_into()?);
                let itu_t_t35_terminal_provider_oriented_code =
                    u16::from_be_bytes(itu_t35_bytes[3..5].try_into()?);

                if itu_t_t35_country_code == 0xB5
                    && itu_t_t35_terminal_provider_code == 0x003C
                    && itu_t_t35_terminal_provider_oriented_code == 0x0001
                {
                    let application_identifier = itu_t35_bytes[5];
                    let application_version = itu_t35_bytes[6];

                    if application_identifier == 4 && application_version == 1 {
                        return Ok(true);
                    }
                }
            }
        }
    }

    Ok(false)
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
