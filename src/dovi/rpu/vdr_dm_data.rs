use crate::bits::bitvec_reader::BitVecReader;

#[derive(Debug, Default)]
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
    source_min_pq: u16,
    source_max_pq: u16,
    source_diagonal: u16,
    num_ext_blocks: u64,
    ext_metadata_blocks: Vec<ExtMetadataBlock>,
}

#[derive(Debug, Default)]
pub struct ExtMetadataBlock {
    ext_block_length: u64,
    ext_block_level: u8,
    min_pq: u16,
    max_pq: u16,
    avg_pq: u16,
    target_max_pq: u16,
    trim_slope: u16,
    trim_offset: u16,
    trim_power: u16,
    trim_chroma_weight: u16,
    trim_saturation_gain: u16,
    ms_weight: i16,
    active_area_left_offset: u16,
    active_area_right_offset: u16,
    active_area_top_offset: u16,
    active_area_bottom_offset: u16,
}

impl VdrDmData {
    pub fn vdr_dm_data_payload(reader: &mut BitVecReader) -> VdrDmData {
        let mut data = VdrDmData::default();
        data.affected_dm_metadata_id = reader.get_ue();
        data.current_dm_metadata_id = reader.get_ue();
        data.scene_refresh_flag = reader.get_ue();

        data.ycc_to_rgb_coef0 = reader.get_n::<u16>(16) as i16;
        data.ycc_to_rgb_coef1 = reader.get_n::<u16>(16) as i16;
        data.ycc_to_rgb_coef2 = reader.get_n::<u16>(16) as i16;
        data.ycc_to_rgb_coef3 = reader.get_n::<u16>(16) as i16;
        data.ycc_to_rgb_coef4 = reader.get_n::<u16>(16) as i16;
        data.ycc_to_rgb_coef5 = reader.get_n::<u16>(16) as i16;
        data.ycc_to_rgb_coef6 = reader.get_n::<u16>(16) as i16;
        data.ycc_to_rgb_coef7 = reader.get_n::<u16>(16) as i16;
        data.ycc_to_rgb_coef8 = reader.get_n::<u16>(16) as i16;
        data.ycc_to_rgb_offset0 = reader.get_n(32);
        data.ycc_to_rgb_offset1 = reader.get_n(32);
        data.ycc_to_rgb_offset2 = reader.get_n(32);

        data.rgb_to_lms_coef0 = reader.get_n::<u16>(16) as i16;
        data.rgb_to_lms_coef1 = reader.get_n::<u16>(16) as i16;
        data.rgb_to_lms_coef2 = reader.get_n::<u16>(16) as i16;
        data.rgb_to_lms_coef3 = reader.get_n::<u16>(16) as i16;
        data.rgb_to_lms_coef4 = reader.get_n::<u16>(16) as i16;
        data.rgb_to_lms_coef5 = reader.get_n::<u16>(16) as i16;
        data.rgb_to_lms_coef6 = reader.get_n::<u16>(16) as i16;
        data.rgb_to_lms_coef7 = reader.get_n::<u16>(16) as i16;
        data.rgb_to_lms_coef8 = reader.get_n::<u16>(16) as i16;

        data.signal_eotf = reader.get_n(16);
        data.signal_eotf_param0 = reader.get_n(16);
        data.signal_eotf_param1 = reader.get_n(16);
        data.signal_eotf_param2 = reader.get_n(32);
        data.signal_bit_depth = reader.get_n(5);
        data.signal_color_space = reader.get_n(2);
        data.signal_chroma_format = reader.get_n(2);
        data.signal_full_range_flag = reader.get_n(2);
        data.source_min_pq = reader.get_n(12);
        data.source_max_pq = reader.get_n(12);
        data.source_diagonal = reader.get_n(10);
        data.num_ext_blocks = reader.get_ue();

        if data.num_ext_blocks > 0 {
            while !reader.is_aligned() {
                reader.get();
            }

            for _ in 0..data.num_ext_blocks {
                let mut ext_metadata_block = ExtMetadataBlock::default();

                ext_metadata_block.ext_block_length = reader.get_ue();
                ext_metadata_block.ext_block_level = reader.get_n(8);

                let ext_block_len_bits = 8 * ext_metadata_block.ext_block_length;
                let mut ext_block_use_bits = 0;

                if ext_metadata_block.ext_block_level == 1 {
                    ext_metadata_block.min_pq = reader.get_n(12);
                    ext_metadata_block.max_pq = reader.get_n(12);
                    ext_metadata_block.avg_pq = reader.get_n(12);

                    ext_block_use_bits += 36;
                }

                if ext_metadata_block.ext_block_level == 2 {
                    ext_metadata_block.target_max_pq = reader.get_n(12);
                    ext_metadata_block.trim_slope = reader.get_n(12);
                    ext_metadata_block.trim_offset = reader.get_n(12);
                    ext_metadata_block.trim_power = reader.get_n(12);
                    ext_metadata_block.trim_chroma_weight = reader.get_n(12);
                    ext_metadata_block.trim_saturation_gain = reader.get_n(12);
                    ext_metadata_block.ms_weight = reader.get_n::<u16>(13) as i16;

                    ext_block_use_bits += 85;
                }

                if ext_metadata_block.ext_block_level == 5 {
                    ext_metadata_block.active_area_left_offset = reader.get_n(13);
                    ext_metadata_block.active_area_right_offset = reader.get_n(13);
                    ext_metadata_block.active_area_top_offset = reader.get_n(13);
                    ext_metadata_block.active_area_bottom_offset = reader.get_n(13);

                    ext_block_use_bits += 52;
                }

                while ext_block_use_bits < ext_block_len_bits {
                    reader.get();
                    ext_block_use_bits += 1;
                }

                data.ext_metadata_blocks.push(ext_metadata_block);
            }
        }

        data.validate();

        data
    }

    pub fn validate(&self) {
        assert!(self.affected_dm_metadata_id <= 15);
        assert!(self.signal_bit_depth >= 8 && self.signal_bit_depth <= 16);
        assert_eq!(self.signal_eotf, 65535);
    }
}
