use std::io::{stdout, Write};
use std::path::PathBuf;

use anyhow::{bail, ensure, Result};

use dolby_vision::rpu::dovi_rpu::DoviRpu;
use utilities_dovi::parse_rpu_file;

use super::input_from_either;
use crate::commands::InfoArgs;

pub struct RpuInfo {
    input: PathBuf,
    frame: Option<usize>,
    rpus: Option<Vec<DoviRpu>>,
}

impl RpuInfo {
    pub fn info(args: InfoArgs) -> Result<()> {
        let InfoArgs {
            input,
            input_pos,
            frame,
        } = args;

        if frame.is_none() {
            bail!("No frame number to look up");
        }

        let input = input_from_either("info", input, input_pos)?;

        let mut info = RpuInfo {
            input,
            frame,
            rpus: None,
        };

        println!("Parsing RPU file...");
        stdout().flush().ok();

        info.rpus = parse_rpu_file(&info.input)?;

        if let Some(ref rpus) = info.rpus {
            let f = info.frame.unwrap();
            ensure!(
                f < rpus.len(),
                format!(
                    "info: invalid frame number (out of range).\nNumber of valid RPUs parsed: {}",
                    rpus.len()
                )
            );

            let rpu = &rpus[f];

            if let Ok(rpu_serialized) = serde_json::to_string_pretty(&rpu) {
                println!("{}", rpu_serialized);
            }
        }

        Ok(())
    }
}
