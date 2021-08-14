use std::path::PathBuf;

use super::{parse_rpu_file, rpu::DoviRpu};

pub struct RpuInfo {
    input: PathBuf,
    frame: Option<usize>,
    rpus: Option<Vec<DoviRpu>>,
}

impl RpuInfo {
    pub fn info(input: PathBuf, frame: Option<usize>) {
        let mut info = RpuInfo {
            input,
            frame,
            rpus: None,
        };

        info.rpus = parse_rpu_file(&info.input);

        if let Some(ref rpus) = info.rpus {
            if let Some(f) = info.frame {
                assert!(f < rpus.len());

                let rpu = &rpus[f];

                if let Ok(rpu_serialized) = serde_json::to_string_pretty(&rpu) {
                    println!("{}", rpu_serialized);
                }
            }
        }
    }
}
