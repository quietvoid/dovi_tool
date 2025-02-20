use crate::rpu::vdr_dm_data::VdrDmData as RuVdrDmData;

use super::DmData;

/// C struct for vdr_dm_data()
#[repr(C)]
pub struct VdrDmData {
    compressed: bool,

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
    dm_data: DmData,
}

impl VdrDmData {
    /// # Safety
    pub unsafe fn free(&self) {
        unsafe {
            self.dm_data.free();
        }
    }
}

impl From<&RuVdrDmData> for VdrDmData {
    fn from(data: &RuVdrDmData) -> Self {
        Self {
            compressed: data.compressed,
            affected_dm_metadata_id: data.affected_dm_metadata_id,
            current_dm_metadata_id: data.current_dm_metadata_id,
            scene_refresh_flag: data.scene_refresh_flag,
            ycc_to_rgb_coef0: data.ycc_to_rgb_coef0,
            ycc_to_rgb_coef1: data.ycc_to_rgb_coef1,
            ycc_to_rgb_coef2: data.ycc_to_rgb_coef2,
            ycc_to_rgb_coef3: data.ycc_to_rgb_coef3,
            ycc_to_rgb_coef4: data.ycc_to_rgb_coef4,
            ycc_to_rgb_coef5: data.ycc_to_rgb_coef5,
            ycc_to_rgb_coef6: data.ycc_to_rgb_coef6,
            ycc_to_rgb_coef7: data.ycc_to_rgb_coef7,
            ycc_to_rgb_coef8: data.ycc_to_rgb_coef8,
            ycc_to_rgb_offset0: data.ycc_to_rgb_offset0,
            ycc_to_rgb_offset1: data.ycc_to_rgb_offset1,
            ycc_to_rgb_offset2: data.ycc_to_rgb_offset2,
            rgb_to_lms_coef0: data.rgb_to_lms_coef0,
            rgb_to_lms_coef1: data.rgb_to_lms_coef1,
            rgb_to_lms_coef2: data.rgb_to_lms_coef2,
            rgb_to_lms_coef3: data.rgb_to_lms_coef3,
            rgb_to_lms_coef4: data.rgb_to_lms_coef4,
            rgb_to_lms_coef5: data.rgb_to_lms_coef5,
            rgb_to_lms_coef6: data.rgb_to_lms_coef6,
            rgb_to_lms_coef7: data.rgb_to_lms_coef7,
            rgb_to_lms_coef8: data.rgb_to_lms_coef8,
            signal_eotf: data.signal_eotf,
            signal_eotf_param0: data.signal_eotf_param0,
            signal_eotf_param1: data.signal_eotf_param1,
            signal_eotf_param2: data.signal_eotf_param2,
            signal_bit_depth: data.signal_bit_depth,
            signal_color_space: data.signal_color_space,
            signal_chroma_format: data.signal_chroma_format,
            signal_full_range_flag: data.signal_full_range_flag,
            source_min_pq: data.source_min_pq,
            source_max_pq: data.source_max_pq,
            source_diagonal: data.source_diagonal,
            dm_data: DmData::combine_dm_data(
                data.cmv29_metadata.as_ref(),
                data.cmv40_metadata.as_ref(),
            ),
        }
    }
}
