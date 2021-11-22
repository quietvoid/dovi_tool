use anyhow::{bail, Result};
use indicatif::ProgressBar;
use std::path::PathBuf;

use super::{input_format, io, CliOptions, Format};

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

    pub fn demux(
        input: Option<PathBuf>,
        stdin: Option<PathBuf>,
        bl_out: Option<PathBuf>,
        el_out: Option<PathBuf>,
        options: CliOptions,
    ) -> Result<()> {
        let input = match input {
            Some(input) => input,
            None => match stdin {
                Some(stdin) => stdin,
                None => PathBuf::new(),
            },
        };

        let format = input_format(&input)?;

        let bl_out = match bl_out {
            Some(path) => path,
            None => PathBuf::from("BL.hevc"),
        };

        let el_out = match el_out {
            Some(path) => path,
            None => PathBuf::from("EL.hevc"),
        };

        let demuxer = Demuxer::new(format, input, bl_out, el_out);
        demuxer.process_input(options)
    }

    fn process_input(&self, options: CliOptions) -> Result<()> {
        let pb = super::initialize_progress_bar(&self.format, &self.input)?;

        match self.format {
            Format::Matroska => bail!("unsupported"),
            _ => self.demux_raw_hevc(Some(&pb), options),
        }
    }

    fn demux_raw_hevc(&self, pb: Option<&ProgressBar>, options: CliOptions) -> Result<()> {
        let mut dovi_reader = DoviReader::new(options);
        let mut dovi_writer = DoviWriter::new(Some(&self.bl_out), Some(&self.el_out), None, None);

        dovi_reader.read_write_from_io(&self.format, &self.input, pb, &mut dovi_writer)
    }
}
