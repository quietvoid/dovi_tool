use anyhow::Result;
use indicatif::ProgressBar;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use crate::commands::ExtractRpuArgs;

use super::{
    CliOptions, IoFormat,
    general_read_write::{self, DoviProcessorOptions},
    input_from_either,
};
use general_read_write::{DoviProcessor, DoviWriter};

use super::av1::{
    OBU_METADATA, ObuReader,
    is_dovi_rpu_obu, extract_dovi_t35_payload,
    try_read_ivf_file_header, read_ivf_frame_header, read_obus_from_ivf_frame,
};
use dolby_vision::rpu::dovi_rpu::DoviRpu;
use hevc_parser::hevc::{NAL_UNSPEC62, NALUnit};
use hevc_parser::io::StartCodePreset;

pub struct RpuExtractor {
    format: IoFormat,
    input: PathBuf,
    rpu_out: PathBuf,
    limit: Option<u64>,
}

fn is_av1_input(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("av1") | Some("ivf")
    )
}

impl RpuExtractor {
    pub fn from_args(args: ExtractRpuArgs) -> Result<Self> {
        let ExtractRpuArgs {
            input,
            input_pos,
            rpu_out,
            limit,
        } = args;

        let input = input_from_either("extract-rpu", input, input_pos)?;

        // For AV1 inputs use a dummy format; for HEVC use the existing detection
        let format = if is_av1_input(&input) {
            IoFormat::Raw
        } else {
            hevc_parser::io::format_from_path(&input)?
        };

        let rpu_out = match rpu_out {
            Some(path) => path,
            None => PathBuf::from("RPU.bin"),
        };

        Ok(Self {
            format,
            input,
            rpu_out,
            limit,
        })
    }

    pub fn extract_rpu(args: ExtractRpuArgs, options: CliOptions) -> Result<()> {
        let rpu_extractor = RpuExtractor::from_args(args)?;
        rpu_extractor.process_input(options)
    }

    fn process_input(&self, options: CliOptions) -> Result<()> {
        if is_av1_input(&self.input) {
            self.extract_rpu_from_av1()
        } else {
            let pb = super::initialize_progress_bar(&self.format, &self.input)?;
            self.extract_rpu_from_el(pb, options)
        }
    }

    fn extract_rpu_from_av1(&self) -> Result<()> {
        println!("Extracting RPU from AV1 bitstream...");

        let file = File::open(&self.input)?;
        let mut reader = BufReader::new(file);

        let mut rpus: Vec<Vec<u8>> = Vec::new();
        let mut frame_count: u64 = 0;

        // Detect IVF container by peeking at first bytes
        if let Some(_ivf_header) = try_read_ivf_file_header(&mut reader)? {
            // IVF container: iterate over IVF frames
            while let Some(frame_hdr) = read_ivf_frame_header(&mut reader)? {
                if let Some(limit) = self.limit {
                    if frame_count >= limit {
                        break;
                    }
                }

                let mut frame_data = vec![0u8; frame_hdr.frame_size as usize];
                std::io::Read::read_exact(&mut reader, &mut frame_data)?;

                let obus = read_obus_from_ivf_frame(frame_data)?;
                for obu in &obus {
                    if is_dovi_rpu_obu(obu) {
                        if let Some(t35_payload) = extract_dovi_t35_payload(&obu.payload) {
                            let rpu = DoviRpu::parse_itu_t35_dovi_metadata_obu(t35_payload)?;
                            rpus.push(rpu.write_hevc_unspec62_nalu()?);
                        }
                    }
                }

                frame_count += 1;
            }
        } else {
            // Raw AV1 bitstream
            let mut obu_reader = ObuReader::new(reader);
            while let Some(obu) = obu_reader.next_obu()? {
                if let Some(limit) = self.limit {
                    if frame_count >= limit {
                        break;
                    }
                }

                if obu.obu_type == OBU_METADATA && is_dovi_rpu_obu(&obu) {
                    if let Some(t35_payload) = extract_dovi_t35_payload(&obu.payload) {
                        let rpu = DoviRpu::parse_itu_t35_dovi_metadata_obu(t35_payload)?;
                        rpus.push(rpu.write_hevc_unspec62_nalu()?);
                    }
                    frame_count += 1;
                }
            }
        }

        println!("Found {} RPU(s).", rpus.len());
        self.write_av1_rpu_file(&rpus)
    }

    fn write_av1_rpu_file(&self, rpus: &[Vec<u8>]) -> Result<()> {
        println!("Writing RPU file...");
        let mut writer = BufWriter::with_capacity(
            100_000,
            File::create(&self.rpu_out).expect("Can't create file"),
        );

        for encoded_rpu in rpus {
            // encoded_rpu is write_hevc_unspec62_nalu() output: starts with 0x7C 0x01
            // Same format as HEVC path: [00 00 00 01] + rpu[2..]
            NALUnit::write_with_preset(
                &mut writer,
                &encoded_rpu[2..],
                StartCodePreset::Four,
                NAL_UNSPEC62,
                true,
            )?;
        }

        writer.flush()?;
        Ok(())
    }

    fn extract_rpu_from_el(&self, pb: ProgressBar, options: CliOptions) -> Result<()> {
        let dovi_writer = DoviWriter::new(None, None, Some(&self.rpu_out), None);
        let mut dovi_processor = DoviProcessor::new(
            options,
            self.input.clone(),
            dovi_writer,
            pb,
            DoviProcessorOptions { limit: self.limit },
        );

        dovi_processor.read_write_from_io(&self.format)
    }
}
