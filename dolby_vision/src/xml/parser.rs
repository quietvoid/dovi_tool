use anyhow::{bail, ensure, Result};
use roxmltree::{Document, Node};
use std::cmp::min;
use std::collections::HashMap;
use std::convert::TryInto;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::rpu::extension_metadata::{blocks::*, primaries};
use crate::rpu::generate::{GenerateConfig, ShotFrameEdit, VideoShot};
use crate::rpu::vdr_dm_data::CmVersion;
use crate::utils::nits_to_pq;

use level10::PRESET_TARGET_DISPLAYS;
use primaries::ColorPrimaries;

#[derive(Default, Debug)]
pub struct CmXmlParser {
    opts: XmlParserOpts,
    xml_version: u16,
    separator: char,

    pub target_displays: HashMap<String, TargetDisplay>,

    pub config: GenerateConfig,
}

#[derive(Default, Debug)]
pub struct XmlParserOpts {
    pub canvas_width: Option<u16>,
    pub canvas_height: Option<u16>,
}

#[derive(Default, Debug)]
pub struct TargetDisplay {
    id: String,
    peak_nits: u16,
    min_nits: f64,
    primaries: [f64; 8],
}

impl CmXmlParser {
    pub fn parse_file<P: AsRef<Path>>(file_path: P, opts: XmlParserOpts) -> Result<CmXmlParser> {
        let mut s = String::new();
        File::open(file_path)?.read_to_string(&mut s)?;

        Self::new(s, opts)
    }

    pub fn new(s: String, opts: XmlParserOpts) -> Result<CmXmlParser> {
        let mut parser = CmXmlParser {
            opts,
            ..Default::default()
        };

        let doc = roxmltree::Document::parse(&s).unwrap();

        parser.xml_version = parser.parse_xml_version(&doc)?;

        parser.separator = if parser.is_cmv4() { ' ' } else { ',' };

        // Override version
        if !parser.is_cmv4() {
            parser.config.cm_version = CmVersion::V29;
        }

        if let Some(output) = doc.descendants().find(|e| e.has_tag_name("Output")) {
            parser.parse_global_level5(&output)?;

            if let Some(video) = output.descendants().find(|e| e.has_tag_name("Video")) {
                let (max_frame_average_light_level, max_content_light_level) =
                    parser.parse_level6(&video);
                let (min_display_mastering_luminance, max_display_mastering_luminance) =
                    parser.parse_mastering_display_metadata(&video);

                parser.config.level6 = Some(ExtMetadataBlockLevel6 {
                    max_display_mastering_luminance,
                    min_display_mastering_luminance,
                    max_content_light_level,
                    max_frame_average_light_level,
                });
                parser.config.level254 = parser.parse_level254(&video);

                parser.add_level11(&video)?;

                parser.target_displays = parser.parse_target_displays(&video)?;

                parser.config.shots = parser.parse_shots(&video)?;
                parser.config.shots.sort_by_key(|s| s.start);

                // Add default L10 blocks
                if parser.is_cmv4() {
                    parser.parse_global_level10_targets()?;
                }

                parser.config.length = parser.config.shots.iter().map(|s| s.duration).sum();
            } else {
                bail!("Could not find Video node");
            }
        } else {
            bail!("Could not find Output node");
        }

        Ok(parser)
    }

    fn parse_xml_version(&self, doc: &Document) -> Result<u16> {
        if let Some(node) = doc.descendants().find(|e| e.has_tag_name("DolbyLabsMDF")) {
            let version_attr = node.attribute("version");
            let version_node =
                if let Some(version_node) = node.children().find(|e| e.has_tag_name("Version")) {
                    version_node.text()
                } else {
                    None
                };

            let version_text = if let Some(v) = version_attr {
                v
            } else if let Some(v) = version_node {
                v
            } else {
                bail!("No XML version found!");
            };

            let version_split: Vec<&str> = version_text.split('.').collect();

            let rev = version_split
                .iter()
                .rev()
                .enumerate()
                .fold(0, |rev, (i, v)| {
                    rev + (v.parse::<u16>().unwrap() << (i * 4))
                });

            if rev >= 0x402 {
                match rev {
                    0x402 | 0x500 | 0x510 => {}
                    0x510.. => println!("Possibly unhandled new XML version {version_text} found! Please open an issue if you get anything wrong."),
                    _ => bail!("invalid XML version {version_text} found!")
                };
            } else {
                match rev {
                    0x205 => {}
                    0x1 | 0x20 | 0x201 | 0x204 => bail!(
                        "Unhandled legacy XML version {version_text} found! Please open an issue."
                    ),
                    _ => bail!("invalid legacy XML version {version_text} found!"),
                };
            }

            Ok(rev)
        } else {
            bail!("Could not find DolbyLabsMDF root node.");
        }
    }

    fn parse_level6(&self, video: &Node) -> (u16, u16) {
        if let Some(node) = video.descendants().find(|e| e.has_tag_name("Level6")) {
            let maxfall = if let Some(fall) = node.children().find(|e| e.has_tag_name("MaxFALL")) {
                fall.text()
                    .map_or(0, |e| e.parse::<f32>().unwrap().round() as u16)
            } else {
                0
            };

            let maxcll = if let Some(cll) = node.children().find(|e| e.has_tag_name("MaxCLL")) {
                cll.text()
                    .map_or(0, |e| e.parse::<f32>().unwrap().round() as u16)
            } else {
                0
            };

            (maxfall, maxcll)
        } else {
            (0, 0)
        }
    }

    fn parse_mastering_display_metadata(&self, video: &Node) -> (u16, u16) {
        if let Some(node) = video
            .descendants()
            .find(|e| e.has_tag_name("MasteringDisplay"))
        {
            let min = if let Some(min_brightness) = node
                .children()
                .find(|e| e.has_tag_name("MinimumBrightness"))
            {
                min_brightness.text().map_or(0, |e| {
                    let v = e.parse::<f32>().unwrap();
                    (v * 10000.0) as u16
                })
            } else {
                0
            };

            let max = if let Some(max_brightness) =
                node.children().find(|e| e.has_tag_name("PeakBrightness"))
            {
                max_brightness
                    .text()
                    .map_or(0, |e| e.parse::<u16>().unwrap())
            } else {
                0
            };

            (min, max)
        } else {
            (0, 0)
        }
    }

    fn parse_target_displays(&mut self, video: &Node) -> Result<HashMap<String, TargetDisplay>> {
        let mut targets = HashMap::new();
        let target_display_nodes = video
            .descendants()
            .filter(|e| e.has_tag_name("TargetDisplay"));

        for target_node in target_display_nodes {
            let id = target_node
                .children()
                .find(|e| e.has_tag_name("ID"))
                .unwrap()
                .text()
                .unwrap()
                .to_string();

            let peak_nits = target_node
                .children()
                .find(|e| e.has_tag_name("PeakBrightness"))
                .unwrap()
                .text()
                .unwrap()
                .parse::<u16>()
                .unwrap();

            let min_nits = target_node
                .children()
                .find(|e| e.has_tag_name("MinimumBrightness"))
                .unwrap()
                .text()
                .unwrap()
                .parse::<f64>()
                .unwrap();

            let primary_red = target_node
                .descendants()
                .find(|e| e.has_tag_name("Red"))
                .unwrap()
                .text()
                .unwrap();

            let primary_green = target_node
                .descendants()
                .find(|e| e.has_tag_name("Green"))
                .unwrap()
                .text()
                .unwrap();

            let primary_blue = target_node
                .descendants()
                .find(|e| e.has_tag_name("Blue"))
                .unwrap()
                .text()
                .unwrap();

            let primary_white = target_node
                .children()
                .find(|e| e.has_tag_name("WhitePoint"))
                .unwrap()
                .text()
                .unwrap();

            let primaries: Vec<f64> = [primary_red, primary_green, primary_blue, primary_white]
                .join(&self.separator.to_string())
                .split(self.separator)
                .map(|v| v.parse::<f64>().unwrap())
                .collect();

            ensure!(
                primaries.len() == 8,
                "Primaries + WP should be a total of 8 values"
            );

            let include_target = if self.xml_version >= 0x500 {
                let application_type = target_node
                    .children()
                    .find(|e| e.has_tag_name("ApplicationType"));

                ensure!(
                    application_type.is_some(),
                    format!("XML v5.0+: Missing ApplicationType for Target display ID {id}")
                );

                let application_type = application_type.unwrap().text().unwrap().to_string();

                // Only parse HOME targets
                application_type == "HOME"
            } else {
                true
            };

            if include_target {
                targets.insert(
                    id.clone(),
                    TargetDisplay {
                        id: id.clone(),
                        peak_nits,
                        min_nits,
                        primaries: primaries.try_into().unwrap(),
                    },
                );
            }
        }

        Ok(targets)
    }

    fn parse_level254(&self, video: &Node) -> Option<ExtMetadataBlockLevel254> {
        if let Some(node) = video.descendants().find(|e| e.has_tag_name("Level254")) {
            let dm_mode = if let Some(dmm) = node.children().find(|e| e.has_tag_name("DMMode")) {
                dmm.text().map_or(0, |e| e.parse::<u8>().unwrap())
            } else {
                0
            };

            let dm_version_index =
                if let Some(dmv) = node.children().find(|e| e.has_tag_name("DMVersion")) {
                    dmv.text().map_or(2, |e| e.parse::<u8>().unwrap())
                } else {
                    2
                };

            Some(ExtMetadataBlockLevel254 {
                dm_mode,
                dm_version_index,
            })
        } else {
            // No L254 in the case of CM v2.9
            None
        }
    }

    fn add_level11(&mut self, video: &Node) -> Result<()> {
        if let Some(node) = video.descendants().find(|e| e.has_tag_name("Level11")) {
            let content_type: Option<u8> = if let Some(content_type_node) =
                node.children().find(|e| e.has_tag_name("ContentType"))
            {
                content_type_node.text().map(|e| e.parse::<u8>().unwrap())
            } else {
                None
            };

            let whitepoint: Option<u8> = if let Some(wp_node) = node
                .children()
                .find(|e| e.has_tag_name("IntendedWhitePoint"))
            {
                wp_node.text().map(|e| e.parse::<u8>().unwrap())
            } else {
                None
            };

            if let (Some(content_type), Some(whitepoint)) = (content_type, whitepoint) {
                self.config
                    .default_metadata_blocks
                    .push(ExtMetadataBlock::Level11(ExtMetadataBlockLevel11 {
                        content_type,
                        whitepoint,
                        ..Default::default()
                    }))
            }
        }
        Ok(())
    }

    fn parse_shots(&self, video: &Node) -> Result<Vec<VideoShot>> {
        let shots = video
            .descendants()
            .filter(|e| e.has_tag_name("Shot"))
            .map(|n| {
                let mut shot = VideoShot {
                    id: n
                        .children()
                        .find(|e| e.has_tag_name("UniqueID"))
                        .unwrap()
                        .text()
                        .unwrap()
                        .to_string(),
                    ..Default::default()
                };

                if let Some(record) = n.children().find(|e| e.has_tag_name("Record")) {
                    shot.start = record
                        .children()
                        .find(|e| e.has_tag_name("In"))
                        .unwrap()
                        .text()
                        .unwrap()
                        .parse::<usize>()
                        .unwrap();
                    shot.duration = record
                        .children()
                        .find(|e| e.has_tag_name("Duration"))
                        .unwrap()
                        .text()
                        .unwrap()
                        .parse::<usize>()
                        .unwrap();
                }

                shot.metadata_blocks = self.parse_shot_trims(&n)?;

                let frames = n.children().filter(|e| e.has_tag_name("Frame"));

                for frame in frames {
                    let edit_offset = frame
                        .children()
                        .find(|e| e.has_tag_name("EditOffset"))
                        .unwrap()
                        .text()
                        .unwrap()
                        .parse::<usize>()
                        .unwrap();

                    shot.frame_edits.push(ShotFrameEdit {
                        edit_offset,
                        metadata_blocks: self.parse_shot_trims(&frame)?,
                    });
                }

                Ok(shot)
            })
            .collect();

        shots
    }

    fn parse_shot_trims(&self, node: &Node) -> Result<Vec<ExtMetadataBlock>> {
        let mut metadata_blocks = Vec::new();

        let dynamic_meta_tag = if self.is_cmv4() {
            "DVDynamicData"
        } else {
            "PluginNode"
        };

        if let Some(defaults_node) = node
            .descendants()
            .find(|e| e.has_tag_name(dynamic_meta_tag))
        {
            if self.is_cmv4() {
                let level_nodes = defaults_node
                    .children()
                    .filter(|e| e.has_attribute("level"));

                for level_node in level_nodes {
                    let level = level_node.attribute("level").unwrap();
                    self.parse_trim_levels(&level_node, level, &mut metadata_blocks)?;
                }
            } else {
                let edr_nodes = defaults_node
                    .children()
                    .filter(|e| e.has_tag_name("DolbyEDR") && e.has_attribute("level"));

                for edr in edr_nodes {
                    let level = edr.attribute("level").unwrap();
                    self.parse_trim_levels(&edr, level, &mut metadata_blocks)?;
                }
            };
        }

        Ok(metadata_blocks)
    }

    fn parse_trim_levels(
        &self,
        node: &Node,
        level: &str,
        metadata_blocks: &mut Vec<ExtMetadataBlock>,
    ) -> Result<()> {
        if level == "1" {
            metadata_blocks.push(ExtMetadataBlock::Level1(self.parse_level1_trim(node)?));
        } else if level == "2" {
            metadata_blocks.push(ExtMetadataBlock::Level2(self.parse_level2_trim(node)?));
        } else if level == "3" {
            metadata_blocks.push(ExtMetadataBlock::Level3(self.parse_level3_trim(node)?));
        } else if level == "5" {
            metadata_blocks.push(ExtMetadataBlock::Level5(self.parse_level5_trim(node)?));
        } else if level == "8" {
            metadata_blocks.push(ExtMetadataBlock::Level8(self.parse_level8_trim(node)?));
        } else if level == "9" {
            metadata_blocks.push(ExtMetadataBlock::Level9(self.parse_level9_trim(node)?));
        }

        Ok(())
    }

    pub fn parse_global_level5(&mut self, output: &Node) -> Result<()> {
        let canvas_ar = if let Some(canvas_ar) = output
            .children()
            .find(|e| e.has_tag_name("CanvasAspectRatio"))
        {
            canvas_ar.text().and_then(|v| v.parse::<f32>().ok())
        } else {
            None
        };

        let image_ar = if let Some(image_ar) = output
            .children()
            .find(|e| e.has_tag_name("ImageAspectRatio"))
        {
            image_ar.text().and_then(|v| v.parse::<f32>().ok())
        } else {
            None
        };

        if let (Some(c_ar), Some(i_ar)) = (canvas_ar, image_ar) {
            self.config.level5 = self
                .calculate_level5_metadata(c_ar, i_ar)
                .ok()
                .unwrap_or_default();
        }

        Ok(())
    }

    /// Parse every target display to create L10 metadata if they use custom primaries
    fn parse_global_level10_targets(&mut self) -> Result<()> {
        for (id, target) in &self.target_displays {
            let index = Self::find_primary_index(&target.primaries, false)?;

            let length = if index == 255 { 21 } else { 5 };

            let mut block = ExtMetadataBlockLevel10 {
                length,
                target_display_index: target.id.parse::<u8>().unwrap(),
                target_max_pq: min(
                    4095,
                    (nits_to_pq(target.peak_nits.into()) * 4095.0).round() as u16,
                ),
                target_min_pq: min(4095, (nits_to_pq(target.min_nits) * 4095.0).round() as u16),
                target_primary_index: index,
                ..Default::default()
            };

            if index == 255 {
                let color_primaries = ColorPrimaries::from_array_float(&target.primaries);
                block.set_from_primaries(&color_primaries);
            }

            // Only allow custom L10
            if !PRESET_TARGET_DISPLAYS.contains(&id.parse::<u8>().unwrap()) {
                self.config
                    .default_metadata_blocks
                    .push(ExtMetadataBlock::Level10(block));
            }
        }

        Ok(())
    }

    pub fn parse_level1_trim(&self, node: &Node) -> Result<ExtMetadataBlockLevel1> {
        let measurements = node
            .children()
            .find(|e| e.has_tag_name("ImageCharacter"))
            .unwrap()
            .text()
            .unwrap();
        let measurements: Vec<&str> = measurements.split(self.separator).collect();

        ensure!(
            measurements.len() == 3,
            "invalid L1 trim: should be 3 values"
        );

        let min_pq = (measurements[0].parse::<f32>().unwrap() * 4095.0).round() as u16;
        let avg_pq = (measurements[1].parse::<f32>().unwrap() * 4095.0).round() as u16;
        let max_pq = (measurements[2].parse::<f32>().unwrap() * 4095.0).round() as u16;

        Ok(ExtMetadataBlockLevel1::from_stats_cm_version(
            min_pq,
            max_pq,
            avg_pq,
            self.config.cm_version,
        ))
    }

    pub fn parse_level2_trim(&self, node: &Node) -> Result<ExtMetadataBlockLevel2> {
        let target_id = node
            .children()
            .find(|e| e.has_tag_name("TID"))
            .unwrap()
            .text()
            .unwrap()
            .to_string();

        let trim = node
            .children()
            .find(|e| e.has_tag_name("Trim"))
            .unwrap()
            .text()
            .unwrap();
        let trim: Vec<&str> = trim.split(self.separator).collect();

        let target_display = self
            .target_displays
            .get(&target_id)
            .expect("No target display found for L2 trim");

        ensure!(trim.len() == 9, "invalid L2 trim: should be 9 values");

        let trim_lift = trim[3].parse::<f32>().unwrap();
        let trim_gain = trim[4].parse::<f32>().unwrap();
        let trim_gamma = trim[5].parse::<f32>().unwrap().clamp(-1.0, 1.0);

        let trim_slope = min(
            4095,
            ((((trim_gain + 2.0) * (1.0 - trim_lift / 2.0) - 2.0) * 2048.0) + 2048.0).round()
                as u16,
        );
        let trim_offset = min(
            4095,
            ((((trim_gain + 2.0) * (trim_lift / 2.0)) * 2048.0) + 2048.0).round() as u16,
        );
        let trim_power = min(
            4095,
            (((2.0 / (1.0 + trim_gamma / 2.0) - 2.0) * 2048.0) + 2048.0).round() as u16,
        );
        let trim_chroma_weight = min(
            4095,
            ((trim[6].parse::<f32>().unwrap() * 2048.0) + 2048.0).round() as u16,
        );
        let trim_saturation_gain = min(
            4095,
            ((trim[7].parse::<f32>().unwrap() * 2048.0) + 2048.0).round() as u16,
        );
        let ms_weight = min(
            4095,
            ((trim[8].parse::<f32>().unwrap() * 2048.0) + 2048.0).round() as i16,
        );

        Ok(ExtMetadataBlockLevel2 {
            trim_slope,
            trim_offset,
            trim_power,
            trim_chroma_weight,
            trim_saturation_gain,
            ms_weight,
            ..ExtMetadataBlockLevel2::from_nits(target_display.peak_nits)
        })
    }

    pub fn parse_level3_trim(&self, node: &Node) -> Result<ExtMetadataBlockLevel3> {
        let measurements = node
            .children()
            .find(|e| e.has_tag_name("L1Offset"))
            .unwrap()
            .text()
            .unwrap();

        // [min, avg, max]
        let measurements: Vec<&str> = measurements.split(self.separator).collect();

        ensure!(
            measurements.len() == 3,
            "invalid L3 trim: should be 3 values"
        );

        Ok(ExtMetadataBlockLevel3 {
            min_pq_offset: ((measurements[0].parse::<f32>().unwrap() * 2048.0) + 2048.0).round()
                as u16,
            avg_pq_offset: ((measurements[1].parse::<f32>().unwrap() * 2048.0) + 2048.0).round()
                as u16,
            max_pq_offset: ((measurements[2].parse::<f32>().unwrap() * 2048.0) + 2048.0).round()
                as u16,
        })
    }

    pub fn parse_level5_trim(&self, node: &Node) -> Result<ExtMetadataBlockLevel5> {
        let ratios = node
            .children()
            .find(|e| e.has_tag_name("AspectRatios"))
            .unwrap()
            .text()
            .unwrap();
        let ratios: Vec<&str> = ratios.split(self.separator).collect();

        ensure!(ratios.len() == 2, "invalid L5 trim: should be 2 values");

        let canvas_ar = ratios[0].parse::<f32>().unwrap();
        let image_ar = ratios[1].parse::<f32>().unwrap();

        Ok(self
            .calculate_level5_metadata(canvas_ar, image_ar)
            .ok()
            .unwrap_or_default())
    }

    pub fn parse_level8_trim(&self, node: &Node) -> Result<ExtMetadataBlockLevel8> {
        let target_id = node
            .children()
            .find(|e| e.has_tag_name("TID"))
            .unwrap()
            .text()
            .unwrap()
            .to_string();

        let trim = node
            .children()
            .find(|e| e.has_tag_name("L8Trim"))
            .unwrap()
            .text()
            .unwrap();
        let trim: Vec<&str> = trim.split(self.separator).collect();

        let target_display = self
            .target_displays
            .get(&target_id)
            .expect("No target display found for L8 trim");

        ensure!(trim.len() == 6, "Invalid L8 trim: should be 6 values");

        let trim_lift = trim[0].parse::<f32>().unwrap();
        let trim_gain = trim[1].parse::<f32>().unwrap();
        let trim_gamma = trim[2].parse::<f32>().unwrap().clamp(-1.0, 1.0);

        let trim_slope = min(
            4095,
            ((((trim_gain + 2.0) * (1.0 - trim_lift / 2.0) - 2.0) * 2048.0) + 2048.0).round()
                as u16,
        );
        let trim_offset = min(
            4095,
            ((((trim_gain + 2.0) * (trim_lift / 2.0)) * 2048.0) + 2048.0).round() as u16,
        );
        let trim_power = min(
            4095,
            (((2.0 / (1.0 + trim_gamma / 2.0) - 2.0) * 2048.0) + 2048.0).round() as u16,
        );
        let trim_chroma_weight = min(
            4095,
            ((trim[3].parse::<f32>().unwrap() * 2048.0) + 2048.0).round() as u16,
        );
        let trim_saturation_gain = min(
            4095,
            ((trim[4].parse::<f32>().unwrap() * 2048.0) + 2048.0).round() as u16,
        );
        let ms_weight = min(
            4095,
            ((trim[5].parse::<f32>().unwrap() * 2048.0) + 2048.0).round() as u16,
        );

        let mid_contrast_bias_text = node
            .children()
            .find(|e| e.has_tag_name("MidContrastBias"))
            .unwrap()
            .text()
            .unwrap();

        let highlight_clipping_text = node
            .children()
            .find(|e| e.has_tag_name("HighlightClipping"))
            .unwrap()
            .text()
            .unwrap();

        let target_mid_contrast = min(
            4095,
            ((mid_contrast_bias_text.parse::<f32>().unwrap() * 2048.0) + 2048.0).round() as u16,
        );

        let clip_trim = min(
            4095,
            ((highlight_clipping_text.parse::<f32>().unwrap() * 2048.0) + 2048.0).round() as u16,
        );

        // L8 SaturationVectorField
        let satvec_text = node
            .children()
            .find(|e| e.has_tag_name("SaturationVectorField"))
            .unwrap()
            .text()
            .unwrap();

        let satvec: Vec<u8> = satvec_text
            .split(self.separator)
            .map(|v| {
                min(
                    255,
                    ((v.parse::<f32>().unwrap() * 128.0) + 128.0).round() as u8,
                )
            })
            .collect();

        ensure!(
            satvec.len() == 6,
            "Invalid L8 SatVectorField: should be 6 values"
        );

        // L8 HueVectorField
        let huevec_text = node
            .children()
            .find(|e| e.has_tag_name("HueVectorField"))
            .unwrap()
            .text()
            .unwrap();

        let huevec: Vec<u8> = huevec_text
            .split(self.separator)
            .map(|v| {
                min(
                    255,
                    ((v.parse::<f32>().unwrap() * 128.0) + 128.0).round() as u8,
                )
            })
            .collect();

        ensure!(
            huevec.len() == 6,
            "Invalid L8 HueVectorField: should be 6 values"
        );

        // Set variable length according to the metadata
        // Only write trims which were modified
        let length = if huevec.iter().any(|v| *v != 128) {
            25
        } else if satvec.iter().any(|v| *v != 128) {
            19
        } else if clip_trim != 2048 {
            13
        } else if target_mid_contrast != 2048 {
            12
        } else {
            10
        };

        Ok(ExtMetadataBlockLevel8 {
            length,
            target_display_index: target_display.id.parse::<u8>()?,
            trim_slope,
            trim_offset,
            trim_power,
            trim_chroma_weight,
            trim_saturation_gain,
            ms_weight,
            target_mid_contrast,
            clip_trim,
            saturation_vector_field0: satvec[0],
            saturation_vector_field1: satvec[1],
            saturation_vector_field2: satvec[2],
            saturation_vector_field3: satvec[3],
            saturation_vector_field4: satvec[4],
            saturation_vector_field5: satvec[5],
            hue_vector_field0: huevec[0],
            hue_vector_field1: huevec[1],
            hue_vector_field2: huevec[2],
            hue_vector_field3: huevec[3],
            hue_vector_field4: huevec[4],
            hue_vector_field5: huevec[5],
        })
    }

    fn find_primary_index(primaries: &[f64; 8], check_realdevice: bool) -> Result<u8> {
        // Check PREDEFINED_COLORSPACE_PRIMARIES anyway
        if check_realdevice {
            let primary_index = Self::find_primary_index(primaries, false)?;
            if primary_index < 255 {
                return Ok(primary_index);
            }
        };

        let presets = if check_realdevice {
            level9::PREDEFINED_REALDEVICE_PRIMARIES
        } else {
            primaries::PREDEFINED_COLORSPACE_PRIMARIES
        };

        let matching_primaries = presets.iter().enumerate().find(|(_, preset_primaries)| {
            primaries
                .iter()
                .zip(preset_primaries.iter())
                .all(|(a, b)| (*a - *b).abs() < f64::EPSILON)
        });

        // Exact match to preset primaries
        let primary_index = if let Some((primary_index, _)) = matching_primaries {
            if check_realdevice {
                primary_index + primaries::PREDEFINED_COLORSPACE_PRIMARIES.len()
            } else {
                primary_index
            }
        } else {
            255
        };

        Ok(primary_index as u8)
    }

    /// Mastering display primaries
    fn parse_level9_trim(&self, node: &Node) -> Result<ExtMetadataBlockLevel9> {
        let source_color_primary = node
            .children()
            .find(|e| e.has_tag_name("SourceColorPrimary"))
            .unwrap()
            .text()
            .unwrap();

        let primaries: Vec<f64> = source_color_primary
            .split(self.separator)
            .map(|v| v.parse::<f64>().unwrap())
            .collect();
        ensure!(
            primaries.len() == 8,
            "Invalid L9 SourceColorPrimary: should be 8 values"
        );

        let primaries = primaries.try_into().unwrap();

        let index = Self::find_primary_index(&primaries, true)?;

        let length = if index == 255 { 17 } else { 1 };

        let mut block = ExtMetadataBlockLevel9 {
            length,
            source_primary_index: index,
            ..Default::default()
        };

        if index == 255 {
            let color_primaries = ColorPrimaries::from_array_float(&primaries);
            block.set_from_primaries(&color_primaries);
        }

        Ok(block)
    }

    fn calculate_level5_metadata(
        &self,
        canvas_ar: f32,
        image_ar: f32,
    ) -> Result<ExtMetadataBlockLevel5> {
        ensure!(
            self.opts.canvas_width.is_some(),
            "Missing canvas width to calculate L5"
        );
        ensure!(
            self.opts.canvas_height.is_some(),
            "Missing canvas height to calculate L5"
        );

        let cw = self.opts.canvas_width.unwrap() as f32;
        let ch = self.opts.canvas_height.unwrap() as f32;

        let mut calculated_level5 = ExtMetadataBlockLevel5::default();

        if (canvas_ar - image_ar).abs() < f32::EPSILON {
            // No AR difference, zero offsets
        } else if image_ar > canvas_ar {
            let image_h = (ch * (canvas_ar / image_ar)).round();
            let diff = ch - image_h;
            let offset_top = (diff / 2.0).trunc();
            let offset_bottom = diff - offset_top;

            calculated_level5.active_area_top_offset = offset_top as u16;
            calculated_level5.active_area_bottom_offset = offset_bottom as u16;
        } else {
            let image_w = (cw * (image_ar / canvas_ar)).round();
            let diff = cw - image_w;
            let offset_left = (diff / 2.0).trunc();
            let offset_right = diff - offset_left;

            calculated_level5.active_area_left_offset = offset_left as u16;
            calculated_level5.active_area_right_offset = offset_right as u16;
        }

        Ok(calculated_level5)
    }

    pub fn is_cmv4(&self) -> bool {
        self.xml_version >= 0x402
    }
}
