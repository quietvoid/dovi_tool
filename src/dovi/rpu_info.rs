use std::fmt::Write;
use std::fs::File;
use std::io::Write as WriteFile;
use std::path::PathBuf;

use anyhow::{Result, bail, ensure};
use dolby_vision::rpu::vdr_dm_data::CmVersion;
use itertools::Itertools;

use dolby_vision::rpu::dovi_rpu::DoviRpu;
use dolby_vision::rpu::extension_metadata::blocks::{
    ExtMetadataBlock, ExtMetadataBlockLevel1, ExtMetadataBlockLevel6,
};
use dolby_vision::rpu::utils::parse_rpu_file;
use dolby_vision::utils::pq_to_nits;

use super::input_from_either;
use crate::commands::InfoArgs;

pub struct RpuInfo {
    input: PathBuf,
}

pub struct RpusListSummary {
    pub count: usize,
    pub scene_count: usize,
    pub rpu_mastering_meta_str: String,
    pub profiles_str: String,
    pub dm_version_str: &'static str,
    pub dm_version_counts: Option<(usize, usize)>,
    pub l6_meta: Option<Vec<String>>,

    pub l1_data: Vec<(f64, f64, f64)>,
    pub l1_stats: SummaryL1Stats,
    pub l2_trims: Vec<String>,
}

pub struct SummaryL1Stats {
    pub maxcll: f64,
    pub maxcll_avg: f64,

    pub maxfall: f64,
    pub maxfall_avg: f64,

    pub max_min_nits: f64,
}

impl RpuInfo {
    pub fn info(args: InfoArgs) -> Result<()> {
        let InfoArgs {
            input,
            input_pos,
            frame,
            summary,
            qpfile,
        } = args;

        if !summary && !qpfile.is_some() && frame.is_none() {
            bail!("No frame number to look up");
        }

        let input = input_from_either("info", input, input_pos)?;

        let info = RpuInfo { input };

        println!("Parsing RPU file...");

        let rpus = parse_rpu_file(info.input)?;

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
                println!("{rpu_serialized}");
            }
        }

        if summary {
            let RpusListSummary {
                count,
                rpu_mastering_meta_str,
                scene_count,
                profiles_str,
                dm_version_str,
                dm_version_counts,
                l6_meta,
                l1_stats,
                l2_trims,
                ..
            } = RpusListSummary::new(&rpus)?;

            // Summary output
            let mut summary_str = format!(
                "Summary:\n  Frames: {count}\n  {profiles_str}\n  DM version: {dm_version_str}"
            );

            if let Some((dmv1_count, dmv2_count)) = dm_version_counts {
                write!(
                    summary_str,
                    "\n    v2.9 count: {dmv1_count}\n    v4.0 count: {dmv2_count}"
                )?;
            }

            write!(summary_str, "\n  Scene/shot count: {scene_count}")?;
            write!(summary_str, "\n  {rpu_mastering_meta_str}")?;
            write!(
                summary_str,
                "\n  RPU content light level (L1): MaxCLL: {:.2} nits, MaxFALL: {:.2} nits",
                l1_stats.maxcll, l1_stats.maxfall
            )?;

            if let Some(l6_meta) = l6_meta {
                let mut final_str = String::from("L6 metadata");
                if l6_meta.len() > 1 {
                    write!(final_str, "\n    {}", l6_meta.join("\n    "))?;
                } else {
                    write!(final_str, ": {}", l6_meta.first().unwrap())?;
                }

                write!(summary_str, "\n  {final_str}")?;
            }

            if !l2_trims.is_empty() {
                write!(summary_str, "\n  L2 trims: {}", l2_trims.join(", "))?;
            }

            println!("\n{summary_str}");
        }

        if qpfile.is_some() {
            let mut qpfile_file =
                File::create(qpfile.unwrap()).expect("Error opening qpfile for writing!");

            let scene_cuts = rpus.iter().positions(|rpu| {
                rpu.vdr_dm_data
                    .as_ref()
                    .and_then(|vdr| (vdr.scene_refresh_flag == 1).then_some(1))
                    .is_some()
            });

            for scene in scene_cuts {
                qpfile_file.write(format!("{} K\n", scene).as_bytes())?;
            }
        }

        Ok(())
    }
}

impl RpusListSummary {
    pub fn new(rpus: &[DoviRpu]) -> Result<Self> {
        let profiles = rpus
            .iter()
            .map(|rpu| rpu.dovi_profile)
            .unique()
            .sorted()
            .join(", ");

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

        let (dm_version_counts, dm_version_str) = if dmv2_count == dmv1_count {
            (None, "2 (CM v4.0)")
        } else if dmv2_count == 0 {
            (None, "1 (CM v2.9)")
        } else {
            (Some((dmv1_count, dmv2_count)), "1 + 2 (CM 2.9 and 4.0)")
        };

        let scene_count = rpus
            .iter()
            .filter(|rpu| {
                rpu.vdr_dm_data
                    .as_ref()
                    .and_then(|vdr| (vdr.scene_refresh_flag == 1).then_some(1))
                    .is_some()
            })
            .count();

        // Profile
        let mut profiles_str = "Profile".to_string();
        if profiles.contains(", ") {
            write!(profiles_str, "s")?;
        }
        write!(profiles_str, ": {profiles}")?;

        if profiles.contains('7') {
            let idx = profiles_str.find('7').unwrap();

            let subprofiles = rpus
                .iter()
                .filter_map(|rpu| rpu.el_type.as_ref().map(|e| e.to_string()))
                .unique()
                .sorted()
                .join(", ");

            profiles_str.insert_str(idx + 1, &format!(" ({subprofiles})"));
        }

        let mut rpu_mastering_meta_str = String::from("RPU mastering display: ");
        let rpu_mastering_meta = rpus
            .iter()
            .filter_map(|rpu| {
                rpu.vdr_dm_data
                    .as_ref()
                    .map(|vdr| (vdr.source_min_pq, vdr.source_max_pq))
            })
            .unique()
            .sorted()
            .map(|meta| {
                let min = (pq_to_nits(meta.0 as f64 / 4095.0) * 1e6).round() / 1e6;
                let max = (pq_to_nits(meta.1 as f64 / 4095.0) / 1000.0).round() * 1000.0;

                format!("{min:.4}/{max} nits")
            })
            .join(", ");
        rpu_mastering_meta_str.push_str(&rpu_mastering_meta);

        let l6_meta: Vec<ExtMetadataBlockLevel6> = rpus
            .iter()
            .filter_map(|rpu| {
                rpu.vdr_dm_data.as_ref().and_then(|vdr| {
                    vdr.get_block(6).map(|b| {
                        if let ExtMetadataBlock::Level6(l6) = b {
                            l6
                        } else {
                            unreachable!()
                        }
                    })
                })
            })
            .unique()
            .cloned()
            .collect();

        let l6_meta = if !l6_meta.is_empty() {
            let l6_meta_str: Vec<String> = l6_meta.iter().map(|l6| {
                let min = l6.min_display_mastering_luminance as f64 / 10000.0;
                let max = l6.max_display_mastering_luminance;
                let maxcll = l6.max_content_light_level;
                let maxfall = l6.max_frame_average_light_level;

                format!("Mastering display: {min:.4}/{max} nits. MaxCLL: {maxcll} nits, MaxFALL: {maxfall} nits")
            }).collect();

            Some(l6_meta_str)
        } else {
            None
        };

        let cm_version = if dmv2_count > 0 {
            CmVersion::V40
        } else {
            CmVersion::V29
        };
        let default_l1_for_missing = ExtMetadataBlock::Level1(
            ExtMetadataBlockLevel1::from_stats_cm_version(0, 0, 0, cm_version),
        );

        let l1_data: Vec<_> = rpus
            .iter()
            .map(|rpu| {
                let block = rpu
                    .vdr_dm_data
                    .as_ref()
                    .and_then(|dm| dm.get_block(1))
                    .unwrap_or(&default_l1_for_missing);

                if let ExtMetadataBlock::Level1(l1) = block {
                    let min_pq = (l1.min_pq as f64) / 4095.0;
                    let max_pq = (l1.max_pq as f64) / 4095.0;
                    let avg_pq = (l1.avg_pq as f64) / 4095.0;

                    (min_pq, max_pq, avg_pq)
                } else {
                    unreachable!();
                }
            })
            .collect();

        let max_pq_value = l1_data
            .iter()
            .map(|e| e.1)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        let max_pq_mean_value = l1_data.iter().map(|e| e.1).sum::<f64>() / l1_data.len() as f64;
        let max_avg_pq_value = l1_data
            .iter()
            .map(|e| e.2)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        let avg_pq_mean_value = l1_data.iter().map(|e| e.2).sum::<f64>() / l1_data.len() as f64;

        let min_pq_max_value = l1_data
            .iter()
            .map(|e| e.0)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();

        let l1_stats = SummaryL1Stats {
            maxcll: pq_to_nits(max_pq_value),
            maxcll_avg: pq_to_nits(max_pq_mean_value),
            maxfall: pq_to_nits(max_avg_pq_value),
            maxfall_avg: pq_to_nits(avg_pq_mean_value),
            max_min_nits: pq_to_nits(min_pq_max_value),
        };

        let l2_trims: Vec<String> = rpus
            .iter()
            .filter_map(|rpu| {
                rpu.vdr_dm_data.as_ref().map(|vdr| {
                    vdr.level_blocks_iter(2)
                        .map(|b| {
                            if let ExtMetadataBlock::Level2(l2) = b {
                                l2.target_max_pq
                            } else {
                                unreachable!()
                            }
                        })
                        .unique()
                })
            })
            .flatten()
            .unique()
            .map(|target_max_pq| {
                ((pq_to_nits(target_max_pq as f64 / 4095.0) / 100.0).round() * 100.0) as u16
            })
            .map(|target_nits| format!("{target_nits} nits"))
            .collect();

        Ok(Self {
            count: rpus.len(),
            scene_count,
            rpu_mastering_meta_str,
            profiles_str,
            dm_version_str,
            dm_version_counts,
            l6_meta,
            l1_data,
            l1_stats,
            l2_trims,
        })
    }
}
