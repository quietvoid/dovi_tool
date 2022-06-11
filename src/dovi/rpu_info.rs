use std::fmt::Write as FmtWrite;
use std::io::{stdout, Write};
use std::path::PathBuf;

use anyhow::{bail, ensure, Result};

use dolby_vision::rpu::dovi_rpu::DoviRpu;
use utilities_dovi::parse_rpu_file;

use super::input_from_either;
use crate::commands::InfoArgs;

pub struct RpuInfo {
    input: PathBuf,
    rpus: Option<Vec<DoviRpu>>,
}

impl RpuInfo {
    pub fn info(args: InfoArgs) -> Result<()> {
        let InfoArgs {
            input,
            input_pos,
            frame,
            summary,
        } = args;

        if !summary && frame.is_none() {
            bail!("No frame number to look up");
        }

        let input = input_from_either("info", input, input_pos)?;

        let mut info = RpuInfo { input, rpus: None };

        println!("Parsing RPU file...");
        stdout().flush().ok();

        info.rpus = parse_rpu_file(&info.input)?;

        if let Some(ref rpus) = info.rpus {
            if let Some(f) = frame {
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

            if summary {
                let count = rpus.len();

                let dmv1_count = rpus
                    .iter()
                    .filter(|rpu| {
                        rpu.vdr_dm_data
                            .as_ref()
                            .and_then(|vdr| vdr.cmv29_metadata.as_ref())
                            .is_some()
                    })
                    .count();
                let dmv2_count = rpus
                    .iter()
                    .filter(|rpu| {
                        rpu.vdr_dm_data
                            .as_ref()
                            .and_then(|vdr| vdr.cmv40_metadata.as_ref())
                            .is_some()
                    })
                    .count();

                let (needs_count, dm_version) = if dmv2_count == dmv1_count {
                    (false, "2 (CM v4.x)")
                } else if dmv2_count == 0 {
                    (false, "1 (CM v2.9)")
                } else {
                    (true, "1 + 2 (CM 2.9 and 4.x)")
                };

                let scene_count = rpus
                    .iter()
                    .filter(|rpu| {
                        rpu.vdr_dm_data
                            .as_ref()
                            .and_then(|vdr| (vdr.scene_refresh_flag == 1).then(|| 1))
                            .is_some()
                    })
                    .count();

                let mut summary_str =
                    format!("Summary:\n  Frames: {count}\n  DM version: {dm_version}");

                if needs_count {
                    write!(
                        summary_str,
                        "\n    v2.9 count: {dmv1_count}\n    v4.x count: {dmv2_count}"
                    )?;
                }

                write!(summary_str, "\n  Scene/shot count: {scene_count}")?;

                println!("\n{}", summary_str)
            }
        }

        Ok(())
    }
}
