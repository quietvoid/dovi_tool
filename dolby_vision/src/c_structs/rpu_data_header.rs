use libc::c_char;
use std::ptr::null;

use crate::rpu::rpu_data_header::RpuDataHeader as RuRpuDataHeader;

/// C struct for rpu_data_header()
#[repr(C)]
pub struct RpuDataHeader {
    /// Profile guessed from the values in the header
    guessed_profile: u8,

    /// Enhancement layer type (FEL or MEL) if the RPU is profile 7
    /// null pointer if not profile 7
    pub(crate) el_type: *const c_char,

    /// Deprecated since 3.2.0
    /// The field is not actually part of the RPU header
    #[deprecated(
        since = "3.2.0",
        note = "The field is not actually part of the RPU header"
    )]
    rpu_nal_prefix: u8,

    rpu_type: u8,
    rpu_format: u16,
    vdr_rpu_profile: u8,
    vdr_rpu_level: u8,
    vdr_seq_info_present_flag: bool,
    chroma_resampling_explicit_filter_flag: bool,
    coefficient_data_type: u8,
    coefficient_log2_denom: u64,
    vdr_rpu_normalized_idc: u8,
    bl_video_full_range_flag: bool,
    bl_bit_depth_minus8: u64,
    el_bit_depth_minus8: u64,
    vdr_bit_depth_minus8: u64,
    spatial_resampling_filter_flag: bool,
    reserved_zero_3bits: u8,
    el_spatial_resampling_filter_flag: bool,
    disable_residual_flag: bool,
    vdr_dm_metadata_present_flag: bool,
    use_prev_vdr_rpu_flag: bool,
    prev_vdr_rpu_id: u64,
}

impl From<&RuRpuDataHeader> for RpuDataHeader {
    fn from(header: &RuRpuDataHeader) -> Self {
        #[allow(deprecated)]
        Self {
            guessed_profile: header.get_dovi_profile(),
            el_type: null(),
            // FIXME: rpu_nal_prefix deprecation
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
            vdr_bit_depth_minus8: header.vdr_bit_depth_minus8,
            spatial_resampling_filter_flag: header.spatial_resampling_filter_flag,
            reserved_zero_3bits: header.reserved_zero_3bits,
            el_spatial_resampling_filter_flag: header.el_spatial_resampling_filter_flag,
            disable_residual_flag: header.disable_residual_flag,
            vdr_dm_metadata_present_flag: header.vdr_dm_metadata_present_flag,
            use_prev_vdr_rpu_flag: header.use_prev_vdr_rpu_flag,
            prev_vdr_rpu_id: header.prev_vdr_rpu_id,
        }
    }
}
