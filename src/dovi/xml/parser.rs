use roxmltree::{Document, Node};
use std::cmp::min;
use std::collections::HashMap;

use crate::dovi::generator::{
    Level1Metadata, Level2Metadata, Level3Metadata, Level6Metadata,
};

#[derive(Default, Debug)]
pub struct CmXmlParser {
    cm_version: String,
    separator: char,

    length: usize,
    target_displays: HashMap<String, TargetDisplay>,
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
}

#[derive(Debug)]
pub enum DynamicMeta {
    Level1(Level1Metadata),
    Level2(Vec<Level2Metadata>),
    Level3(Level3Metadata),
}

impl CmXmlParser {
    pub fn new(s: String) -> CmXmlParser {
        let mut parser = CmXmlParser::default();

        let doc = roxmltree::Document::parse(&s).unwrap();

        parser.cm_version = parser.parse_cm_version(&doc);

        parser.separator = if parser.is_cmv4() { ' ' } else { ',' };

        if let Some(video) = doc.descendants().find(|e| e.has_tag_name("Video")) {
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

            parser.shots = parser.parse_shots(&video);
            parser.shots.sort_by_key(|s| s.start);

            let first_shot = parser.shots.first().unwrap();
            let last_shot = parser.shots.last().unwrap();

            parser.length = (last_shot.start + last_shot.duration) - first_shot.start;
        } else {
            panic!("Could not find Video node");
        }

        parser
    }

    fn parse_cm_version(&self, doc: &Document) -> String {
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
                    v.to_string()
                } else if let Some(v) = version_level254 {
                    v.to_string()
                } else if let Some(v) = version_attr {
                    v.to_string()
                } else {
                    panic!("No CM version found!")
                }
            } else if let Some(v) = version_attr {
                v.to_string()
            } else {
                panic!("No CM version found!")
            }
        } else {
            panic!("Could not find DolbyLabsMDF root node.");
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

    fn parse_shots(&self, video: &Node) -> Vec<VideoShot> {
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

                let trims = self.parse_shot_trims(&n);

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

                n.children()
                    .filter(|e| e.has_tag_name("Frame"))
                    .for_each(|frame| {
                        let edit_offset = frame
                            .children()
                            .find(|e| e.has_tag_name("EditOffset"))
                            .unwrap()
                            .text()
                            .unwrap()
                            .parse::<usize>()
                            .unwrap();

                        let trims = self.parse_shot_trims(&frame);

                        if let Some(list) = &mut l1_list {
                            assert!(edit_offset < list.len());

                            if let Some(Some(DynamicMeta::Level1(new_l1))) = trims.get("1") {
                                list[edit_offset] = new_l1.clone();
                            }
                        }

                        if let Some(list) = &mut l2_list {
                            assert!(edit_offset < list.len());

                            if let Some(Some(DynamicMeta::Level2(new_l2))) = trims.get("2") {
                                list[edit_offset] = new_l2.clone();
                            }
                        }

                        if let Some(list) = &mut l3_list {
                            assert!(edit_offset < list.len());

                            if let Some(Some(DynamicMeta::Level3(new_l3))) = trims.get("3") {
                                list[edit_offset] = new_l3.clone();
                            }
                        }
                    });

                shot.level1 = l1_list;
                shot.level2 = l2_list;
                shot.level3 = l3_list;

                shot
            })
            .collect();

        shots
    }

    fn parse_shot_trims(&self, node: &Node) -> HashMap<&str, Option<DynamicMeta>> {
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

            if self.is_cmv4() {
                defaults_node
                    .children()
                    .filter(|e| e.has_attribute("level"))
                    .for_each(|level_node| {
                        let level = level_node.attribute("level").unwrap();
                        let (level1, level3) =
                            self.parse_trim_levels(&level_node, level, &mut default_l2);

                        if let Some(l1) = level1 {
                            default_l1 = Some(l1);
                        }

                        if let Some(l3) = level3 {
                            default_l3 = Some(l3);
                        }
                    });
            } else {
                defaults_node
                    .children()
                    .filter(|e| e.has_tag_name("DolbyEDR") && e.has_attribute("level"))
                    .for_each(|edr| {
                        let level = edr.attribute("level").unwrap();
                        let (level1, level3) = self.parse_trim_levels(&edr, level, &mut default_l2);

                        if let Some(l1) = level1 {
                            default_l1 = Some(l1);
                        }

                        if let Some(l3) = level3 {
                            default_l3 = Some(l3);
                        }
                    });
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
        }

        trims
    }

    fn parse_trim_levels(
        &self,
        node: &Node,
        level: &str,
        mut default_l2: &mut Option<Vec<Level2Metadata>>,
    ) -> (Option<Level1Metadata>, Option<Level3Metadata>) {
        let mut default_l1 = None;
        let mut default_l3 = None;

        if level == "1" {
            let level1 = self.parse_level1_trim(node);
            default_l1 = Some(level1);
        } else if level == "2" {
            let level2 = self.parse_level2_trim(node);

            if let Some(l2_list) = &mut default_l2 {
                l2_list.push(level2);
            }
        } else if level == "3" {
            let level3 = self.parse_level3_trim(node);
            default_l3 = Some(level3);
        }

        (default_l1, default_l3)
    }

    pub fn parse_level1_trim(&self, node: &Node) -> Level1Metadata {
        let measurements = node
            .children()
            .find(|e| e.has_tag_name("ImageCharacter"))
            .unwrap()
            .text()
            .unwrap();
        let measurements: Vec<&str> = measurements.split(self.separator).collect();

        assert!(measurements.len() == 3);

        Level1Metadata {
            min_pq: (measurements[0].parse::<f32>().unwrap() * 4095.0).round() as u16,
            avg_pq: (measurements[1].parse::<f32>().unwrap() * 4095.0).round() as u16,
            max_pq: (measurements[2].parse::<f32>().unwrap() * 4095.0).round() as u16,
        }
    }

    pub fn parse_level2_trim(&self, node: &Node) -> Level2Metadata {
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

        assert!(trim.len() == 9);

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

        Level2Metadata {
            target_nits: target_display.peak_nits,
            trim_slope,
            trim_offset,
            trim_power,
            trim_chroma_weight,
            trim_saturation_gain,
            ms_weight,
        }
    }

    pub fn parse_level3_trim(&self, node: &Node) -> Level3Metadata {
        let measurements = node
            .children()
            .find(|e| e.has_tag_name("L1Offset"))
            .unwrap()
            .text()
            .unwrap();
        let measurements: Vec<&str> = measurements.split(self.separator).collect();

        assert!(measurements.len() == 3);

        Level3Metadata {
            min_pq_offset: ((measurements[0].parse::<f32>().unwrap() * 2048.0) + 2048.0).round()
                as u16,
            max_pq_offset: ((measurements[1].parse::<f32>().unwrap() * 2048.0) + 2048.0).round()
                as u16,
            avg_pq_offset: ((measurements[2].parse::<f32>().unwrap() * 2048.0) + 2048.0).round()
                as u16,
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

    pub fn is_cmv4(&self) -> bool {
        self.cm_version == "4.0.2"
    }
}
