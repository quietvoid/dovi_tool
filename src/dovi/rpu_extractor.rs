use std::path::PathBuf;

use super::{io, Format, RpuOptions};
use indicatif::ProgressBar;

use io::{DoviReader, DoviWriter};

pub struct RpuExtractor {
    format: Format,
    input: PathBuf,
    rpu_out: PathBuf,
}

impl RpuExtractor {
    pub fn new(format: Format, input: PathBuf, rpu_out: PathBuf) -> Self {
        Self {
            format,
            input,
            rpu_out,
        }
    }

    pub fn process_input(&self, options: RpuOptions) {
        let pb = super::initialize_progress_bar(&self.format, &self.input);

        match self.format {
            Format::Matroska => panic!("unsupported"),
            _ => self.extract_rpu_from_el(Some(&pb), options),
        };
    }

    pub fn extract_rpu_from_el(&self, pb: Option<&ProgressBar>, options: RpuOptions) {
        let mut dovi_reader = DoviReader::new(options);
        let mut dovi_writer = DoviWriter::new(None, None, Some(&self.rpu_out));

        match dovi_reader.read_write_from_io(&self.format, &self.input, pb, &mut dovi_writer) {
            Ok(_) => (),
            Err(e) => panic!("{}", e),
        }
    }
}
