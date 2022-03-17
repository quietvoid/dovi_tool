use anyhow::{bail, Result};
use indicatif::ProgressBar;
use std::path::PathBuf;

use super::{general_read_write, CliOptions, IoFormat};
use general_read_write::{DoviReader, DoviWriter};

pub struct RpuExtractor {
    format: IoFormat,
    input: PathBuf,
    rpu_out: PathBuf,
}

impl RpuExtractor {
    pub fn new(format: IoFormat, input: PathBuf, rpu_out: PathBuf) -> Self {
        Self {
            format,
            input,
            rpu_out,
        }
    }

    pub fn extract_rpu(
        input: Option<PathBuf>,
        stdin: Option<PathBuf>,
        rpu_out: Option<PathBuf>,
        options: CliOptions,
    ) -> Result<()> {
        let input = match input {
            Some(input) => input,
            None => match stdin {
                Some(stdin) => stdin,
                None => PathBuf::new(),
            },
        };

        let format = hevc_parser::io::format_from_path(&input)?;

        let rpu_out = match rpu_out {
            Some(path) => path,
            None => PathBuf::from("RPU.bin"),
        };

        let parser = RpuExtractor::new(format, input, rpu_out);
        parser.process_input(options)
    }

    fn process_input(&self, options: CliOptions) -> Result<()> {
        let pb = super::initialize_progress_bar(&self.format, &self.input)?;

        match self.format {
            IoFormat::Matroska => bail!("Extractor: Matroska input is unsupported"),
            _ => self.extract_rpu_from_el(Some(&pb), options),
        }
    }

    fn extract_rpu_from_el(&self, pb: Option<&ProgressBar>, options: CliOptions) -> Result<()> {
        let mut dovi_reader = DoviReader::new(options);
        let mut dovi_writer = DoviWriter::new(None, None, Some(&self.rpu_out), None);

        dovi_reader.read_write_from_io(&self.format, &self.input, pb, &mut dovi_writer)
    }
}
