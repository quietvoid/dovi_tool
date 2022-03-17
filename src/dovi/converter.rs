use anyhow::{bail, Result};
use indicatif::ProgressBar;
use std::path::PathBuf;

use super::{general_read_write, CliOptions, IoFormat};

use general_read_write::{DoviReader, DoviWriter};

pub struct Converter {
    format: IoFormat,
    input: PathBuf,
    output: PathBuf,
}

impl Converter {
    pub fn new(format: IoFormat, input: PathBuf, output: PathBuf) -> Self {
        Self {
            format,
            input,
            output,
        }
    }

    pub fn convert(
        input: Option<PathBuf>,
        stdin: Option<PathBuf>,
        output: Option<PathBuf>,
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

        let output = match output {
            Some(path) => path,
            None => match options.discard_el {
                true => PathBuf::from("BL_RPU.hevc"),
                false => PathBuf::from("BL_EL_RPU.hevc"),
            },
        };

        let demuxer = Converter::new(format, input, output);
        demuxer.process_input(options)
    }

    fn process_input(&self, options: CliOptions) -> Result<()> {
        let pb = super::initialize_progress_bar(&self.format, &self.input)?;

        match self.format {
            IoFormat::Matroska => bail!("Converter: Matroska input is unsupported"),
            _ => self.convert_raw_hevc(Some(&pb), options),
        }
    }

    fn convert_raw_hevc(&self, pb: Option<&ProgressBar>, options: CliOptions) -> Result<()> {
        let mut dovi_reader = DoviReader::new(options);
        let mut dovi_writer = DoviWriter::new(None, None, None, Some(&self.output));

        dovi_reader.read_write_from_io(&self.format, &self.input, pb, &mut dovi_writer)
    }
}
