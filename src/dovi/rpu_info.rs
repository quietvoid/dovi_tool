use anyhow::{bail, ensure, Result};
use std::path::PathBuf;

use dolby_vision::rpu::dovi_rpu::DoviRpu;

use super::parse_rpu_file;

pub struct RpuInfo {
    input: PathBuf,
    frame: Option<usize>,
    rpus: Option<Vec<DoviRpu>>,
}

impl RpuInfo {
    pub fn info(input: PathBuf, frame: Option<usize>) -> Result<()> {
        let mut info = RpuInfo {
            input,
            frame,
            rpus: None,
        };

        info.rpus = parse_rpu_file(&info.input)?;

        if let Some(ref rpus) = info.rpus {
            if let Some(f) = info.frame {
                ensure!(f < rpus.len(), format!("info: invalid frame number (out of range).\nNumber of valid RPUs parsed: {}", rpus.len()));

                let rpu = &rpus[f];

                if let Ok(rpu_serialized) = serde_json::to_string_pretty(&rpu) {
                    println!("{}", rpu_serialized);
                }
            } else {
                bail!("No frame number to look up");
            }
        }

        Ok(())
    }
}
