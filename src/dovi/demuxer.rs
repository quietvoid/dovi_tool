use std::path::PathBuf;

use indicatif::ProgressBar;

use super::{io, Format};

use io::{DoviReader, DoviWriter};

pub struct Demuxer {
    format: Format,
    input: PathBuf,
    bl_out: PathBuf,
    el_out: PathBuf,
}

impl Demuxer {
    pub fn new(format: Format, input: PathBuf, bl_out: PathBuf, el_out: PathBuf) -> Self {
        Self {
            format,
            input,
            bl_out,
            el_out,
        }
    }

    pub fn process_input(&self, mode: Option<u8>) {
        let pb = super::initialize_progress_bar(&self.format, &self.input);

        match self.format {
            Format::Matroska => panic!("unsupported"),
            _ => self.demux_raw_hevc(Some(&pb), mode),
        };

        pb.finish_and_clear();
    }

    pub fn demux_raw_hevc(&self, pb: Option<&ProgressBar>, mode: Option<u8>) {
        let mut dovi_reader = DoviReader::new(mode);
        let mut dovi_writer = DoviWriter::new(Some(&self.bl_out), Some(&self.el_out), None);

        match dovi_reader.read_write_from_io(&self.format, &self.input, pb, &mut dovi_writer) {
            Ok(_) => (),
            Err(e) => panic!("{}", e),
        }
    }
}
