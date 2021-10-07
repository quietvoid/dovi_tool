use anyhow::{bail, Result};
use dolby_vision::st2094_10::ExtMetadataBlock;
use serde_json::Value;
use std::fs::File;
use std::io::{stdout, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use dolby_vision::rpu::dovi_rpu::DoviRpu;
use dolby_vision::st2094_10::generate::{GenerateConfig, Level1Metadata, Level6Metadata};
use dolby_vision::utils::nits_to_pq;
use dolby_vision::xml::CmXmlParser;

use crate::commands::Command;

use super::OUT_NAL_HEADER;

pub struct Generator {
    json_path: Option<PathBuf>,
    rpu_out: PathBuf,
    hdr10plus_path: Option<PathBuf>,
    xml_path: Option<PathBuf>,
    canvas_width: Option<u16>,
    canvas_height: Option<u16>,
    madvr_path: Option<PathBuf>,
}

impl Generator {
    pub fn generate(cmd: Command) -> Result<()> {
        if let Command::Generate {
            json_file,
            rpu_out,
            hdr10plus_json,
            xml,
            canvas_width,
            canvas_height,
            madvr_file,
        } = cmd
        {
            let out_path = if let Some(out_path) = rpu_out {
                out_path
            } else {
                PathBuf::from("RPU_generated.bin".to_string())
            };

            let generator = Generator {
                json_path: json_file,
                rpu_out: out_path,
                hdr10plus_path: hdr10plus_json,
                xml_path: xml,
                canvas_width,
                canvas_height,
                madvr_path: madvr_file,
            };

            println!("Generating metadata...");

            if let Some(json_path) = &generator.json_path {
                let json_file = File::open(json_path)?;
                let mut config: GenerateConfig = serde_json::from_reader(&json_file)?;

                println!("{:#?}", config);

                generator.execute(&mut config)?;
            } else if let Some(xml_path) = &generator.xml_path {
                generator.generate_from_xml(xml_path)?;
            } else {
                bail!("Missing configuration or XML file!");
            }

            println!("Done.");
        }

        Ok(())
    }

    fn execute(&self, config: &mut GenerateConfig) -> Result<()> {
        let (l1_meta, scene_cuts) = if let Some(hdr10plus_path) = &self.hdr10plus_path {
            parse_hdr10plus_for_l1(hdr10plus_path)?
        } else if let Some(madvr_path) = &self.madvr_path {
            let (l1, l6, scene_cuts) = generate_metadata_from_madvr(madvr_path)?;

            // Set MaxCLL and MaxFALL if not set in config
            if let Some(ref mut config_l6) = config.level6 {
                if config_l6.max_content_light_level == 0 {
                    config_l6.max_content_light_level = l6.max_content_light_level;
                }

                if config_l6.max_frame_average_light_level == 0 {
                    config_l6.max_frame_average_light_level = l6.max_frame_average_light_level;
                }
            }

            (l1, scene_cuts)
        } else {
            (None, Vec::new())
        };

        let mut writer = BufWriter::with_capacity(
            100_000,
            File::create(&self.rpu_out).expect("Can't create file"),
        );

        let length = if let Some(l1) = &l1_meta {
            l1.len()
        } else {
            config.length as usize
        };

        for i in 0..length {
            let mut rpu = DoviRpu::profile8_config(config);

            let encoded_rpu = if let Some(l1_list) = &l1_meta {
                if let Some(meta) = &l1_list.get(i) {
                    if let Some(dm_meta) = &mut rpu.vdr_dm_data {
                        dm_meta.st2094_10_metadata.add_level1_metadata(
                            meta.min_pq,
                            meta.max_pq,
                            meta.avg_pq,
                        );

                        if scene_cuts.contains(&i) {
                            dm_meta.set_scene_cut(true);
                        }
                    }
                }

                rpu.write_rpu_data()?
            } else {
                rpu.write_rpu_data()?
            };

            writer.write_all(OUT_NAL_HEADER)?;

            // Remove 0x7C01
            writer.write_all(&encoded_rpu[2..])?;
        }

        println!("Generated metadata for {} frames", length);

        writer.flush()?;

        Ok(())
    }

    fn generate_from_xml(&self, xml_path: &Path) -> Result<()> {
        let mut s = String::new();
        File::open(xml_path)?.read_to_string(&mut s)?;

        let parser = CmXmlParser::new(s)?;

        let length = parser.get_video_length();

        let level5 = if self.canvas_width.is_some() && self.canvas_height.is_some() {
            let cw = self.canvas_width.unwrap();
            let ch = self.canvas_height.unwrap();

            parser.get_global_level5(cw, ch)
        } else {
            None
        };

        let level6 = parser.get_hdr10_metadata();

        let config = GenerateConfig {
            length: length as u64,
            level5,
            level6: Some(level6.clone()),
            ..Default::default()
        };

        let mut writer = BufWriter::with_capacity(
            100_000,
            File::create(&self.rpu_out).expect("Can't create file"),
        );

        let shots = parser.get_shots();

        for shot in shots {
            let end = shot.duration;

            for i in 0..end {
                let mut rpu = DoviRpu::profile8_config(&config);

                if let Some(dm_meta) = &mut rpu.vdr_dm_data {
                    if let Some(l1_list) = &shot.level1 {
                        if let Some(meta) = l1_list.get(i) {
                            dm_meta.st2094_10_metadata.add_level1_metadata(
                                meta.min_pq,
                                meta.max_pq,
                                meta.avg_pq,
                            );

                            if i == 0 {
                                dm_meta.set_scene_cut(true);
                            }
                        }
                    }

                    if let Some(l2_list) = &shot.level2 {
                        if let Some(meta) = l2_list.get(i) {
                            for l2 in meta {
                                dm_meta.st2094_10_metadata.add_level2_metadata(
                                    l2.target_nits,
                                    l2.trim_slope,
                                    l2.trim_offset,
                                    l2.trim_power,
                                    l2.trim_chroma_weight,
                                    l2.trim_saturation_gain,
                                    l2.ms_weight,
                                )
                            }
                        }
                    }

                    if let Some(l3_list) = &shot.level3 {
                        if let Some(meta) = l3_list.get(i) {
                            dm_meta.st2094_10_metadata.add_level3_metadata(
                                meta.min_pq_offset,
                                meta.max_pq_offset,
                                meta.avg_pq_offset,
                            );
                        }
                    }

                    if let Some(l5_list) = &shot.level5 {
                        if let Some(ar) = l5_list.get(i) {
                            if self.canvas_width.is_some() && self.canvas_height.is_some() {
                                let cw = self.canvas_width.unwrap();
                                let ch = self.canvas_height.unwrap();

                                let level5_block = dm_meta
                                    .st2094_10_metadata
                                    .ext_metadata_blocks
                                    .iter_mut()
                                    .find(|e| matches!(e, ExtMetadataBlock::Level5(_)));

                                if let Some(ExtMetadataBlock::Level5(ref mut existing_l5)) =
                                    level5_block
                                {
                                    // Existing L5 block to override
                                    let (left, right, top, bottom) = if let Some(l5) =
                                        CmXmlParser::calculate_level5_metadata(ar, cw, ch)
                                    {
                                        // AR requires an offset
                                        l5.get_offsets()
                                    } else {
                                        // AR doesn't need an offset
                                        (0, 0, 0, 0)
                                    };

                                    existing_l5.set_offsets(left, right, top, bottom);
                                } else if let Some(l5) =
                                    CmXmlParser::calculate_level5_metadata(ar, cw, ch)
                                {
                                    // No L5 block, add one
                                    dm_meta.st2094_10_metadata.add_level5_metadata(
                                        l5.active_area_left_offset,
                                        l5.active_area_right_offset,
                                        l5.active_area_top_offset,
                                        l5.active_area_bottom_offset,
                                    );
                                }
                            }
                        }
                    }
                }

                let encoded_rpu = rpu.write_rpu_data()?;

                writer.write_all(OUT_NAL_HEADER)?;

                // Remove 0x7C01
                writer.write_all(&encoded_rpu[2..])?;
            }
        }

        println!("Generated metadata for {} frames", length);

        writer.flush()?;

        Ok(())
    }
}

fn parse_hdr10plus_for_l1(
    hdr10plus_path: &Path,
) -> Result<(Option<Vec<Level1Metadata>>, Vec<usize>)> {
    let mut l1_meta = None;
    let mut scene_cuts: Vec<usize> = Vec::new();

    println!("Parsing HDR10+ JSON file...");
    stdout().flush().ok();

    let mut s = String::new();
    File::open(hdr10plus_path)?.read_to_string(&mut s)?;

    let hdr10plus: Value = serde_json::from_str(&s)?;

    if let Some(json) = hdr10plus.as_object() {
        if let Some(scene_info) = json.get("SceneInfo") {
            if let Some(list) = scene_info.as_array() {
                let info_list = list
                    .iter()
                    .filter_map(|e| e.as_object())
                    .map(|e| {
                        let lum_v = e.get("LuminanceParameters").unwrap();
                        let lum = lum_v.as_object().unwrap();

                        let avg_rgb = lum.get("AverageRGB").unwrap().as_u64().unwrap();
                        let maxscl = lum.get("MaxScl").unwrap().as_array().unwrap();

                        let max_rgb = maxscl.iter().filter_map(|e| e.as_u64()).max().unwrap();

                        let scene_frame_index =
                            e.get("SceneFrameIndex").unwrap().as_u64().unwrap() as usize;

                        if scene_frame_index == 0 {
                            let sequence_frame_index =
                                e.get("SequenceFrameIndex").unwrap().as_u64().unwrap() as usize;

                            scene_cuts.push(sequence_frame_index);
                        }

                        Level1Metadata {
                            min_pq: 0,
                            max_pq: (nits_to_pq((max_rgb as f64 / 10.0).round() as u16) * 4095.0)
                                .round() as u16,
                            avg_pq: (nits_to_pq((avg_rgb as f64 / 10.0).round() as u16) * 4095.0)
                                .round() as u16,
                        }
                    })
                    .collect();

                l1_meta = Some(info_list)
            }
        }
    }

    Ok((l1_meta, scene_cuts))
}

pub fn generate_metadata_from_madvr(
    madvr_path: &Path,
) -> Result<(Option<Vec<Level1Metadata>>, Level6Metadata, Vec<usize>)> {
    println!("Parsing madVR measurement file...");
    stdout().flush().ok();

    let madvr_info = madvr_parse::MadVRMeasurements::parse_file(madvr_path)?;

    let mut l1_meta = Some(Vec::with_capacity(madvr_info.frames.len()));
    let l6_meta = Level6Metadata {
        max_content_light_level: madvr_info.header.maxcll as u16,
        max_frame_average_light_level: madvr_info.header.maxfall as u16,
        ..Default::default()
    };

    let scene_cuts: Vec<usize> = madvr_info.scenes.iter().map(|s| s.start as usize).collect();

    if let Some(ref mut meta) = l1_meta {
        madvr_info.scenes.iter().for_each(|s| {
            let shot_l1 = Level1Metadata {
                min_pq: 0,
                max_pq: (s.max_pq * 4095.0).round() as u16,
                avg_pq: (s.avg_pq * 4095.0).round() as u16,
            };

            let l1_list = std::iter::repeat(shot_l1).take(s.length);
            meta.extend(l1_list);
        });
    }

    Ok((l1_meta, l6_meta, scene_cuts))
}
