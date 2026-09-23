use anyhow::Result;
use indicatif::ProgressBar;
use std::io::BufRead;
use std::path::PathBuf;

use crate::commands::RemoveArgs;

use super::av1::{self, RpuAction};
use super::{CliOptions, IoFormat, general_read_write, input_from_either};

use general_read_write::{DoviProcessor, DoviWriter};

pub struct Remover {
    format: IoFormat,
    input: PathBuf,
    output: PathBuf,
}

impl Remover {
    pub fn from_args(args: RemoveArgs) -> Result<Self> {
        let RemoveArgs {
            input,
            input_pos,
            output,
        } = args;

        let input = input_from_either("remove", input, input_pos)?;
        let format = hevc_parser::io::format_from_path(&input)?;

        let output = output.unwrap_or(PathBuf::from("BL.hevc"));

        Ok(Self {
            format,
            input,
            output,
        })
    }

    pub fn remove(args: RemoveArgs, options: CliOptions) -> Result<()> {
        let input = input_from_either("remove", args.input.clone(), args.input_pos.clone())?;

        match av1::Input::open(&input, None)? {
            av1::Input::Av1(mut av1) => {
                let output = args.output.unwrap_or_else(|| av1.default_output("BL"));
                av1.rewrite(&output, RpuAction::Remove, options)
            }
            av1::Input::Other(stdin) => {
                let remover = Remover::from_args(args)?;
                remover.process_input(options, stdin)
            }
        }
    }

    fn process_input(&self, options: CliOptions, stdin: Option<Box<dyn BufRead>>) -> Result<()> {
        let pb = super::initialize_progress_bar(&self.format, &self.input)?;

        if self.format == IoFormat::Matroska {
            println!("Remover: Matroska input is experimental!");
        }

        self.remove_from_hevc(pb, options, stdin)
    }

    fn remove_from_hevc(
        &self,
        pb: ProgressBar,
        options: CliOptions,
        stdin: Option<Box<dyn BufRead>>,
    ) -> Result<()> {
        let bl_out = Some(self.output.as_path());

        let dovi_writer = DoviWriter::new(bl_out, None, None, None);
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
