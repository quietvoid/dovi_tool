use anyhow::{bail, Result};
use indicatif::ProgressBar;
use std::path::PathBuf;

use crate::commands::RemoveArgs;

use super::{general_read_write, input_from_either, CliOptions, IoFormat};

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
        let remover = Remover::from_args(args)?;
        remover.process_input(options)
    }

    fn process_input(&self, options: CliOptions) -> Result<()> {
        let pb = super::initialize_progress_bar(&self.format, &self.input)?;

        match self.format {
            IoFormat::Matroska => bail!("Remover: Matroska input is unsupported"),
            _ => self.remove_from_raw_hevc(pb, options),
        }
    }

    fn remove_from_raw_hevc(&self, pb: ProgressBar, options: CliOptions) -> Result<()> {
        let bl_out = Some(self.output.as_path());

        let dovi_writer = DoviWriter::new(bl_out, None, None, None);
        let mut dovi_processor = DoviProcessor::new(
            options,
            self.input.clone(),
            dovi_writer,
            pb,
            Default::default(),
        );

        dovi_processor.read_write_from_io(&self.format)
    }
}
