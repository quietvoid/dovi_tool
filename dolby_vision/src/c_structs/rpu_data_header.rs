use libc::c_char;
use std::{ffi::CString, ptr::null};

use crate::rpu::rpu_data_header::RpuDataHeader as RuRpuDataHeader;

use super::{components_to_cdata, Freeable, U64Data, NUM_COMPONENTS};

/// C struct for rpu_data_header()
#[repr(C)]
pub struct RpuDataHeader {
    /// Profile guessed from the values in the header
    pub guessed_profile: u8,

    /// Subprofile (FEL or MEL) if the RPU is profile 7
    /// null pointer if not profile 7
    pub subprofile: *const c_char,

    pub rpu_nal_prefix: u8,
    pub rpu_type: u8,
    pub rpu_format: u16,
    pub vdr_rpu_profile: u8,
    pub vdr_rpu_level: u8,
    pub vdr_seq_info_present_flag: bool,
    pub chroma_resampling_explicit_filter_flag: bool,
    pub coefficient_data_type: u8,
    pub coefficient_log2_denom: u64,
    pub vdr_rpu_normalized_idc: u8,
    pub bl_video_full_range_flag: bool,
    pub bl_bit_depth_minus8: u64,
    pub el_bit_depth_minus8: u64,
    pub vdr_bit_depth_minus_8: u64,
    pub spatial_resampling_filter_flag: bool,
    pub reserved_zero_3bits: u8,
    pub el_spatial_resampling_filter_flag: bool,
    pub disable_residual_flag: bool,
    pub vdr_dm_metadata_present_flag: bool,
    pub use_prev_vdr_rpu_flag: bool,
    pub prev_vdr_rpu_id: u64,
    pub vdr_rpu_id: u64,
    pub mapping_color_space: u64,
    pub mapping_chroma_format_idc: u64,
    pub num_pivots_minus_2: [u64; NUM_COMPONENTS],
    pub pred_pivot_value: [U64Data; NUM_COMPONENTS],
    /// Set to -1 to represent Option::None
    pub nlq_method_idc: i32,
    /// Set to -1 to represent Option::None
    pub nlq_num_pivots_minus2: i32,
    /// Length of zero when not present. Only present in profile 4 and 7.
    pub nlq_pred_pivot_value: U64Data,
    pub num_x_partitions_minus1: u64,
    pub num_y_partitions_minus1: u64,
}

impl RpuDataHeader {
    /// # Safety
    /// The buffer pointers should be valid.
    pub unsafe fn free(&self) {
        self.pred_pivot_value.iter().for_each(|data| data.free());
        self.nlq_pred_pivot_value.free();

        if !self.subprofile.is_null() {
            drop(CString::from_raw(self.subprofile as *mut c_char));
        }
    }
}

impl From<&RuRpuDataHeader> for RpuDataHeader {
    fn from(header: &RuRpuDataHeader) -> Self {
        Self {
            guessed_profile: header.get_dovi_profile(),
            subprofile: null(),
            rpu_nal_prefix: header.rpu_nal_prefix,
            rpu_type: header.rpu_type,
            rpu_format: header.rpu_format,
            vdr_rpu_profile: header.vdr_rpu_profile,
            vdr_rpu_level: header.vdr_rpu_level,
            vdr_seq_info_present_flag: header.vdr_seq_info_present_flag,
            chroma_resampling_explicit_filter_flag: header.chroma_resampling_explicit_filter_flag,
            coefficient_data_type: header.coefficient_data_type,
            coefficient_log2_denom: header.coefficient_log2_denom,
            vdr_rpu_normalized_idc: header.vdr_rpu_normalized_idc,
            bl_video_full_range_flag: header.bl_video_full_range_flag,
            bl_bit_depth_minus8: header.bl_bit_depth_minus8,
            el_bit_depth_minus8: header.el_bit_depth_minus8,
            vdr_bit_depth_minus_8: header.vdr_bit_depth_minus_8,
            spatial_resampling_filter_flag: header.spatial_resampling_filter_flag,
            reserved_zero_3bits: header.reserved_zero_3bits,
            el_spatial_resampling_filter_flag: header.el_spatial_resampling_filter_flag,
            disable_residual_flag: header.disable_residual_flag,
            vdr_dm_metadata_present_flag: header.vdr_dm_metadata_present_flag,
            use_prev_vdr_rpu_flag: header.use_prev_vdr_rpu_flag,
            prev_vdr_rpu_id: header.prev_vdr_rpu_id,
            vdr_rpu_id: header.vdr_rpu_id,
            mapping_color_space: header.mapping_color_space,
            mapping_chroma_format_idc: header.mapping_chroma_format_idc,
            num_pivots_minus_2: header.num_pivots_minus_2,
            pred_pivot_value: components_to_cdata::<Vec<u64>, U64Data>(&header.pred_pivot_value),
            nlq_method_idc: header.nlq_method_idc.map_or(-1, |e| e as i32),
            nlq_num_pivots_minus2: header.nlq_num_pivots_minus2.map_or(-1, |e| e as i32),
            nlq_pred_pivot_value: U64Data::from(header.nlq_pred_pivot_value),
            num_x_partitions_minus1: header.num_x_partitions_minus1,
            num_y_partitions_minus1: header.num_y_partitions_minus1,
        }
    }
}
