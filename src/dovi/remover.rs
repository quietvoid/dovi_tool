use anyhow::{Result, bail};
use indicatif::ProgressBar;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

use crate::commands::RemoveArgs;

use super::av1::{
    IvfWriter, ObuReader, ObuWriter,
    is_dovi_rpu_obu,
    try_read_ivf_file_header, read_ivf_frame_header, read_obus_from_ivf_frame,
};
use super::{CliOptions, IoFormat, general_read_write, input_from_either};

use general_read_write::{DoviProcessor, DoviWriter};

fn is_av1_input(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("av1") | Some("ivf")
    )
}

pub struct Remover {
    format: IoFormat,
    input: PathBuf,
    output: PathBuf,
}

impl Remover {
    pub fn from_args(args: RemoveArgs) -> Result<Self> {
        let RemoveArgs {
            input,
            input_pos,
            output,
        } = args;

        let input = input_from_either("remove", input, input_pos)?;

        let (format, default_output) = if is_av1_input(&input) {
            let ext = input.extension().and_then(|e| e.to_str()).unwrap_or("av1");
            (IoFormat::Raw, PathBuf::from(format!("BL_no_dovi.{ext}")))
        } else {
            (hevc_parser::io::format_from_path(&input)?, PathBuf::from("BL.hevc"))
        };

        let output = output.unwrap_or(default_output);

        Ok(Self {
            format,
            input,
            output,
        })
    }

    pub fn remove(args: RemoveArgs, options: CliOptions) -> Result<()> {
        let remover = Remover::from_args(args)?;
        remover.process_input(options)
    }

    fn process_input(&self, options: CliOptions) -> Result<()> {
        if is_av1_input(&self.input) {
            return self.remove_from_av1();
        }

        let pb = super::initialize_progress_bar(&self.format, &self.input)?;

        match self.format {
            IoFormat::Matroska => bail!("Remover: Matroska input is unsupported"),
            _ => self.remove_from_raw_hevc(pb, options),
        }
    }

    fn remove_from_av1(&self) -> Result<()> {
        println!("Removing DoVi RPU from AV1 bitstream...");

        let in_file = File::open(&self.input)?;
        let mut reader = BufReader::new(in_file);

        if let Some(ivf_header) = try_read_ivf_file_header(&mut reader)? {
            // IVF container
            let out_file = BufWriter::new(File::create(&self.output).expect("Can't create file"));
            let mut ivf_writer = IvfWriter::new(out_file, &ivf_header)?;

            while let Some(frame_hdr) = read_ivf_frame_header(&mut reader)? {
                let mut frame_data = vec![0u8; frame_hdr.frame_size as usize];
                std::io::Read::read_exact(&mut reader, &mut frame_data)?;

                let obus = read_obus_from_ivf_frame(frame_data)?;
                let mut new_frame: Vec<u8> = Vec::new();

                for obu in &obus {
                    if !is_dovi_rpu_obu(obu) {
                        new_frame.extend_from_slice(&obu.raw_bytes);
                    }
                }

                ivf_writer.write_frame(frame_hdr.timestamp, &new_frame)?;
            }

            ivf_writer.flush()?;
        } else {
            // Raw AV1 bitstream
            let out_file = BufWriter::new(File::create(&self.output).expect("Can't create file"));
            let mut obu_writer = ObuWriter::new(out_file);
            let mut obu_reader = ObuReader::new(reader);

            while let Some(obu) = obu_reader.next_obu()? {
                if !is_dovi_rpu_obu(&obu) {
                    obu_writer.write_raw(&obu.raw_bytes)?;
                }
            }

            obu_writer.flush()?;
        }

        println!("Done.");
        Ok(())
    }

    fn remove_from_raw_hevc(&self, pb: ProgressBar, options: CliOptions) -> Result<()> {
        let bl_out = Some(self.output.as_path());

        let dovi_writer = DoviWriter::new(bl_out, None, None, None);
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
