use anyhow::{bail, Result};
use indicatif::ProgressBar;
use std::path::PathBuf;

use crate::commands::ExtractRpuArgs;

use super::{
    general_read_write::{self, DoviProcessorOptions},
    input_from_either, CliOptions, IoFormat,
};
use general_read_write::{DoviProcessor, DoviWriter};

pub struct RpuExtractor {
    format: IoFormat,
    input: PathBuf,
    rpu_out: PathBuf,
    limit: Option<u64>,
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
        let format = hevc_parser::io::format_from_path(&input)?;

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
        let pb = super::initialize_progress_bar(&self.format, &self.input)?;

        match self.format {
            IoFormat::Matroska => bail!("Extractor: Matroska input is unsupported"),
            _ => self.extract_rpu_from_el(pb, options),
        }
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
