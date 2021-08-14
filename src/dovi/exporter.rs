use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

use serde::ser::SerializeSeq;
use serde::Serializer;

use crate::dovi::parse_rpu_file;

use super::DoviRpu;

pub struct Exporter {
    input: PathBuf,
    output: PathBuf,
    rpus: Option<Vec<DoviRpu>>,
}

impl Exporter {
    pub fn export(input: PathBuf, output: Option<PathBuf>) {
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

        exporter.rpus = parse_rpu_file(&exporter.input);

        if let Err(res) = exporter.execute() {
            panic!("{:?}", res);
        }

        println!("Done.")
    }

    fn execute(&self) -> Result<(), std::io::Error> {
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
