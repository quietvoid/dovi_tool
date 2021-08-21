use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use crate::dovi::OUT_NAL_HEADER;

use super::DoviRpu;

use super::rpu::{
    rpu_data_header::RpuDataHeader, vdr_dm_data::VdrDmData, vdr_rpu_data::VdrRpuData,
};

pub struct Generator {
    json_path: PathBuf,
    rpu_out: PathBuf,
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
    pub fn generate(json_path: PathBuf, rpu_out: Option<PathBuf>) {
        let out_path = if let Some(out_path) = rpu_out {
            out_path
        } else {
            PathBuf::from("RPU_generated.bin".to_string())
        };

        let generator = Generator {
            json_path,
            rpu_out: out_path,
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

        let encoded_rpu = rpu.write_rpu_data();

        println!("Writing RPU file...");
        let mut writer = BufWriter::with_capacity(
            100_000,
            File::create(&self.rpu_out).expect("Can't create file"),
        );

        for _ in 0..config.length {
            writer.write_all(OUT_NAL_HEADER)?;

            // Remove 0x7C01
            // For some reason there's an extra byte
            writer.write_all(&encoded_rpu[2..])?;
        }

        writer.flush()?;

        Ok(())
    }
}
