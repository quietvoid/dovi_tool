use std::fs::File;
use std::io::{stdout, BufWriter, Write};
use std::path::PathBuf;

use anyhow::Result;
use serde::ser::SerializeSeq;
use serde::Serializer;

use utilities_dovi::parse_rpu_file;

use crate::commands::ExportArgs;
use crate::dovi::input_from_either;

use super::DoviRpu;

pub struct Exporter {
    input: PathBuf,
    output: PathBuf,
    rpus: Option<Vec<DoviRpu>>,
}

impl Exporter {
    pub fn export(args: ExportArgs) -> Result<()> {
        let ExportArgs {
            input,
            input_pos,
            output,
        } = args;

        let input = input_from_either("editor", input, input_pos)?;

        let out_path = if let Some(out_path) = output {
            out_path
        } else {
            PathBuf::from("RPU_export.json".to_string())
        };

        let mut exporter = Exporter {
            input,
            output: out_path,
            rpus: None,
        };

        println!("Parsing RPU file...");
        stdout().flush().ok();

        exporter.rpus = parse_rpu_file(&exporter.input)?;
        exporter.execute()?;

        println!("Done.");

        Ok(())
    }

    fn execute(&self) -> Result<()> {
        println!("Exporting metadata...");

        if let Some(rpus) = &self.rpus {
            let writer = BufWriter::with_capacity(
                100_000,
                File::create(&self.output).expect("Can't create file"),
            );

            let mut ser = serde_json::Serializer::new(writer);
            let mut seq = ser.serialize_seq(Some(rpus.len()))?;

            for rpu in rpus {
                seq.serialize_element(&rpu)?;
            }
            seq.end()?;
        }

        Ok(())
    }
}
