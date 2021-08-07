use super::{prelude::*, BitVecReader, BitVecWriter, DoviRpu};

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
    pub(crate) ext_metadata_blocks: Vec<ExtMetadataBlock>,
}

#[derive(Debug)]
pub enum ExtMetadataBlock {
    Level1(ExtMetadataBlockLevel1),
    Level2(ExtMetadataBlockLevel2),
    Level3(ExtMetadataBlockLevel3),
    Level4(ExtMetadataBlockLevel4),
    Level5(ExtMetadataBlockLevel5),
    Level6(ExtMetadataBlockLevel6),
    Reserved(ReservedExtMetadataBlock),
}

#[derive(Debug, Default)]
pub struct BlockInfo {
    ext_block_length: u64,
    ext_block_level: u8,
    remaining: BitVec<Msb0, u8>,
}

#[derive(Debug, Default)]
pub struct ExtMetadataBlockLevel1 {
    block_info: BlockInfo,
    min_pq: u16,
    max_pq: u16,
    avg_pq: u16,
}

#[derive(Debug, Default)]
pub struct ExtMetadataBlockLevel2 {
    block_info: BlockInfo,
    target_max_pq: u16,
    trim_slope: u16,
    trim_offset: u16,
    trim_power: u16,
    trim_chroma_weight: u16,
    trim_saturation_gain: u16,
    ms_weight: i16,
}

#[derive(Debug, Default)]
pub struct ExtMetadataBlockLevel3 {
    block_info: BlockInfo,
    min_pq_offset: u16,
    max_pq_offset: u16,
    avg_pq_offset: u16,
}

#[derive(Debug, Default)]
pub struct ExtMetadataBlockLevel4 {
    block_info: BlockInfo,
    anchor_pq: u16,
    anchor_power: u16,
}

#[derive(Debug, Default)]
pub struct ExtMetadataBlockLevel5 {
    block_info: BlockInfo,
    active_area_left_offset: u16,
    active_area_right_offset: u16,
    active_area_top_offset: u16,
    active_area_bottom_offset: u16,
}

#[derive(Debug, Default)]
pub struct ExtMetadataBlockLevel6 {
    block_info: BlockInfo,
    max_display_mastering_luminance: u16,
    min_display_mastering_luminance: u16,
    max_content_light_level: u16,
    max_frame_average_light_level: u16,
}

#[derive(Debug, Default)]
pub struct ReservedExtMetadataBlock {
    block_info: BlockInfo,
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
                assert_eq!(reader.get(), false);
            }

            for _ in 0..data.num_ext_blocks {
                let ext_metadata_block = ExtMetadataBlock::parse(reader);
                data.ext_metadata_blocks.push(ext_metadata_block);
            }
        }

        data
    }

    pub fn validate(&self) {
        assert!(self.affected_dm_metadata_id <= 15);
        assert!(self.signal_bit_depth >= 8 && self.signal_bit_depth <= 16);

        if self.signal_eotf_param0 == 0
            && self.signal_eotf_param1 == 0
            && self.signal_eotf_param2 == 0
        {
            assert_eq!(self.signal_eotf, 65535);
        }
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
        writer.write_ue(self.num_ext_blocks);

        if self.num_ext_blocks > 0 {
            while !writer.is_aligned() {
                writer.write(false);
            }

            for ext_metadata_block in &self.ext_metadata_blocks {
                ext_metadata_block.write(writer);
            }
        }
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
}

impl ExtMetadataBlock {
    pub fn parse(reader: &mut BitVecReader) -> ExtMetadataBlock {
        let mut block_info = BlockInfo::default();

        block_info.ext_block_length = reader.get_ue();
        block_info.ext_block_level = reader.get_n(8);

        let ext_block_len_bits = 8 * block_info.ext_block_length;
        let mut ext_block_use_bits = 0;

        let mut ext_metadata_block = match block_info.ext_block_level {
            1 => {
                assert_eq!(block_info.ext_block_length, 5);

                let mut block = ExtMetadataBlockLevel1::default();
                block.min_pq = reader.get_n(12);
                block.max_pq = reader.get_n(12);
                block.avg_pq = reader.get_n(12);

                ext_block_use_bits += 36;

                ExtMetadataBlock::Level1(block)
            }
            2 => {
                assert_eq!(block_info.ext_block_length, 11);

                let mut block = ExtMetadataBlockLevel2::default();
                block.target_max_pq = reader.get_n(12);
                block.trim_slope = reader.get_n(12);
                block.trim_offset = reader.get_n(12);
                block.trim_power = reader.get_n(12);
                block.trim_chroma_weight = reader.get_n(12);
                block.trim_saturation_gain = reader.get_n(12);
                block.ms_weight = reader.get_n::<u16>(13) as i16;

                ext_block_use_bits += 85;

                ExtMetadataBlock::Level2(block)
            }
            3 => {
                assert_eq!(block_info.ext_block_length, 2);

                let mut block = ExtMetadataBlockLevel3::default();
                block.min_pq_offset = reader.get_n(12);
                block.max_pq_offset = reader.get_n(12);
                block.avg_pq_offset = reader.get_n(12);

                ext_block_use_bits += 36;

                ExtMetadataBlock::Level3(block)
            }
            4 => {
                assert_eq!(block_info.ext_block_length, 3);

                let mut block = ExtMetadataBlockLevel4::default();
                block.anchor_pq = reader.get_n(12);
                block.anchor_power = reader.get_n(12);

                ext_block_use_bits += 24;

                ExtMetadataBlock::Level4(block)
            }
            5 => {
                assert_eq!(block_info.ext_block_length, 7);

                let mut block = ExtMetadataBlockLevel5::default();
                block.active_area_left_offset = reader.get_n(13);
                block.active_area_right_offset = reader.get_n(13);
                block.active_area_top_offset = reader.get_n(13);
                block.active_area_bottom_offset = reader.get_n(13);

                ext_block_use_bits += 52;

                ExtMetadataBlock::Level5(block)
            }
            6 => {
                assert_eq!(block_info.ext_block_length, 8);
                let mut block = ExtMetadataBlockLevel6::default();

                block.max_display_mastering_luminance = reader.get_n(16);
                block.min_display_mastering_luminance = reader.get_n(16);
                block.max_content_light_level = reader.get_n(16);
                block.max_frame_average_light_level = reader.get_n(16);

                ext_block_use_bits += 64;

                ExtMetadataBlock::Level6(block)
            }
            _ => {
                let block = ReservedExtMetadataBlock::default();
                ExtMetadataBlock::Reserved(block)
            }
        };

        while ext_block_use_bits < ext_block_len_bits {
            block_info.remaining.push(reader.get());
            ext_block_use_bits += 1;
        }

        match ext_metadata_block {
            ExtMetadataBlock::Level1(ref mut b) => b.block_info = block_info,
            ExtMetadataBlock::Level2(ref mut b) => b.block_info = block_info,
            ExtMetadataBlock::Level3(ref mut b) => b.block_info = block_info,
            ExtMetadataBlock::Level4(ref mut b) => b.block_info = block_info,
            ExtMetadataBlock::Level5(ref mut b) => b.block_info = block_info,
            ExtMetadataBlock::Level6(ref mut b) => b.block_info = block_info,
            ExtMetadataBlock::Reserved(ref mut b) => b.block_info = block_info,
        }

        ext_metadata_block
    }

    pub fn write(&self, writer: &mut BitVecWriter) {
        let block_info = match self {
            ExtMetadataBlock::Level1(b) => &b.block_info,
            ExtMetadataBlock::Level2(b) => &b.block_info,
            ExtMetadataBlock::Level3(b) => &b.block_info,
            ExtMetadataBlock::Level4(b) => &b.block_info,
            ExtMetadataBlock::Level5(b) => &b.block_info,
            ExtMetadataBlock::Level6(b) => &b.block_info,
            ExtMetadataBlock::Reserved(b) => &b.block_info,
        };

        writer.write_ue(block_info.ext_block_length);
        writer.write_n(&block_info.ext_block_level.to_be_bytes(), 8);

        match self {
            ExtMetadataBlock::Level1(block) => {
                writer.write_n(&block.min_pq.to_be_bytes(), 12);
                writer.write_n(&block.max_pq.to_be_bytes(), 12);
                writer.write_n(&block.avg_pq.to_be_bytes(), 12);
            }
            ExtMetadataBlock::Level2(block) => {
                writer.write_n(&block.target_max_pq.to_be_bytes(), 12);
                writer.write_n(&block.trim_slope.to_be_bytes(), 12);
                writer.write_n(&block.trim_offset.to_be_bytes(), 12);
                writer.write_n(&block.trim_power.to_be_bytes(), 12);
                writer.write_n(&block.trim_chroma_weight.to_be_bytes(), 12);
                writer.write_n(&block.trim_saturation_gain.to_be_bytes(), 12);

                writer.write_n(&block.ms_weight.to_be_bytes(), 13);
            }
            ExtMetadataBlock::Level3(block) => {
                writer.write_n(&block.min_pq_offset.to_be_bytes(), 12);
                writer.write_n(&block.max_pq_offset.to_be_bytes(), 12);
                writer.write_n(&block.avg_pq_offset.to_be_bytes(), 12);
            }
            ExtMetadataBlock::Level4(block) => {
                writer.write_n(&block.anchor_pq.to_be_bytes(), 12);
                writer.write_n(&block.anchor_power.to_be_bytes(), 12);
            }
            ExtMetadataBlock::Level5(block) => {
                writer.write_n(&block.active_area_left_offset.to_be_bytes(), 13);
                writer.write_n(&block.active_area_right_offset.to_be_bytes(), 13);
                writer.write_n(&block.active_area_top_offset.to_be_bytes(), 13);
                writer.write_n(&block.active_area_bottom_offset.to_be_bytes(), 13);
            }
            ExtMetadataBlock::Level6(block) => {
                writer.write_n(&block.max_display_mastering_luminance.to_be_bytes(), 16);
                writer.write_n(&block.min_display_mastering_luminance.to_be_bytes(), 16);
                writer.write_n(&block.max_content_light_level.to_be_bytes(), 16);
                writer.write_n(&block.max_frame_average_light_level.to_be_bytes(), 16);
            }
            ExtMetadataBlock::Reserved(_) => {
                // Copy the data
                block_info.remaining.iter().for_each(|b| writer.write(*b));
            }
        }

        // Write zero bytes until aligned
        match self {
            ExtMetadataBlock::Reserved(_) => (),
            _ => block_info
                .remaining
                .iter()
                .for_each(|_| writer.write(false)),
        }
    }
}

impl ExtMetadataBlockLevel5 {
    pub fn _get_offsets(&self) -> Vec<u16> {
        vec![
            self.active_area_left_offset,
            self.active_area_right_offset,
            self.active_area_top_offset,
            self.active_area_bottom_offset,
        ]
    }

    pub fn set_offsets(&mut self, left: u16, right: u16, top: u16, bottom: u16) {
        self.active_area_left_offset = left;
        self.active_area_right_offset = right;
        self.active_area_top_offset = top;
        self.active_area_bottom_offset = bottom;
    }

    pub fn crop(&mut self) {
        self.active_area_left_offset = 0;
        self.active_area_right_offset = 0;
        self.active_area_top_offset = 0;
        self.active_area_bottom_offset = 0;
    }

    pub fn get_mut(rpu: &mut DoviRpu) -> Option<&mut ExtMetadataBlockLevel5> {
        if let Some(ref mut vdr_dm_data) = rpu.vdr_dm_data {
            for ext in vdr_dm_data.ext_metadata_blocks.iter_mut() {
                if let ExtMetadataBlock::Level5(block) = ext {
                    return Some(block);
                }
            }
        }

        None
    }
}
