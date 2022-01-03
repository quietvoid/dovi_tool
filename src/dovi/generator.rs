use anyhow::{bail, ensure, Result};
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

#[derive(Default)]
pub struct Generator {
    json_path: Option<PathBuf>,
    rpu_out: PathBuf,
    hdr10plus_path: Option<PathBuf>,
    xml_path: Option<PathBuf>,
    canvas_width: Option<u16>,
    canvas_height: Option<u16>,
    madvr_path: Option<PathBuf>,
    use_custom_targets: bool,

    pub config: Option<GenerateConfig>,
}

impl Generator {
    pub fn from_command(cmd: Command) -> Result<Generator> {
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
                config: None,
            };

            Ok(generator)
        } else {
            bail!("Invalid command variant.");
        }
    }

    pub fn generate(&mut self) -> Result<()> {
        let config = if let Some(json_path) = &self.json_path {
            let json_file = File::open(json_path)?;
            let mut config: GenerateConfig = serde_json::from_reader(&json_file)?;

            if let Some(hdr10plus_path) = &self.hdr10plus_path {
                parse_hdr10plus_for_l1(hdr10plus_path, &mut config)?;
            } else if let Some(madvr_path) = &self.madvr_path {
                generate_metadata_from_madvr(madvr_path, self.use_custom_targets, &mut config)?;
            } else if config.length == 0 && !config.shots.is_empty() {
                // Set length from sum of shot durations
                config.length = config.shots.iter().map(|s| s.duration).sum();
            }

            ensure!(
                config.length > 0 || !config.shots.is_empty(),
                "Missing number of RPUs to generate, and no shots to derive it from"
            );

            // Create a single shot by default
            if config.shots.is_empty() {
                config.shots.push(VideoShot {
                    start: 0,
                    duration: config.length,
                    ..Default::default()
                })
            }

            config
        } else if let Some(xml_path) = &self.xml_path {
            self.config_from_xml(xml_path)?
        } else {
            bail!("Missing configuration or XML file!");
        };

        self.config = Some(config);
        self.execute()?;

        println!("Done.");

        Ok(())
    }

    fn execute(&self) -> Result<()> {
        if let Some(config) = &self.config {
            println!("Generating metadata...");

            config.write_rpus(&self.rpu_out)?;

            println!("Generated metadata for {} frames", config.length);
        } else {
            bail!("No generation config to execute!");
        }

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

        let mut hdr10plus_shots = Vec::with_capacity(scene_first_frames.len());

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

                    let mut shot = VideoShot {
                        start: frame_no,
                        duration: scene_frame_lengths[current_shot_id],
                        metadata_blocks: vec![ExtMetadataBlock::Level1(
                            ExtMetadataBlockLevel1::from_stats(min_pq, max_pq, avg_pq),
                        )],
                        ..Default::default()
                    };

                    let config_shot = config.shots.get(hdr10plus_shots.len());

                    if let Some(override_shot) = config_shot {
                        shot.copy_metadata_from_shot(override_shot, Some(&[1]))
                    }

                    hdr10plus_shots.push(shot);

                    current_shot_id += 1;
                }
            }
        }

        // Now that the metadata was copied, we can replace the shots
        config.shots.clear();
        config.shots.extend(hdr10plus_shots);
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

    let madvr_info = madvr_parse::MadVRMeasurements::parse_file(madvr_path)?;

    let level6_meta = ExtMetadataBlockLevel6 {
        max_content_light_level: madvr_info.header.maxcll as u16,
        max_frame_average_light_level: madvr_info.header.maxfall as u16,
        ..Default::default()
    };

    let frame_count = madvr_info.frames.len();
    let mut madvr_shots = Vec::with_capacity(madvr_info.scenes.len());

    for (i, scene) in madvr_info.scenes.iter().enumerate() {
        let min_pq = 0;
        let max_pq = (scene.max_pq * 4095.0).round() as u16;
        let avg_pq = (scene.avg_pq * 4095.0).round() as u16;

        let mut shot = VideoShot {
            start: scene.start as usize,
            duration: scene.length,
            metadata_blocks: vec![ExtMetadataBlock::Level1(
                ExtMetadataBlockLevel1::from_stats(min_pq, max_pq, avg_pq),
            )],
            ..Default::default()
        };

        let config_shot = config.shots.get(i);

        if use_custom_targets && madvr_info.header.flags == 3 {
            // Use peak per frame, average from scene
            let frames = scene.get_frames(frame_count, &madvr_info.frames)?;

            frames.iter().enumerate().for_each(|(i, f)| {
                let min_pq = 0;
                let max_pq = (f.target_pq * 4095.0).round() as u16;
                let avg_pq = (scene.avg_pq * 4095.0).round() as u16;

                let frame_edit = ShotFrameEdit {
                    edit_offset: i,
                    metadata_blocks: vec![ExtMetadataBlock::Level1(
                        ExtMetadataBlockLevel1::from_stats(min_pq, max_pq, avg_pq),
                    )],
                };

                shot.frame_edits.push(frame_edit);
            });
        }

        if let Some(override_shot) = config_shot {
            shot.copy_metadata_from_shot(override_shot, Some(&[1]))
        }

        madvr_shots.push(shot);
    }

    // Now that the metadata was copied, we can replace the shots
    config.shots.clear();
    config.shots.extend(madvr_shots);

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
