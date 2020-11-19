use std::path::PathBuf;

use super::{io, Format};
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

    pub fn process_input(&self) {
        let pb = super::initialize_progress_bar(&self.format, &self.input);

        match self.format {
            Format::Matroska => panic!("unsupported"),
            _ => self.extract_rpu_from_el(Some(&pb)),
        };
    }

    pub fn extract_rpu_from_el(&self, pb: Option<&ProgressBar>) {
        let dovi_reader = DoviReader::new();
        let mut dovi_writer = DoviWriter::new(None, None, Some(&self.rpu_out));

        dovi_reader.read_write_from_io(&self.format, &self.input, pb, &mut dovi_writer);
    }
}
