use anyhow::{ensure, Result};
use bitvec_helpers::{bitvec_reader::BitVecReader, bitvec_writer::BitVecWriter};

#[cfg(feature = "serde_feature")]
use serde::Serialize;

use crate::st2094_10::{
    ext_metadata_blocks::ExtMetadataBlock, generate::GenerateConfig, ST2094_10Meta,
};

#[derive(Debug, Default)]
#[cfg_attr(feature = "serde_feature", derive(Serialize))]
pub struct VdrDmData {
    affected_dm_metadata_id: u64,
    current_dm_metadata_id: u64,
    scene_refresh_flag: u64,
    ycc_to_rgb_coef0: i16,
    ycc_to_rgb_coef1: i16,
    ycc_to_rgb_coef2: i16,
    ycc_to_rgb_coef3: i16,
    ycc_to_rgb_coef4: i16,
    ycc_to_rgb_coef5: i16,
    ycc_to_rgb_coef6: i16,
    ycc_to_rgb_coef7: i16,
    ycc_to_rgb_coef8: i16,
    ycc_to_rgb_offset0: u32,
    ycc_to_rgb_offset1: u32,
    ycc_to_rgb_offset2: u32,
    rgb_to_lms_coef0: i16,
    rgb_to_lms_coef1: i16,
    rgb_to_lms_coef2: i16,
    rgb_to_lms_coef3: i16,
    rgb_to_lms_coef4: i16,
    rgb_to_lms_coef5: i16,
    rgb_to_lms_coef6: i16,
    rgb_to_lms_coef7: i16,
    rgb_to_lms_coef8: i16,
    signal_eotf: u16,
    signal_eotf_param0: u16,
    signal_eotf_param1: u16,
    signal_eotf_param2: u32,
    signal_bit_depth: u8,
    signal_color_space: u8,
    signal_chroma_format: u8,
    signal_full_range_flag: u8,
    pub source_min_pq: u16,
    pub source_max_pq: u16,
    source_diagonal: u16,

    #[serde(flatten)]
    pub st2094_10_metadata: ST2094_10Meta,
}

impl VdrDmData {
    pub fn parse(reader: &mut BitVecReader) -> Result<VdrDmData> {
        let data = VdrDmData {
            affected_dm_metadata_id: reader.get_ue(),
            current_dm_metadata_id: reader.get_ue(),
            scene_refresh_flag: reader.get_ue(),

            ycc_to_rgb_coef0: reader.get_n::<u16>(16) as i16,
            ycc_to_rgb_coef1: reader.get_n::<u16>(16) as i16,
            ycc_to_rgb_coef2: reader.get_n::<u16>(16) as i16,
            ycc_to_rgb_coef3: reader.get_n::<u16>(16) as i16,
            ycc_to_rgb_coef4: reader.get_n::<u16>(16) as i16,
            ycc_to_rgb_coef5: reader.get_n::<u16>(16) as i16,
            ycc_to_rgb_coef6: reader.get_n::<u16>(16) as i16,
            ycc_to_rgb_coef7: reader.get_n::<u16>(16) as i16,
            ycc_to_rgb_coef8: reader.get_n::<u16>(16) as i16,
            ycc_to_rgb_offset0: reader.get_n(32),
            ycc_to_rgb_offset1: reader.get_n(32),
            ycc_to_rgb_offset2: reader.get_n(32),

            rgb_to_lms_coef0: reader.get_n::<u16>(16) as i16,
            rgb_to_lms_coef1: reader.get_n::<u16>(16) as i16,
            rgb_to_lms_coef2: reader.get_n::<u16>(16) as i16,
            rgb_to_lms_coef3: reader.get_n::<u16>(16) as i16,
            rgb_to_lms_coef4: reader.get_n::<u16>(16) as i16,
            rgb_to_lms_coef5: reader.get_n::<u16>(16) as i16,
            rgb_to_lms_coef6: reader.get_n::<u16>(16) as i16,
            rgb_to_lms_coef7: reader.get_n::<u16>(16) as i16,
            rgb_to_lms_coef8: reader.get_n::<u16>(16) as i16,

            signal_eotf: reader.get_n(16),
            signal_eotf_param0: reader.get_n(16),
            signal_eotf_param1: reader.get_n(16),
            signal_eotf_param2: reader.get_n(32),
            signal_bit_depth: reader.get_n(5),
            signal_color_space: reader.get_n(2),
            signal_chroma_format: reader.get_n(2),
            signal_full_range_flag: reader.get_n(2),
            source_min_pq: reader.get_n(12),
            source_max_pq: reader.get_n(12),
            source_diagonal: reader.get_n(10),
            st2094_10_metadata: ST2094_10Meta::parse(reader)?,
        };

        Ok(data)
    }

    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.affected_dm_metadata_id <= 15,
            "affected_dm_metadata_id should be <= 15"
        );
        ensure!(
            self.signal_bit_depth >= 8 && self.signal_bit_depth <= 16,
            "signal_bit_depth should be between 8 and 16"
        );

        if self.signal_eotf_param0 == 0
            && self.signal_eotf_param1 == 0
            && self.signal_eotf_param2 == 0
        {
            ensure!(self.signal_eotf == 65535, "signal_eotf should be 65535");
        }

        Ok(())
    }

    pub fn write(&self, writer: &mut BitVecWriter) {
        writer.write_ue(self.affected_dm_metadata_id);
        writer.write_ue(self.current_dm_metadata_id);
        writer.write_ue(self.scene_refresh_flag);

        writer.write_n(&self.ycc_to_rgb_coef0.to_be_bytes(), 16);
        writer.write_n(&self.ycc_to_rgb_coef1.to_be_bytes(), 16);
        writer.write_n(&self.ycc_to_rgb_coef2.to_be_bytes(), 16);
        writer.write_n(&self.ycc_to_rgb_coef3.to_be_bytes(), 16);
        writer.write_n(&self.ycc_to_rgb_coef4.to_be_bytes(), 16);
        writer.write_n(&self.ycc_to_rgb_coef5.to_be_bytes(), 16);
        writer.write_n(&self.ycc_to_rgb_coef6.to_be_bytes(), 16);
        writer.write_n(&self.ycc_to_rgb_coef7.to_be_bytes(), 16);
        writer.write_n(&self.ycc_to_rgb_coef8.to_be_bytes(), 16);

        writer.write_n(&self.ycc_to_rgb_offset0.to_be_bytes(), 32);
        writer.write_n(&self.ycc_to_rgb_offset1.to_be_bytes(), 32);
        writer.write_n(&self.ycc_to_rgb_offset2.to_be_bytes(), 32);

        writer.write_n(&self.rgb_to_lms_coef0.to_be_bytes(), 16);
        writer.write_n(&self.rgb_to_lms_coef1.to_be_bytes(), 16);
        writer.write_n(&self.rgb_to_lms_coef2.to_be_bytes(), 16);
        writer.write_n(&self.rgb_to_lms_coef3.to_be_bytes(), 16);
        writer.write_n(&self.rgb_to_lms_coef4.to_be_bytes(), 16);
        writer.write_n(&self.rgb_to_lms_coef5.to_be_bytes(), 16);
        writer.write_n(&self.rgb_to_lms_coef6.to_be_bytes(), 16);
        writer.write_n(&self.rgb_to_lms_coef7.to_be_bytes(), 16);
        writer.write_n(&self.rgb_to_lms_coef8.to_be_bytes(), 16);

        writer.write_n(&self.signal_eotf.to_be_bytes(), 16);
        writer.write_n(&self.signal_eotf_param0.to_be_bytes(), 16);
        writer.write_n(&self.signal_eotf_param1.to_be_bytes(), 16);
        writer.write_n(&self.signal_eotf_param2.to_be_bytes(), 32);

        writer.write_n(&self.signal_bit_depth.to_be_bytes(), 5);
        writer.write_n(&self.signal_color_space.to_be_bytes(), 2);
        writer.write_n(&self.signal_chroma_format.to_be_bytes(), 2);
        writer.write_n(&self.signal_full_range_flag.to_be_bytes(), 2);

        writer.write_n(&self.source_min_pq.to_be_bytes(), 12);
        writer.write_n(&self.source_max_pq.to_be_bytes(), 12);
        writer.write_n(&self.source_diagonal.to_be_bytes(), 10);

        self.st2094_10_metadata.write(writer);
    }

    pub fn p5_to_p81(&mut self) {
        self.ycc_to_rgb_coef0 = 9574;
        self.ycc_to_rgb_coef1 = 0;
        self.ycc_to_rgb_coef2 = 13802;
        self.ycc_to_rgb_coef3 = 9574;
        self.ycc_to_rgb_coef4 = -1540;
        self.ycc_to_rgb_coef5 = -5348;
        self.ycc_to_rgb_coef6 = 9574;
        self.ycc_to_rgb_coef7 = 17610;
        self.ycc_to_rgb_coef8 = 0;
        self.ycc_to_rgb_offset0 = 16777216;
        self.ycc_to_rgb_offset1 = 134217728;
        self.ycc_to_rgb_offset2 = 134217728;

        self.rgb_to_lms_coef0 = 7222;
        self.rgb_to_lms_coef1 = 8771;
        self.rgb_to_lms_coef2 = 390;
        self.rgb_to_lms_coef3 = 2654;
        self.rgb_to_lms_coef4 = 12430;
        self.rgb_to_lms_coef5 = 1300;
        self.rgb_to_lms_coef6 = 0;
        self.rgb_to_lms_coef7 = 422;
        self.rgb_to_lms_coef8 = 15962;

        self.signal_color_space = 0;
    }

    // Source PQ means the mastering display
    // MDL 1000,1-10 = 7,3079
    // MDL 4000,50   = 62,3696
    pub fn change_source_levels(&mut self, min_pq: Option<u16>, max_pq: Option<u16>) {
        if let Some(v) = min_pq {
            self.source_min_pq = v;
        }

        if let Some(v) = max_pq {
            self.source_max_pq = v;
        }
    }

    pub fn set_scene_cut(&mut self, is_scene_cut: bool) {
        self.scene_refresh_flag = is_scene_cut as u64;
    }

    pub fn from_config(config: &GenerateConfig) -> VdrDmData {
        let mut vdr_dm_data = VdrDmData {
            affected_dm_metadata_id: 0,
            current_dm_metadata_id: 0,
            scene_refresh_flag: 0,
            ycc_to_rgb_coef0: 9574,
            ycc_to_rgb_coef1: 0,
            ycc_to_rgb_coef2: 13802,
            ycc_to_rgb_coef3: 9574,
            ycc_to_rgb_coef4: -1540,
            ycc_to_rgb_coef5: -5348,
            ycc_to_rgb_coef6: 9574,
            ycc_to_rgb_coef7: 17610,
            ycc_to_rgb_coef8: 0,
            ycc_to_rgb_offset0: 16777216,
            ycc_to_rgb_offset1: 134217728,
            ycc_to_rgb_offset2: 134217728,
            rgb_to_lms_coef0: 7222,
            rgb_to_lms_coef1: 8771,
            rgb_to_lms_coef2: 390,
            rgb_to_lms_coef3: 2654,
            rgb_to_lms_coef4: 12430,
            rgb_to_lms_coef5: 1300,
            rgb_to_lms_coef6: 0,
            rgb_to_lms_coef7: 422,
            rgb_to_lms_coef8: 15962,
            signal_eotf: 65535,
            signal_eotf_param0: 0,
            signal_eotf_param1: 0,
            signal_eotf_param2: 0,
            signal_bit_depth: 12,
            signal_color_space: 0,
            signal_chroma_format: 0,
            signal_full_range_flag: 1,
            source_diagonal: 42,
            ..Default::default()
        };

        vdr_dm_data.change_source_levels(config.source_min_pq, config.source_max_pq);

        vdr_dm_data.st2094_10_metadata.update_from_config(config);
        vdr_dm_data.update_source_levels();

        vdr_dm_data
    }

    pub fn update_source_levels(&mut self) {
        let level6_block = self
            .st2094_10_metadata
            .ext_metadata_blocks
            .iter()
            .find(|e| matches!(e, ExtMetadataBlock::Level6(_)));

        if let Some(ExtMetadataBlock::Level6(m)) = level6_block {
            let mdl_min = m.min_display_mastering_luminance;
            let mdl_max = m.max_display_mastering_luminance;

            // Adjust source by MDL if not set
            if mdl_min > 0 && self.source_min_pq == 0 {
                self.source_min_pq = if mdl_min <= 10 {
                    7
                } else if mdl_min == 50 {
                    62
                } else {
                    0
                };
            }

            if self.source_max_pq == 0 {
                self.source_max_pq = match mdl_max {
                    1000 => 3079,
                    4000 => 3696,
                    10000 => 4095,
                    _ => 3079,
                };
            }
        }
    }
}
