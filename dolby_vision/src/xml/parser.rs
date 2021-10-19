use anyhow::{bail, ensure, Result};
use roxmltree::{Document, Node};
use std::cmp::min;
use std::collections::HashMap;

use crate::st2094_10::generate::{
    Level1Metadata, Level2Metadata, Level3Metadata, Level5Metadata, Level6Metadata,
};

#[derive(Default, Debug)]
pub struct CmXmlParser {
    cm_version: String,
    separator: char,

    length: usize,
    target_displays: HashMap<String, TargetDisplay>,
    level5: Option<Level5AspectRatios>,
    level6: Level6Metadata,
    shots: Vec<VideoShot>,
}

#[derive(Default, Debug)]
pub struct TargetDisplay {
    id: String,
    peak_nits: u16,
}

#[derive(Default, Debug)]
pub struct VideoShot {
    pub id: String,
    pub start: usize,
    pub duration: usize,
    pub level1: Option<Vec<Level1Metadata>>,
    pub level2: Option<Vec<Vec<Level2Metadata>>>,
    pub level3: Option<Vec<Level3Metadata>>,
    pub level5: Option<Vec<Level5AspectRatios>>,
}

#[derive(Default, Debug, Clone)]
pub struct Level5AspectRatios {
    pub canvas: f32,
    pub image: f32,
}

#[derive(Debug)]
pub enum DynamicMeta {
    Level1(Level1Metadata),
    Level2(Vec<Level2Metadata>),
    Level3(Level3Metadata),
    Level5(Level5AspectRatios),
}

impl CmXmlParser {
    pub fn new(s: String) -> Result<CmXmlParser> {
        let mut parser = CmXmlParser::default();

        let doc = roxmltree::Document::parse(&s).unwrap();

        parser.cm_version = parser.parse_cm_version(&doc)?;

        parser.separator = if parser.is_cmv4() { ' ' } else { ',' };

        if let Some(output) = doc.descendants().find(|e| e.has_tag_name("Output")) {
            parser.parse_global_level5(&output);

            if let Some(video) = output.descendants().find(|e| e.has_tag_name("Video")) {
                let (max_frame_average_light_level, max_content_light_level) =
                    parser.parse_level6(&video);
                let (min_display_mastering_luminance, max_display_mastering_luminance) =
                    parser.parse_mastering_display_metadata(&video);

                parser.level6 = Level6Metadata {
                    max_display_mastering_luminance,
                    min_display_mastering_luminance,
                    max_content_light_level,
                    max_frame_average_light_level,
                };

                parser.target_displays = parser.parse_target_displays(&video);

                parser.shots = parser.parse_shots(&video)?;
                parser.shots.sort_by_key(|s| s.start);

                let first_shot = parser.shots.first().unwrap();
                let last_shot = parser.shots.last().unwrap();

                parser.length = (last_shot.start + last_shot.duration) - first_shot.start;
            } else {
                bail!("Could not find Video node");
            }
        } else {
            bail!("Could not find Output node");
        }

        Ok(parser)
    }

    fn parse_cm_version(&self, doc: &Document) -> Result<String> {
        if let Some(node) = doc.descendants().find(|e| e.has_tag_name("DolbyLabsMDF")) {
            let version_attr = node.attribute("version");
            let version_node =
                if let Some(version_node) = node.children().find(|e| e.has_tag_name("Version")) {
                    version_node.text()
                } else {
                    None
                };

            let version_level254 = if let Some(level254_node) =
                node.descendants().find(|e| e.has_tag_name("Level254"))
            {
                let cm_version_node = level254_node
                    .children()
                    .find(|e| e.has_tag_name("CMVersion"))
                    .unwrap();
                let cm_version = cm_version_node.text().unwrap();

                if cm_version.contains('4') {
                    Some("4.0.2")
                } else {
                    None
                }
            } else {
                None
            };

            if version_node.is_some() || version_level254.is_some() {
                if let Some(v) = version_node {
                    Ok(v.to_string())
                } else if let Some(v) = version_level254 {
                    Ok(v.to_string())
                } else if let Some(v) = version_attr {
                    Ok(v.to_string())
                } else {
                    bail!("No CM version found!");
                }
            } else if let Some(v) = version_attr {
                Ok(v.to_string())
            } else {
                bail!("No CM version found!");
            }
        } else {
            bail!("Could not find DolbyLabsMDF root node.");
        }
    }

    fn parse_level6(&self, video: &Node) -> (u16, u16) {
        if let Some(node) = video.descendants().find(|e| e.has_tag_name("Level6")) {
            let maxfall = if let Some(fall) = node.children().find(|e| e.has_tag_name("MaxFALL")) {
                fall.text().map_or(0, |e| e.parse::<u16>().unwrap())
            } else {
                0
            };

            let maxcll = if let Some(cll) = node.children().find(|e| e.has_tag_name("MaxCLL")) {
                cll.text().map_or(0, |e| e.parse::<u16>().unwrap())
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

    fn parse_target_displays(&self, video: &Node) -> HashMap<String, TargetDisplay> {
        let mut targets = HashMap::new();

        video
            .descendants()
            .filter(|e| e.has_tag_name("TargetDisplay"))
            .for_each(|e| {
                let id = e
                    .children()
                    .find(|e| e.has_tag_name("ID"))
                    .unwrap()
                    .text()
                    .unwrap()
                    .to_string();

                let peak_nits = e
                    .children()
                    .find(|e| e.has_tag_name("PeakBrightness"))
                    .unwrap()
                    .text()
                    .unwrap()
                    .parse::<u16>()
                    .unwrap();

                targets.insert(id.clone(), TargetDisplay { id, peak_nits });
            });

        targets
    }

    fn parse_shots(&self, video: &Node) -> Result<Vec<VideoShot>> {
        let shots = video
            .descendants()
            .filter(|e| e.has_tag_name("Shot"))
            .map(|n| {
                let mut shot = VideoShot::default();

                shot.id = n
                    .children()
                    .find(|e| e.has_tag_name("UniqueID"))
                    .unwrap()
                    .text()
                    .unwrap()
                    .to_string();

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

                let trims = self.parse_shot_trims(&n)?;

                let mut l1_list = if let Some(Some(DynamicMeta::Level1(l1))) = trims.get("1") {
                    let mut list: Vec<Level1Metadata> = Vec::new();

                    list.resize(shot.duration, l1.clone());

                    Some(list)
                } else {
                    None
                };

                let mut l2_list = if let Some(Some(DynamicMeta::Level2(l2))) = trims.get("2") {
                    let mut list: Vec<Vec<Level2Metadata>> = Vec::new();

                    list.resize(shot.duration, l2.clone());

                    Some(list)
                } else {
                    None
                };

                let mut l3_list = if let Some(Some(DynamicMeta::Level3(l3))) = trims.get("3") {
                    let mut list: Vec<Level3Metadata> = Vec::new();

                    list.resize(shot.duration, l3.clone());

                    Some(list)
                } else {
                    None
                };

                let mut l5_list = if let Some(Some(DynamicMeta::Level5(l5))) = trims.get("5") {
                    let mut list: Vec<Level5AspectRatios> = Vec::new();

                    list.resize(shot.duration, l5.clone());

                    Some(list)
                } else {
                    None
                };

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

                    let trims = self.parse_shot_trims(&frame)?;

                    if let Some(list) = &mut l1_list {
                        ensure!(edit_offset < list.len());

                        if let Some(Some(DynamicMeta::Level1(new_l1))) = trims.get("1") {
                            list[edit_offset] = new_l1.clone();
                        }
                    }

                    if let Some(list) = &mut l2_list {
                        ensure!(edit_offset < list.len());

                        if let Some(Some(DynamicMeta::Level2(new_l2))) = trims.get("2") {
                            list[edit_offset] = new_l2.clone();
                        }
                    }

                    if let Some(list) = &mut l3_list {
                        ensure!(edit_offset < list.len());

                        if let Some(Some(DynamicMeta::Level3(new_l3))) = trims.get("3") {
                            list[edit_offset] = new_l3.clone();
                        }
                    }

                    if let Some(list) = &mut l5_list {
                        ensure!(edit_offset < list.len());

                        if let Some(Some(DynamicMeta::Level5(new_l5))) = trims.get("5") {
                            list[edit_offset] = new_l5.clone();
                        }
                    }
                }

                shot.level1 = l1_list;
                shot.level2 = l2_list;
                shot.level3 = l3_list;
                shot.level5 = l5_list;

                Ok(shot)
            })
            .collect();

        shots
    }

    fn parse_shot_trims(&self, node: &Node) -> Result<HashMap<&str, Option<DynamicMeta>>> {
        let mut trims = HashMap::new();

        let dynamic_meta_tag = if self.is_cmv4() {
            "DVDynamicData"
        } else {
            "PluginNode"
        };

        if let Some(defaults_node) = node
            .descendants()
            .find(|e| e.has_tag_name(dynamic_meta_tag))
        {
            let mut default_l1 = None;
            let mut default_l2 = Some(Vec::new());
            let mut default_l3 = None;
            let mut default_l5 = None;

            if self.is_cmv4() {
                let level_nodes = defaults_node
                    .children()
                    .filter(|e| e.has_attribute("level"));

                for level_node in level_nodes {
                    let level = level_node.attribute("level").unwrap();
                    let (level1, level3, level5) =
                        self.parse_trim_levels(&level_node, level, &mut default_l2)?;

                    // Only replace if the shot has a default trim
                    if let Some(l1) = level1 {
                        default_l1 = Some(l1);
                    }

                    if let Some(l3) = level3 {
                        default_l3 = Some(l3);
                    }

                    if let Some(l5) = level5 {
                        default_l5 = Some(l5);
                    }
                }
            } else {
                let edr_nodes = defaults_node
                    .children()
                    .filter(|e| e.has_tag_name("DolbyEDR") && e.has_attribute("level"));

                for edr in edr_nodes {
                    let level = edr.attribute("level").unwrap();
                    let (level1, level3, level5) =
                        self.parse_trim_levels(&edr, level, &mut default_l2)?;

                    // Only replace if the shot has a default trim
                    if let Some(l1) = level1 {
                        default_l1 = Some(l1);
                    }

                    if let Some(l3) = level3 {
                        default_l3 = Some(l3);
                    }

                    if let Some(l5) = level5 {
                        default_l5 = Some(l5);
                    }
                }
            };

            if let Some(level1) = default_l1 {
                trims.insert("1", Some(DynamicMeta::Level1(level1)));
            }

            if let Some(level2) = default_l2 {
                if !level2.is_empty() {
                    trims.insert("2", Some(DynamicMeta::Level2(level2)));
                }
            }

            if let Some(level3) = default_l3 {
                trims.insert("3", Some(DynamicMeta::Level3(level3)));
            }

            if let Some(level5) = default_l5 {
                trims.insert("5", Some(DynamicMeta::Level5(level5)));
            }
        }

        Ok(trims)
    }

    fn parse_trim_levels(
        &self,
        node: &Node,
        level: &str,
        mut default_l2: &mut Option<Vec<Level2Metadata>>,
    ) -> Result<(
        Option<Level1Metadata>,
        Option<Level3Metadata>,
        Option<Level5AspectRatios>,
    )> {
        let mut default_l1 = None;
        let mut default_l3 = None;
        let mut default_l5 = None;

        if level == "1" {
            let level1 = self.parse_level1_trim(node)?;
            default_l1 = Some(level1);
        } else if level == "2" {
            let level2 = self.parse_level2_trim(node)?;

            if let Some(l2_list) = &mut default_l2 {
                l2_list.push(level2);
            }
        } else if level == "3" {
            let level3 = self.parse_level3_trim(node)?;
            default_l3 = Some(level3);
        } else if level == "5" {
            let level5 = self.parse_level5_trim(node)?;
            default_l5 = Some(level5);
        }

        Ok((default_l1, default_l3, default_l5))
    }

    pub fn parse_global_level5(&mut self, output: &Node) {
        let canvas_ar = if let Some(canvas_ar) = output
            .children()
            .find(|e| e.has_tag_name("CanvasAspectRatio"))
        {
            canvas_ar.text().map_or(None, |v| v.parse::<f32>().ok())
        } else {
            None
        };

        let image_ar = if let Some(image_ar) = output
            .children()
            .find(|e| e.has_tag_name("ImageAspectRatio"))
        {
            image_ar.text().map_or(None, |v| v.parse::<f32>().ok())
        } else {
            None
        };

        if canvas_ar.is_some() && image_ar.is_some() {
            self.level5 = Some(Level5AspectRatios {
                canvas: canvas_ar.unwrap(),
                image: image_ar.unwrap(),
            });
        }
    }

    pub fn parse_level1_trim(&self, node: &Node) -> Result<Level1Metadata> {
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

        Ok(Level1Metadata {
            min_pq: (measurements[0].parse::<f32>().unwrap() * 4095.0).round() as u16,
            avg_pq: (measurements[1].parse::<f32>().unwrap() * 4095.0).round() as u16,
            max_pq: (measurements[2].parse::<f32>().unwrap() * 4095.0).round() as u16,
        })
    }

    pub fn parse_level2_trim(&self, node: &Node) -> Result<Level2Metadata> {
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

        let target_display = self.target_displays.get(&target_id).unwrap();

        ensure!(trim.len() == 9, "invalid L2 trim: should be 9 values");

        let trim_slope = min(
            4095,
            ((trim[4].parse::<f32>().unwrap() * 2048.0) + 2048.0).round() as u16,
        );
        let trim_offset = min(
            4095,
            ((trim[3].parse::<f32>().unwrap() * 2048.0) + 2048.0).round() as u16,
        );
        let trim_power = min(
            4095,
            ((trim[5].parse::<f32>().unwrap() * -2048.0) + 2048.0).round() as u16,
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

        Ok(Level2Metadata {
            target_nits: Some(target_display.peak_nits),
            trim_slope,
            trim_offset,
            trim_power,
            trim_chroma_weight,
            trim_saturation_gain,
            ms_weight,
            ..Default::default()
        })
    }

    pub fn parse_level3_trim(&self, node: &Node) -> Result<Level3Metadata> {
        let measurements = node
            .children()
            .find(|e| e.has_tag_name("L1Offset"))
            .unwrap()
            .text()
            .unwrap();
        let measurements: Vec<&str> = measurements.split(self.separator).collect();

        ensure!(
            measurements.len() == 3,
            "invalid L3 trim: should be 3 values"
        );

        Ok(Level3Metadata {
            min_pq_offset: ((measurements[0].parse::<f32>().unwrap() * 2048.0) + 2048.0).round()
                as u16,
            max_pq_offset: ((measurements[1].parse::<f32>().unwrap() * 2048.0) + 2048.0).round()
                as u16,
            avg_pq_offset: ((measurements[2].parse::<f32>().unwrap() * 2048.0) + 2048.0).round()
                as u16,
        })
    }

    pub fn parse_level5_trim(&self, node: &Node) -> Result<Level5AspectRatios> {
        let ratios = node
            .children()
            .find(|e| e.has_tag_name("AspectRatios"))
            .unwrap()
            .text()
            .unwrap();
        let ratios: Vec<&str> = ratios.split(self.separator).collect();

        ensure!(ratios.len() == 2, "invalid L5 trim: should be 2 values");

        Ok(Level5AspectRatios {
            canvas: ratios[0].parse::<f32>().unwrap(),
            image: ratios[1].parse::<f32>().unwrap(),
        })
    }

    pub fn calculate_level5_metadata(
        ar: &Level5AspectRatios,
        canvas_width: u16,
        canvas_height: u16,
    ) -> Option<Level5Metadata> {
        let cw = canvas_width as f32;
        let ch = canvas_height as f32;

        if (ar.canvas - ar.image).abs() < f32::EPSILON {
            None
        } else {
            let mut calculated_level5 = Level5Metadata::default();

            if ar.image > ar.canvas {
                let image_h = (ch * (ar.canvas / ar.image)).round();
                let diff = ch - image_h;
                let offset_top = (diff / 2.0).trunc();
                let offset_bottom = diff - offset_top;

                calculated_level5.active_area_top_offset = offset_top as u16;
                calculated_level5.active_area_bottom_offset = offset_bottom as u16;
            } else {
                let image_w = (cw * (ar.image / ar.canvas)).round();
                let diff = cw - image_w;
                let offset_left = (diff / 2.0).trunc();
                let offset_right = diff - offset_left;

                calculated_level5.active_area_left_offset = offset_left as u16;
                calculated_level5.active_area_right_offset = offset_right as u16;
            }

            Some(calculated_level5)
        }
    }

    pub fn get_video_length(&self) -> usize {
        self.length
    }

    pub fn get_hdr10_metadata(&self) -> &Level6Metadata {
        &self.level6
    }

    pub fn get_shots(&self) -> &Vec<VideoShot> {
        &self.shots
    }

    pub fn get_global_aspect_ratios(&self) -> Option<&Level5AspectRatios> {
        self.level5.as_ref()
    }

    pub fn get_global_level5(
        &self,
        canvas_width: u16,
        canvas_height: u16,
    ) -> Option<Level5Metadata> {
        if let Some(ar) = &self.level5 {
            Self::calculate_level5_metadata(ar, canvas_width, canvas_height)
        } else {
            None
        }
    }

    pub fn is_cmv4(&self) -> bool {
        self.cm_version == "4.0.2"
    }
}
