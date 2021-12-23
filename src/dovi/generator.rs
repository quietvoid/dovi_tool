use anyhow::{bail, Result};
use serde_json::Value;
use std::fs::File;
use std::io::{stdout, Read, Write};
use std::path::{Path, PathBuf};

use crate::commands::Command;
use dolby_vision::rpu::extension_metadata::blocks::{
    ExtMetadataBlock, ExtMetadataBlockLevel1, ExtMetadataBlockLevel6,
};
use dolby_vision::rpu::generate::{GenerateConfig, ShotFrameEdit, VideoShot};
use dolby_vision::utils::nits_to_pq;
use dolby_vision::xml::{CmXmlParser, XmlParserOpts};

pub struct Generator {
    json_path: Option<PathBuf>,
    rpu_out: PathBuf,
    hdr10plus_path: Option<PathBuf>,
    xml_path: Option<PathBuf>,
    canvas_width: Option<u16>,
    canvas_height: Option<u16>,
    madvr_path: Option<PathBuf>,
    use_custom_targets: bool,
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
            use_custom_targets,
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
                use_custom_targets,
            };

            let config = if let Some(json_path) = &generator.json_path {
                let json_file = File::open(json_path)?;
                let mut config: GenerateConfig = serde_json::from_reader(&json_file)?;

                if let Some(hdr10plus_path) = &generator.hdr10plus_path {
                    parse_hdr10plus_for_l1(hdr10plus_path, &mut config)?;
                } else if let Some(madvr_path) = &generator.madvr_path {
                    generate_metadata_from_madvr(
                        madvr_path,
                        generator.use_custom_targets,
                        &mut config,
                    )?;
                }

                config
            } else if let Some(xml_path) = &generator.xml_path {
                generator.config_from_xml(xml_path)?
            } else {
                bail!("Missing configuration or XML file!");
            };

            generator.execute(&config)?;

            println!("Done.");
        }

        Ok(())
    }

    fn execute(&self, config: &GenerateConfig) -> Result<()> {
        println!("Generating metadata...");

        config.write_rpus(&self.rpu_out)?;

        println!("Generated metadata for {} frames", config.length);

        Ok(())
    }

    fn config_from_xml(&self, xml_path: &Path) -> Result<GenerateConfig> {
        println!("Parsing XML metadata...");

        let parser_opts = XmlParserOpts {
            canvas_width: self.canvas_width,
            canvas_height: self.canvas_height,
        };

        let parser = CmXmlParser::parse_file(xml_path, parser_opts)?;

        Ok(parser.config)
    }
}

fn parse_hdr10plus_for_l1(hdr10plus_path: &Path, config: &mut GenerateConfig) -> Result<()> {
    println!("Parsing HDR10+ JSON file...");
    stdout().flush().ok();

    config.shots.clear();

    let mut s = String::new();
    File::open(hdr10plus_path)?.read_to_string(&mut s)?;

    let hdr10plus: Value = serde_json::from_str(&s)?;

    let mut frame_count = 0;

    if let Some(json) = hdr10plus.as_object() {
        // Assume a proper JSON for scene info
        let scene_summary = json
            .get("SceneInfoSummary")
            .expect("No scene info summary in JSON")
            .as_object()
            .unwrap();

        let scene_first_frames: Vec<usize> = scene_summary
            .get("SceneFirstFrameIndex")
            .expect("No scene first frame index array")
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_u64().unwrap() as usize)
            .collect();

        let scene_frame_lengths: Vec<usize> = scene_summary
            .get("SceneFrameNumbers")
            .expect("No scene frame numbers array")
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_u64().unwrap() as usize)
            .collect();

        let mut current_shot_id = 0;

        if let Some(scene_info) = json.get("SceneInfo") {
            if let Some(list) = scene_info.as_array() {
                frame_count = list.len();

                let json_frames = list.iter().filter_map(|e| e.as_object());
                let first_frames = json_frames
                    .enumerate()
                    .filter(|(frame_no, _)| scene_first_frames.contains(frame_no));

                for (frame_no, map) in first_frames {
                    // Only use the metadata from the first frame of a shot.
                    // The JSON is assumed to be shot based already.
                    let lum_v = map.get("LuminanceParameters").unwrap();
                    let lum = lum_v.as_object().unwrap();

                    let avg_rgb = lum.get("AverageRGB").unwrap().as_u64().unwrap();
                    let maxscl = lum.get("MaxScl").unwrap().as_array().unwrap();

                    let max_rgb = maxscl.iter().filter_map(|e| e.as_u64()).max().unwrap();

                    let min_pq = 0;
                    let max_pq = (nits_to_pq((max_rgb as f64 / 10.0).round() as u16) * 4095.0)
                        .round() as u16;
                    let avg_pq = (nits_to_pq((avg_rgb as f64 / 10.0).round() as u16) * 4095.0)
                        .round() as u16;

                    let shot = VideoShot {
                        start: frame_no,
                        duration: scene_frame_lengths[current_shot_id],
                        metadata_blocks: vec![ExtMetadataBlock::Level1(
                            ExtMetadataBlockLevel1::from_stats(min_pq, max_pq, avg_pq),
                        )],
                        ..Default::default()
                    };

                    config.shots.push(shot);
                    current_shot_id += 1;
                }
            }
        }
    }

    config.length = frame_count;

    Ok(())
}

pub fn generate_metadata_from_madvr(
    madvr_path: &Path,
    use_custom_targets: bool,
    config: &mut GenerateConfig,
) -> Result<()> {
    println!("Parsing madVR measurement file...");
    stdout().flush().ok();

    config.shots.clear();

    let madvr_info = madvr_parse::MadVRMeasurements::parse_file(madvr_path)?;

    let level6_meta = ExtMetadataBlockLevel6 {
        max_content_light_level: madvr_info.header.maxcll as u16,
        max_frame_average_light_level: madvr_info.header.maxfall as u16,
        ..Default::default()
    };

    let frame_count = madvr_info.frames.len();

    for s in madvr_info.scenes.iter() {
        let min_pq = 0;
        let max_pq = (s.max_pq * 4095.0).round() as u16;
        let avg_pq = (s.avg_pq * 4095.0).round() as u16;

        let mut shot = VideoShot {
            start: s.start as usize,
            duration: s.length,
            metadata_blocks: vec![ExtMetadataBlock::Level1(
                ExtMetadataBlockLevel1::from_stats(min_pq, max_pq, avg_pq),
            )],
            ..Default::default()
        };

        if use_custom_targets && madvr_info.header.flags == 3 {
            // Use peak per frame, average from scene
            let frames = s.get_frames(frame_count, &madvr_info.frames)?;

            frames.iter().enumerate().for_each(|(i, f)| {
                let min_pq = 0;
                let max_pq = (f.target_pq * 4095.0).round() as u16;
                let avg_pq = (s.avg_pq * 4095.0).round() as u16;

                shot.frame_edits.push(ShotFrameEdit {
                    edit_offset: i,
                    metadata_blocks: vec![ExtMetadataBlock::Level1(
                        ExtMetadataBlockLevel1::from_stats(min_pq, max_pq, avg_pq),
                    )],
                });
            });
        } else {
        };

        config.shots.push(shot);
    }

    // Set MaxCLL and MaxFALL if not set in config
    if config.level6.max_content_light_level == 0 {
        config.level6.max_content_light_level = level6_meta.max_content_light_level;
    }

    if config.level6.max_frame_average_light_level == 0 {
        config.level6.max_frame_average_light_level = level6_meta.max_frame_average_light_level;
    }

    config.length = frame_count;

    Ok(())
}
