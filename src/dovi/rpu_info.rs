use std::collections::HashMap;
use std::fmt::Write;
use std::path::PathBuf;

use anyhow::{Result, bail, ensure};
use dolby_vision::rpu::dovi_rpu::DoviRpu;
use dolby_vision::rpu::extension_metadata::MasteringDisplayPrimaries;
use dolby_vision::rpu::extension_metadata::blocks::{
    ExtMetadataBlock, ExtMetadataBlockLevel1, ExtMetadataBlockLevel2, ExtMetadataBlockLevel5,
    ExtMetadataBlockLevel6, ExtMetadataBlockLevel8,
};
use dolby_vision::rpu::utils::parse_rpu_file;
use dolby_vision::rpu::vdr_dm_data::CmVersion;
use dolby_vision::utils::{nits_to_pq_12_bit, pq_to_nits};
use itertools::Itertools;

use super::input_from_either;
use crate::commands::InfoArgs;

pub struct RpuInfo {
    input: PathBuf,
}

pub struct AggregateStats {
    pub min: f64,
    pub max: f64,
    pub avg: f64,
}

pub struct RpusListSummary {
    pub count: usize,
    pub scene_count: usize,
    pub rpu_mastering_meta_str: String,
    pub profiles_str: String,
    pub dm_version_str: &'static str,
    pub dm_version_counts: Option<(usize, usize)>,
    pub dmv2: bool,
    pub l6_meta: Option<Vec<String>>,
    pub l5_str: String,
    pub l8_trims: Option<Vec<String>>,
    pub l9_mdp: Option<Vec<String>>,

    pub l1_data: Vec<AggregateStats>,
    pub l1_stats: SummaryL1Stats,
    pub l2_trims: Vec<String>,
    pub l2_data: Option<Vec<ExtMetadataBlockLevel2>>,
    pub l2_stats: Option<SummaryTrimsStats>,
    pub l8_data: Option<Vec<ExtMetadataBlockLevel8>>,
    pub l8_stats_trims: Option<SummaryTrimsStats>,
    pub l8_stats_saturation: Option<SummaryL8VectorStats>,
    pub l8_stats_hue: Option<SummaryL8VectorStats>,
}

pub struct SummaryL1Stats {
    pub maxcll: f64,
    pub maxcll_avg: f64,

    pub maxfall: f64,
    pub maxfall_avg: f64,

    pub max_min_nits: f64,
}

pub struct SummaryTrimsStats {
    pub slope: AggregateStats,
    pub offset: AggregateStats,
    pub power: AggregateStats,
    pub chroma: AggregateStats,
    pub saturation: AggregateStats,
    pub ms_weight: AggregateStats,
    pub target_mid_contrast: Option<AggregateStats>,
    pub clip_trim: Option<AggregateStats>,
}

pub struct SummaryL8VectorStats {
    pub red: AggregateStats,
    pub yellow: AggregateStats,
    pub green: AggregateStats,
    pub cyan: AggregateStats,
    pub blue: AggregateStats,
    pub magenta: AggregateStats,
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
                l5_str,
                l8_trims,
                l9_mdp,
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

            write!(summary_str, "\n  L5 offsets: {l5_str}")?;

            if !l2_trims.is_empty() {
                write!(summary_str, "\n  L2 trims: {}", l2_trims.join(", "))?;
            }

            if let Some(l8_trims) = l8_trims.filter(|v| !v.is_empty()) {
                write!(summary_str, "\n  L8 trims: {}", l8_trims.join(", "))?;
            }

            if let Some(l9_mdp) = l9_mdp.filter(|v| !v.is_empty()) {
                write!(summary_str, "\n  L9 MDP: {}", l9_mdp.join(", "))?;
            }

            println!("\n{summary_str}");
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

        let dmv2 = dmv2_count == dmv1_count;
        let (dm_version_counts, dm_version_str) = if dmv2 {
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

        let l1_data = rpus
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

                    AggregateStats {
                        min: min_pq,
                        max: max_pq,
                        avg: avg_pq,
                    }
                } else {
                    unreachable!();
                }
            })
            .collect::<Vec<_>>();

        let min_pq_max_value = l1_data
            .iter()
            .map(|e| e.min)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();

        let max_pq_value = l1_data
            .iter()
            .map(|e| e.max)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        let max_pq_mean_value = l1_data.iter().map(|e| e.max).sum::<f64>() / l1_data.len() as f64;

        let max_avg_pq_value = l1_data
            .iter()
            .map(|e| e.avg)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        let avg_pq_mean_value = l1_data.iter().map(|e| e.avg).sum::<f64>() / l1_data.len() as f64;

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

        let l5_blocks = rpus
            .iter()
            .filter_map(|rpu| {
                rpu.vdr_dm_data.as_ref()?.get_block(5).and_then(|block| {
                    if let ExtMetadataBlock::Level5(l5) = block {
                        Some(l5)
                    } else {
                        None
                    }
                })
            })
            .unique()
            .collect::<Vec<_>>();

        type L5Mapping = (&'static str, fn(&ExtMetadataBlockLevel5) -> u16);
        let l5_mappings: [L5Mapping; 4] = [
            ("top", |l5| l5.active_area_top_offset),
            ("bottom", |l5| l5.active_area_bottom_offset),
            ("left", |l5| l5.active_area_left_offset),
            ("right", |l5| l5.active_area_right_offset),
        ];
        let l5_str = l5_mappings
            .iter()
            .map(|(area, offset_extractor)| {
                l5_blocks
                    .iter()
                    .map(|l5| offset_extractor(l5))
                    .minmax()
                    .into_option()
                    .map_or(format!("{area}=N/A"), |(min, max)| {
                        if min == max {
                            format!("{area}={min}")
                        } else {
                            format!("{area}={min}..{max}")
                        }
                    })
            })
            .join(", ");

        let l8_trims = if dmv2_count > 0 {
            let l8_trims_str: Vec<String> = rpus
                .iter()
                .filter_map(|rpu| {
                    rpu.vdr_dm_data.as_ref()?.get_block(8).and_then(|block| {
                        if let ExtMetadataBlock::Level8(l8) = block {
                            Some(l8.trim_target_nits())
                        } else {
                            None
                        }
                    })
                })
                .unique()
                .sorted()
                .map(|target_nits| format!("{target_nits} nits"))
                .collect();

            Some(l8_trims_str)
        } else {
            None
        };

        let l9_mdp = if dmv2_count > 0 {
            let l9_mdp_str: Vec<String> = rpus
                .iter()
                .filter_map(|rpu| {
                    rpu.vdr_dm_data.as_ref()?.get_block(9).and_then(|block| {
                        if let ExtMetadataBlock::Level9(l9) = block {
                            Some(l9.source_primary_index)
                        } else {
                            None
                        }
                    })
                })
                .fold(HashMap::new(), |mut frames, idx| {
                    *frames.entry(idx).or_insert(0) += 1;
                    frames
                })
                .into_iter()
                .sorted_by_key(|e| e.0)
                .map(|(idx, frames)| {
                    let alias = MasteringDisplayPrimaries::from(idx).to_string();
                    if frames < dmv2_count {
                        format!("{alias} ({frames})")
                    } else {
                        alias
                    }
                })
                .collect();

            Some(l9_mdp_str)
        } else {
            None
        };

        Ok(Self {
            count: rpus.len(),
            scene_count,
            rpu_mastering_meta_str,
            profiles_str,
            dm_version_str,
            dm_version_counts,
            dmv2,
            l6_meta,
            l5_str,
            l8_trims,
            l9_mdp,
            l1_data,
            l1_stats,
            l2_trims,
            l2_data: None,
            l2_stats: None,
            l8_data: None,
            l8_stats_trims: None,
            l8_stats_saturation: None,
            l8_stats_hue: None,
        })
    }

    pub fn with_l2_data(rpus: &[DoviRpu], target_nits: u16) -> Result<Self> {
        let mut summary = Self::new(rpus)?;

        let target_max_pq = nits_to_pq_12_bit(target_nits);
        let l2_data = rpus
            .iter()
            .map(|rpu| {
                rpu.vdr_dm_data
                    .as_ref()
                    .and_then(|dm| {
                        dm.level_blocks_iter(2).find_map(|block| match block {
                            ExtMetadataBlock::Level2(l2) if l2.target_max_pq == target_max_pq => {
                                Some(l2.clone())
                            }
                            _ => None,
                        })
                    })
                    .unwrap_or_default()
            })
            .collect::<Vec<_>>();

        summary.l2_stats = Some(SummaryTrimsStats {
            slope: Self::min_max_avg(&l2_data, |e| e.trim_slope as f64),
            offset: Self::min_max_avg(&l2_data, |e| e.trim_offset as f64),
            power: Self::min_max_avg(&l2_data, |e| e.trim_power as f64),
            chroma: Self::min_max_avg(&l2_data, |e| e.trim_chroma_weight as f64),
            saturation: Self::min_max_avg(&l2_data, |e| e.trim_saturation_gain as f64),
            ms_weight: Self::min_max_avg(&l2_data, |e| e.ms_weight as f64),
            target_mid_contrast: None,
            clip_trim: None,
        });
        summary.l2_data = Some(l2_data);

        Ok(summary)
    }

    fn with_l8_data(rpus: &[DoviRpu], target_nits: u16) -> Result<Self> {
        let mut summary = Self::new(rpus)?;

        let l8_data = rpus
            .iter()
            .map(|rpu| {
                rpu.vdr_dm_data
                    .as_ref()
                    .and_then(|dm| {
                        dm.level_blocks_iter(8).find_map(|block| match block {
                            ExtMetadataBlock::Level8(l8)
                                if l8.trim_target_nits() == target_nits =>
                            {
                                Some(l8.clone())
                            }
                            _ => None,
                        })
                    })
                    .unwrap_or_default()
            })
            .collect::<Vec<_>>();

        summary.l8_data = Some(l8_data);
        Ok(summary)
    }

    pub fn with_l8_trims_data(rpus: &[DoviRpu], target_nits: u16) -> Result<Self> {
        let mut summary = Self::with_l8_data(rpus, target_nits)?;

        if let Some(l8_data) = summary.l8_data.as_ref() {
            summary.l8_stats_trims = Some(SummaryTrimsStats {
                slope: Self::min_max_avg(l8_data, |e| e.trim_slope as f64),
                offset: Self::min_max_avg(l8_data, |e| e.trim_offset as f64),
                power: Self::min_max_avg(l8_data, |e| e.trim_power as f64),
                chroma: Self::min_max_avg(l8_data, |e| e.trim_chroma_weight as f64),
                saturation: Self::min_max_avg(l8_data, |e| e.trim_saturation_gain as f64),
                ms_weight: Self::min_max_avg(l8_data, |e| e.ms_weight as f64),
                target_mid_contrast: Some(Self::min_max_avg(l8_data, |e| {
                    e.target_mid_contrast as f64
                })),
                clip_trim: Some(Self::min_max_avg(l8_data, |e| e.clip_trim as f64)),
            });
        }

        Ok(summary)
    }

    pub fn with_l8_saturation_data(rpus: &[DoviRpu], target_nits: u16) -> Result<Self> {
        let mut summary = Self::with_l8_data(rpus, target_nits)?;

        if let Some(l8_data) = summary.l8_data.as_ref() {
            summary.l8_stats_saturation = Some(SummaryL8VectorStats {
                red: Self::min_max_avg(l8_data, |e| e.saturation_vector_field0 as f64),
                yellow: Self::min_max_avg(l8_data, |e| e.saturation_vector_field1 as f64),
                green: Self::min_max_avg(l8_data, |e| e.saturation_vector_field2 as f64),
                cyan: Self::min_max_avg(l8_data, |e| e.saturation_vector_field3 as f64),
                blue: Self::min_max_avg(l8_data, |e| e.saturation_vector_field4 as f64),
                magenta: Self::min_max_avg(l8_data, |e| e.saturation_vector_field5 as f64),
            });
        }

        Ok(summary)
    }

    pub fn with_l8_hue_data(rpus: &[DoviRpu], target_nits: u16) -> Result<Self> {
        let mut summary = Self::with_l8_data(rpus, target_nits)?;

        if let Some(l8_data) = summary.l8_data.as_ref() {
            summary.l8_stats_hue = Some(SummaryL8VectorStats {
                red: Self::min_max_avg(l8_data, |e| e.hue_vector_field0 as f64),
                yellow: Self::min_max_avg(l8_data, |e| e.hue_vector_field1 as f64),
                green: Self::min_max_avg(l8_data, |e| e.hue_vector_field2 as f64),
                cyan: Self::min_max_avg(l8_data, |e| e.hue_vector_field3 as f64),
                blue: Self::min_max_avg(l8_data, |e| e.hue_vector_field4 as f64),
                magenta: Self::min_max_avg(l8_data, |e| e.hue_vector_field5 as f64),
            });
        }

        Ok(summary)
    }

    fn min_max_avg<T, F>(data: &[T], field_extractor: F) -> AggregateStats
    where
        F: Fn(&T) -> f64,
    {
        let mut iter = data.iter().map(field_extractor);
        let first = iter.next().unwrap();
        let (mut min, mut max, mut sum) = (first, first, first);

        for v in iter {
            if v < min {
                min = v;
            }
            if v > max {
                max = v;
            }
            sum += v;
        }

        AggregateStats {
            min,
            max,
            avg: sum / data.len() as f64,
        }
    }
}
