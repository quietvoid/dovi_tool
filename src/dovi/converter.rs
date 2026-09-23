use anyhow::Result;
use indicatif::ProgressBar;
use std::io::BufRead;
use std::path::PathBuf;

use crate::commands::ConvertArgs;

use super::av1::{self, RpuAction};
use super::{CliOptions, IoFormat, general_read_write, input_from_either};

use general_read_write::{DoviProcessor, DoviWriter};

pub struct Converter {
    format: IoFormat,
    input: PathBuf,
    output: PathBuf,
}

impl Converter {
    pub fn from_args(args: ConvertArgs, options: &mut CliOptions) -> Result<Self> {
        let ConvertArgs {
            input,
            input_pos,
            output,
            discard,
        } = args;

        options.discard_el = discard;

        let input = input_from_either("convert", input, input_pos)?;
        let format = hevc_parser::io::format_from_path(&input)?;

        let output = match output {
            Some(path) => path,
            None => match options.discard_el {
                true => PathBuf::from("BL_RPU.hevc"),
                false => PathBuf::from("BL_EL_RPU.hevc"),
            },
        };

        Ok(Self {
            format,
            input,
            output,
        })
    }

    pub fn convert(args: ConvertArgs, mut options: CliOptions) -> Result<()> {
        let input = input_from_either("convert", args.input.clone(), args.input_pos.clone())?;

        match av1::Input::open(&input, None)? {
            av1::Input::Av1(mut av1) => {
                // An AV1 stream has no enhancement layer to keep or discard
                let output = args.output.unwrap_or_else(|| av1.default_output("BL_RPU"));
                av1.rewrite(&output, RpuAction::Convert, options)
            }
            av1::Input::Other(stdin) => {
                let converter = Converter::from_args(args, &mut options)?;
                converter.process_input(options, stdin)
            }
        }
    }

    fn process_input(&self, options: CliOptions, stdin: Option<Box<dyn BufRead>>) -> Result<()> {
        let pb = super::initialize_progress_bar(&self.format, &self.input)?;

        if self.format == IoFormat::Matroska {
            println!("Converter: Matroska input is experimental!");
        }

        self.convert_hevc(pb, options, stdin)
    }

    fn convert_hevc(
        &self,
        pb: ProgressBar,
        options: CliOptions,
        stdin: Option<Box<dyn BufRead>>,
    ) -> Result<()> {
        let dovi_writer = DoviWriter::new(None, None, None, Some(&self.output));
        let mut dovi_processor = DoviProcessor::new(
            options,
            self.input.clone(),
            dovi_writer,
            pb,
            Default::default(),
        )
        .with_stdin(stdin);

        dovi_processor.read_write_from_io(&self.format)
    }
}
