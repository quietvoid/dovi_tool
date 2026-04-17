use anyhow::{Result, bail};
use indicatif::ProgressBar;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

use crate::commands::ConvertArgs;

use super::av1::{
    IvfWriter, ObuReader, ObuWriter,
    build_dovi_obu, is_dovi_rpu_obu, extract_dovi_t35_payload,
    try_read_ivf_file_header, read_ivf_frame_header, read_obus_from_ivf_frame,
};
use super::{CliOptions, IoFormat, general_read_write, input_from_either};
use dolby_vision::rpu::dovi_rpu::DoviRpu;

use general_read_write::{DoviProcessor, DoviWriter};

fn is_av1_input(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("av1") | Some("ivf")
    )
}

pub struct Converter {
    format: IoFormat,
    input: PathBuf,
    output: PathBuf,
}

impl Converter {
    pub fn from_args(args: ConvertArgs, options: &mut CliOptions) -> Result<Self> {
        let ConvertArgs {
            input,
            input_pos,
            output,
            discard,
        } = args;

        options.discard_el = discard;

        let input = input_from_either("convert", input, input_pos)?;

        let (format, default_output) = if is_av1_input(&input) {
            let ext = input.extension().and_then(|e| e.to_str()).unwrap_or("av1");
            (IoFormat::Raw, PathBuf::from(format!("converted.{ext}")))
        } else {
            let format = hevc_parser::io::format_from_path(&input)?;
            let default = match options.discard_el {
                true => PathBuf::from("BL_RPU.hevc"),
                false => PathBuf::from("BL_EL_RPU.hevc"),
            };
            (format, default)
        };

        let output = output.unwrap_or(default_output);

        Ok(Self {
            format,
            input,
            output,
        })
    }

    pub fn convert(args: ConvertArgs, mut options: CliOptions) -> Result<()> {
        let converter = Converter::from_args(args, &mut options)?;
        converter.process_input(options)
    }

    fn process_input(&self, options: CliOptions) -> Result<()> {
        if is_av1_input(&self.input) {
            return self.convert_av1(&options);
        }

        let pb = super::initialize_progress_bar(&self.format, &self.input)?;

        match self.format {
            IoFormat::Matroska => bail!("Converter: Matroska input is unsupported"),
            _ => self.convert_raw_hevc(pb, options),
        }
    }

    fn convert_av1(&self, options: &CliOptions) -> Result<()> {
        println!("Converting DoVi RPU in AV1 bitstream...");

        let in_file = File::open(&self.input)?;
        let mut reader = BufReader::new(in_file);

        if let Some(ivf_header) = try_read_ivf_file_header(&mut reader)? {
            let out_file = BufWriter::new(File::create(&self.output).expect("Can't create file"));
            let mut ivf_writer = IvfWriter::new(out_file, &ivf_header)?;

            while let Some(frame_hdr) = read_ivf_frame_header(&mut reader)? {
                let mut frame_data = vec![0u8; frame_hdr.frame_size as usize];
                std::io::Read::read_exact(&mut reader, &mut frame_data)?;

                let obus = read_obus_from_ivf_frame(frame_data)?;
                let mut new_frame: Vec<u8> = Vec::new();

                for obu in &obus {
                    if is_dovi_rpu_obu(obu) {
                        if let Some(t35_payload) = extract_dovi_t35_payload(&obu.payload) {
                            let mut dovi_rpu =
                                DoviRpu::parse_itu_t35_dovi_metadata_obu(t35_payload)?;
                            super::convert_encoded_from_opts_rpu(options, &mut dovi_rpu)?;
                            let converted_bytes = build_dovi_obu(&dovi_rpu)?;
                            new_frame.extend_from_slice(&converted_bytes);
                        } else {
                            new_frame.extend_from_slice(&obu.raw_bytes);
                        }
                    } else {
                        new_frame.extend_from_slice(&obu.raw_bytes);
                    }
                }

                ivf_writer.write_frame(frame_hdr.timestamp, &new_frame)?;
            }

            ivf_writer.flush()?;
        } else {
            let out_file = BufWriter::new(File::create(&self.output).expect("Can't create file"));
            let mut obu_writer = ObuWriter::new(out_file);
            let mut obu_reader = ObuReader::new(reader);

            while let Some(obu) = obu_reader.next_obu()? {
                if is_dovi_rpu_obu(&obu) {
                    if let Some(t35_payload) = extract_dovi_t35_payload(&obu.payload) {
                        let mut dovi_rpu = DoviRpu::parse_itu_t35_dovi_metadata_obu(t35_payload)?;
                        super::convert_encoded_from_opts_rpu(options, &mut dovi_rpu)?;
                        let converted_bytes = build_dovi_obu(&dovi_rpu)?;
                        obu_writer.write_raw(&converted_bytes)?;
                    } else {
                        obu_writer.write_raw(&obu.raw_bytes)?;
                    }
                } else {
                    obu_writer.write_raw(&obu.raw_bytes)?;
                }
            }

            obu_writer.flush()?;
        }

        println!("Done.");
        Ok(())
    }

    fn convert_raw_hevc(&self, pb: ProgressBar, options: CliOptions) -> Result<()> {
        let dovi_writer = DoviWriter::new(None, None, None, Some(&self.output));
        let mut dovi_processor = DoviProcessor::new(
            options,
            self.input.clone(),
            dovi_writer,
            pb,
            Default::default(),
        );

        dovi_processor.read_write_from_io(&self.format)
    }
}
