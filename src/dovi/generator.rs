use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::PathBuf;

use crate::dovi::rpu::nits_to_pq;
use crate::dovi::OUT_NAL_HEADER;

use super::DoviRpu;

use super::rpu::{
    rpu_data_header::RpuDataHeader, vdr_dm_data::VdrDmData, vdr_rpu_data::VdrRpuData,
};

pub struct Generator {
    json_path: PathBuf,
    rpu_out: PathBuf,
    hdr10plus_path: Option<PathBuf>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct GenerateConfig {
    pub length: u64,
    pub target_nits: u16,

    #[serde(default)]
    pub source_min_pq: Option<u16>,

    #[serde(default)]
    pub source_max_pq: Option<u16>,

    pub level5: Option<Level5Metadata>,
    pub level6: Option<Level6Metadata>,
}

pub struct Level1Metadata {
    pub min_pq: u16,
    pub max_pq: u16,
    pub avg_pq: u16,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Level5Metadata {
    pub active_area_left_offset: u16,
    pub active_area_right_offset: u16,
    pub active_area_top_offset: u16,
    pub active_area_bottom_offset: u16,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Level6Metadata {
    pub max_display_mastering_luminance: u16,
    pub min_display_mastering_luminance: u16,
    pub max_content_light_level: u16,
    pub max_frame_average_light_level: u16,
}

impl Generator {
    pub fn generate(json_path: PathBuf, rpu_out: Option<PathBuf>, hdr10plus_path: Option<PathBuf>) {
        let out_path = if let Some(out_path) = rpu_out {
            out_path
        } else {
            PathBuf::from("RPU_generated.bin".to_string())
        };

        let generator = Generator {
            json_path,
            rpu_out: out_path,
            hdr10plus_path,
        };

        let json_file = File::open(&generator.json_path).unwrap();
        let config: GenerateConfig = serde_json::from_reader(&json_file).unwrap();

        println!("{:#?}", config);

        if let Err(res) = generator.execute(&config) {
            panic!("{:?}", res);
        }

        println!("Done.")
    }

    fn execute(&self, config: &GenerateConfig) -> Result<(), std::io::Error> {
        println!("Generating metadata...");

        let mut l1_meta: Option<Vec<Level1Metadata>> = None;

        if let Some(path) = &self.hdr10plus_path {
            let mut s = String::new();
            File::open(path).unwrap().read_to_string(&mut s).unwrap();

            let hdr10plus: Value = serde_json::from_str(&s).unwrap();

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

                                let max_rgb =
                                    maxscl.iter().filter_map(|e| e.as_u64()).max().unwrap();

                                Level1Metadata {
                                    min_pq: 0,
                                    max_pq: (nits_to_pq((max_rgb as f64 / 10.0).round() as u16)
                                        * 4095.0)
                                        .round() as u16,
                                    avg_pq: (nits_to_pq((avg_rgb as f64 / 10.0).round() as u16)
                                        * 4095.0)
                                        .round() as u16,
                                }
                            })
                            .collect();

                        l1_meta = Some(info_list)
                    }
                }
            }
        }

        println!("Writing RPU file...");
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
            let mut rpu = DoviRpu {
                dovi_profile: 8,
                modified: true,
                header: RpuDataHeader::p8_default(),
                vdr_rpu_data: Some(VdrRpuData::p8_default()),
                nlq_data: None,
                vdr_dm_data: Some(VdrDmData::from_config(config)),
                last_byte: 0x80,
                ..Default::default()
            };

            let encoded_rpu = if let Some(l1_list) = &l1_meta {
                if let Some(meta) = &l1_list.get(i) {
                    if let Some(dm_meta) = &mut rpu.vdr_dm_data {
                        dm_meta.add_level1_metadata(meta.min_pq, meta.max_pq, meta.avg_pq);
                    }
                }

                rpu.write_rpu_data()
            } else {
                rpu.write_rpu_data()
            };

            writer.write_all(OUT_NAL_HEADER)?;

            // Remove 0x7C01
            // For some reason there's an extra byte
            writer.write_all(&encoded_rpu[2..])?;
        }

        writer.flush()?;

        Ok(())
    }
}
