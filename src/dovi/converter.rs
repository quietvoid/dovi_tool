use std::path::PathBuf;

use indicatif::ProgressBar;

use super::{input_format, io, Format, RpuOptions};

use io::{DoviReader, DoviWriter};

pub struct Converter {
    format: Format,
    input: PathBuf,
    output: PathBuf,
}

impl Converter {
    pub fn new(format: Format, input: PathBuf, output: PathBuf) -> Self {
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
        options: RpuOptions,
    ) {
        let input = match input {
            Some(input) => input,
            None => match stdin {
                Some(stdin) => stdin,
                None => PathBuf::new(),
            },
        };

        match input_format(&input) {
            Ok(format) => {
                let output = match output {
                    Some(path) => path,
                    None => PathBuf::from("BL_EL.hevc"),
                };

                let demuxer = Converter::new(format, input, output);
                demuxer.process_input(options);
            }
            Err(msg) => println!("{}", msg),
        }
    }

    fn process_input(&self, options: RpuOptions) {
        let pb = super::initialize_progress_bar(&self.format, &self.input);

        match self.format {
            Format::Matroska => panic!("unsupported"),
            _ => self.convert_raw_hevc(Some(&pb), options),
        };
    }

    fn convert_raw_hevc(&self, pb: Option<&ProgressBar>, options: RpuOptions) {
        let mut dovi_reader = DoviReader::new(options);
        let mut dovi_writer = DoviWriter::new(None, None, None, Some(&self.output));

        match dovi_reader.read_write_from_io(&self.format, &self.input, pb, &mut dovi_writer) {
            Ok(_) => (),
            Err(e) => panic!("{}", e),
        }
    }
}
