use anyhow::{bail, Result};
use indicatif::ProgressBar;
use std::path::PathBuf;

use super::{general_read_write, CliOptions, IoFormat};

use general_read_write::{DoviReader, DoviWriter};

pub struct Demuxer {
    format: IoFormat,
    input: PathBuf,
    bl_out: PathBuf,
    el_out: PathBuf,
    el_only: bool,
}

impl Demuxer {
    pub fn new(
        format: IoFormat,
        input: PathBuf,
        bl_out: PathBuf,
        el_out: PathBuf,
        el_only: bool,
    ) -> Self {
        Self {
            format,
            input,
            bl_out,
            el_out,
            el_only,
        }
    }

    pub fn demux(
        input: Option<PathBuf>,
        stdin: Option<PathBuf>,
        bl_out: Option<PathBuf>,
        el_out: Option<PathBuf>,
        el_only: bool,
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

        let bl_out = match bl_out {
            Some(path) => path,
            None => PathBuf::from("BL.hevc"),
        };

        let el_out = match el_out {
            Some(path) => path,
            None => PathBuf::from("EL.hevc"),
        };

        let demuxer = Demuxer::new(format, input, bl_out, el_out, el_only);
        demuxer.process_input(options)
    }

    fn process_input(&self, options: CliOptions) -> Result<()> {
        let pb = super::initialize_progress_bar(&self.format, &self.input)?;

        match self.format {
            IoFormat::Matroska => bail!("Demuxer: Matroska input is unsupported"),
            _ => self.demux_raw_hevc(Some(&pb), options),
        }
    }

    fn demux_raw_hevc(&self, pb: Option<&ProgressBar>, options: CliOptions) -> Result<()> {
        let mut dovi_reader = DoviReader::new(options);

        let bl_out = if self.el_only {
            None
        } else {
            Some(self.bl_out.as_path())
        };

        let mut dovi_writer = DoviWriter::new(bl_out, Some(self.el_out.as_path()), None, None);

        dovi_reader.read_write_from_io(&self.format, &self.input, pb, &mut dovi_writer)
    }
}
