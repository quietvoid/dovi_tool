use std::ptr::null;

use super::rpu::rpu_data_header::RpuDataHeader as RuRpuDataHeader;
use crate::rpu::rpu_data_mapping::RpuDataMapping as RuRpuDataMapping;
use crate::rpu::rpu_data_nlq::RpuDataNlq as RuRpuDataNlq;
use crate::rpu::vdr_dm_data::VdrDmData as RuVdrDmData;
use crate::st2094_10::{
    level1::ExtMetadataBlockLevel1, level2::ExtMetadataBlockLevel2, level3::ExtMetadataBlockLevel3,
    level4::ExtMetadataBlockLevel4, level5::ExtMetadataBlockLevel5, level6::ExtMetadataBlockLevel6,
    ExtMetadataBlock, ST2094_10Meta as RuST2094_10Meta,
};

use super::rpu::NUM_COMPONENTS;

mod buffers;
pub use buffers::*;
use libc::size_t;

/// C struct for rpu_data_header()
#[repr(C)]
pub struct RpuDataHeader {
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
    pub num_x_partitions_minus1: u64,
    pub num_y_partitions_minus1: u64,
}

/// C struct for rpu_data_mapping()
#[repr(C)]
pub struct RpuDataMapping {
    mapping_idc: [U64Data; NUM_COMPONENTS],
    mapping_param_pred_flag: [Data; NUM_COMPONENTS],
    num_mapping_param_predictors: [U64Data; NUM_COMPONENTS],
    diff_pred_part_idx_mapping_minus1: [U64Data; NUM_COMPONENTS],
    poly_order_minus1: [U64Data; NUM_COMPONENTS],
    linear_interp_flag: [Data; NUM_COMPONENTS],
    pred_linear_interp_value_int: [U64Data; NUM_COMPONENTS],
    pred_linear_interp_value: [U64Data; NUM_COMPONENTS],
    poly_coef_int: [I64Data2D; NUM_COMPONENTS],
    poly_coef: [U64Data2D; NUM_COMPONENTS],
    mmr_order_minus1: [Data; NUM_COMPONENTS],
    mmr_constant_int: [I64Data; NUM_COMPONENTS],
    mmr_constant: [U64Data; NUM_COMPONENTS],
    mmr_coef_int: [I64Data3D; NUM_COMPONENTS],
    mmr_coef: [U64Data3D; NUM_COMPONENTS],
}

/// C struct for rpu_data_nlq()
///
/// Here all the Data2D structs are of size N x 3.
/// Using dynamic buffers for convenience.
#[repr(C)]
pub struct RpuDataNlq {
    num_nlq_param_predictors: U64Data2D,
    nlq_param_pred_flag: Data2D,
    diff_pred_part_idx_nlq_minus1: U64Data2D,
    nlq_offset: U64Data2D,
    vdr_in_max_int: U64Data2D,
    vdr_in_max: U64Data2D,
    linear_deadzone_slope_int: U64Data2D,
    linear_deadzone_slope: U64Data2D,
    linear_deadzone_threshold_int: U64Data2D,
    linear_deadzone_threshold: U64Data2D,
}

/// C struct for vdr_dm_data()
#[repr(C)]
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
    st2094_10_metadata: ST2094_10Meta,
}

/// C struct for the list of ext_metadata_block()
#[repr(C)]
pub struct ST2094_10Meta {
    /// Number of metadata blocks
    num_ext_blocks: u64,

    level1: *const ExtMetadataBlockLevel1,
    level2: Level2BlockList,
    level3: *const ExtMetadataBlockLevel3,
    level4: *const ExtMetadataBlockLevel4,
    level5: *const ExtMetadataBlockLevel5,
    level6: *const ExtMetadataBlockLevel6,
}

#[repr(C)]
pub struct Level2BlockList {
    /// Pointer to the list of ExtMetadataBlockLevel2 structs
    pub list: *const *const ExtMetadataBlockLevel2,
    /// List length
    pub len: size_t,
}

impl From<&RuRpuDataHeader> for RpuDataHeader {
    fn from(header: &RuRpuDataHeader) -> Self {
        Self {
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
            num_x_partitions_minus1: header.num_x_partitions_minus1,
            num_y_partitions_minus1: header.num_y_partitions_minus1,
        }
    }
}

impl From<&RuRpuDataMapping> for RpuDataMapping {
    fn from(data: &RuRpuDataMapping) -> Self {
        Self {
            mapping_idc: components_to_cdata::<Vec<u64>, U64Data>(&data.mapping_idc),
            mapping_param_pred_flag: components_to_cdata::<Vec<bool>, Data>(
                &data.mapping_param_pred_flag,
            ),
            num_mapping_param_predictors: components_to_cdata::<Vec<u64>, U64Data>(
                &data.num_mapping_param_predictors,
            ),
            diff_pred_part_idx_mapping_minus1: components_to_cdata::<Vec<u64>, U64Data>(
                &data.diff_pred_part_idx_mapping_minus1,
            ),
            poly_order_minus1: components_to_cdata::<Vec<u64>, U64Data>(&data.poly_order_minus1),
            linear_interp_flag: components_to_cdata::<Vec<bool>, Data>(
                &data.mapping_param_pred_flag,
            ),
            pred_linear_interp_value_int: components_to_cdata::<Vec<u64>, U64Data>(
                &data.pred_linear_interp_value_int,
            ),
            pred_linear_interp_value: components_to_cdata::<Vec<u64>, U64Data>(
                &data.pred_linear_interp_value,
            ),
            poly_coef_int: components_to_cdata::<Vec<Vec<i64>>, I64Data2D>(&data.poly_coef_int),
            poly_coef: components_to_cdata::<Vec<Vec<u64>>, U64Data2D>(&data.poly_coef),
            mmr_order_minus1: components_to_cdata::<Vec<u8>, Data>(&data.mmr_order_minus1),
            mmr_constant_int: components_to_cdata::<Vec<i64>, I64Data>(&data.mmr_constant_int),
            mmr_constant: components_to_cdata::<Vec<u64>, U64Data>(&data.mmr_constant),
            mmr_coef_int: components_to_cdata::<Vec<Vec<Vec<i64>>>, I64Data3D>(&data.mmr_coef_int),
            mmr_coef: components_to_cdata::<Vec<Vec<Vec<u64>>>, U64Data3D>(&data.mmr_coef),
        }
    }
}

impl From<&RuRpuDataNlq> for RpuDataNlq {
    fn from(data: &RuRpuDataNlq) -> Self {
        Self {
            num_nlq_param_predictors: U64Data2D::from(&data.num_nlq_param_predictors),
            nlq_param_pred_flag: Data2D::from(&data.nlq_param_pred_flag),
            diff_pred_part_idx_nlq_minus1: U64Data2D::from(&data.diff_pred_part_idx_nlq_minus1),
            nlq_offset: U64Data2D::from(&data.nlq_offset),
            vdr_in_max_int: U64Data2D::from(&data.vdr_in_max_int),
            vdr_in_max: U64Data2D::from(&data.vdr_in_max),
            linear_deadzone_slope_int: U64Data2D::from(&data.linear_deadzone_slope_int),
            linear_deadzone_slope: U64Data2D::from(&data.linear_deadzone_slope),
            linear_deadzone_threshold_int: U64Data2D::from(&data.linear_deadzone_threshold_int),
            linear_deadzone_threshold: U64Data2D::from(&data.linear_deadzone_threshold),
        }
    }
}

impl From<&RuVdrDmData> for VdrDmData {
    fn from(data: &RuVdrDmData) -> Self {
        Self {
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
            st2094_10_metadata: ST2094_10Meta::from(&data.st2094_10_metadata),
        }
    }
}

impl From<&RuST2094_10Meta> for ST2094_10Meta {
    fn from(data: &RuST2094_10Meta) -> Self {
        Self {
            num_ext_blocks: data.num_ext_blocks,
            level1: data
                .ext_metadata_blocks
                .iter()
                .find(|b| matches!(b, ExtMetadataBlock::Level1(_)))
                .map_or(null(), |b| match b {
                    ExtMetadataBlock::Level1(e) => {
                        Box::into_raw(Box::new(e.clone())) as *const ExtMetadataBlockLevel1
                    }
                    _ => null(),
                }),
            level2: Level2BlockList::from(&data.ext_metadata_blocks),
            level3: data
                .ext_metadata_blocks
                .iter()
                .find(|b| matches!(b, ExtMetadataBlock::Level3(_)))
                .map_or(null(), |b| match b {
                    ExtMetadataBlock::Level3(e) => {
                        Box::into_raw(Box::new(e.clone())) as *const ExtMetadataBlockLevel3
                    }
                    _ => null(),
                }),
            level4: data
                .ext_metadata_blocks
                .iter()
                .find(|b| matches!(b, ExtMetadataBlock::Level4(_)))
                .map_or(null(), |b| match b {
                    ExtMetadataBlock::Level4(e) => {
                        Box::into_raw(Box::new(e.clone())) as *const ExtMetadataBlockLevel4
                    }
                    _ => null(),
                }),
            level5: data
                .ext_metadata_blocks
                .iter()
                .find(|b| matches!(b, ExtMetadataBlock::Level5(_)))
                .map_or(null(), |b| match b {
                    ExtMetadataBlock::Level5(e) => {
                        Box::into_raw(Box::new(e.clone())) as *const ExtMetadataBlockLevel5
                    }
                    _ => null(),
                }),
            level6: data
                .ext_metadata_blocks
                .iter()
                .find(|b| matches!(b, ExtMetadataBlock::Level6(_)))
                .map_or(null(), |b| match b {
                    ExtMetadataBlock::Level6(e) => {
                        Box::into_raw(Box::new(e.clone())) as *const ExtMetadataBlockLevel6
                    }
                    _ => null(),
                }),
        }
    }
}

impl From<&Vec<ExtMetadataBlock>> for Level2BlockList {
    fn from(blocks: &Vec<ExtMetadataBlock>) -> Self {
        let level2_blocks: Vec<*const ExtMetadataBlockLevel2> = blocks
            .iter()
            .filter(|b| matches!(b, ExtMetadataBlock::Level2(_)))
            .map(|b| match b {
                ExtMetadataBlock::Level2(e) => {
                    Box::into_raw(Box::new(e.clone())) as *const ExtMetadataBlockLevel2
                }
                _ => null(),
            })
            .collect();

        Self {
            len: level2_blocks.len(),
            list: Box::into_raw(level2_blocks.into_boxed_slice())
                as *const *const ExtMetadataBlockLevel2,
        }
    }
}

impl RpuDataHeader {
    /// # Safety
    /// The buffer pointers should be valid.
    pub unsafe fn free(&self) {
        self.pred_pivot_value.iter().for_each(|data| data.free());
    }
}

impl RpuDataMapping {
    /// # Safety
    /// The buffer pointers should be valid.
    pub unsafe fn free(&self) {
        self.mapping_idc.iter().for_each(|data| data.free());
        self.mapping_param_pred_flag
            .iter()
            .for_each(|data| data.free());
        self.num_mapping_param_predictors
            .iter()
            .for_each(|data| data.free());
        self.diff_pred_part_idx_mapping_minus1
            .iter()
            .for_each(|data| data.free());
        self.poly_order_minus1.iter().for_each(|data| data.free());
        self.linear_interp_flag.iter().for_each(|data| data.free());
        self.pred_linear_interp_value_int
            .iter()
            .for_each(|data| data.free());
        self.pred_linear_interp_value
            .iter()
            .for_each(|data| data.free());
        self.poly_coef_int.iter().for_each(|data| data.free());
        self.poly_coef.iter().for_each(|data| data.free());
        self.mmr_order_minus1.iter().for_each(|data| data.free());
        self.mmr_constant_int.iter().for_each(|data| data.free());
        self.mmr_constant.iter().for_each(|data| data.free());
        self.mmr_coef_int.iter().for_each(|data| data.free());
        self.mmr_coef.iter().for_each(|data| data.free());
    }
}

impl RpuDataNlq {
    /// # Safety
    /// The buffer pointers should be valid.
    pub unsafe fn free(&self) {
        self.num_nlq_param_predictors.free();
        self.nlq_param_pred_flag.free();
        self.diff_pred_part_idx_nlq_minus1.free();
        self.nlq_offset.free();
        self.vdr_in_max_int.free();
        self.vdr_in_max.free();
        self.linear_deadzone_slope_int.free();
        self.linear_deadzone_slope.free();
        self.linear_deadzone_threshold_int.free();
        self.linear_deadzone_threshold.free();
    }
}

impl VdrDmData {
    /// # Safety
    pub unsafe fn free(&self) {
        self.st2094_10_metadata.free();
    }
}

impl ST2094_10Meta {
    /// # Safety
    pub unsafe fn free(&self) {
        Box::from_raw(self.level1 as *mut ExtMetadataBlockLevel1);
        self.level2.free();
        Box::from_raw(self.level3 as *mut ExtMetadataBlockLevel3);
        Box::from_raw(self.level4 as *mut ExtMetadataBlockLevel4);
        Box::from_raw(self.level5 as *mut ExtMetadataBlockLevel5);
        Box::from_raw(self.level6 as *mut ExtMetadataBlockLevel6);
    }
}

impl Level2BlockList {
    /// # Safety
    pub unsafe fn free(&self) {
        let list = Vec::from_raw_parts(
            self.list as *mut *const ExtMetadataBlockLevel2,
            self.len as usize,
            self.len as usize,
        );

        for data_ptr in list {
            Box::from_raw(data_ptr as *mut ExtMetadataBlockLevel2);
        }
    }
}

fn components_to_cdata<T, R>(cmps: &[T; NUM_COMPONENTS]) -> [R; NUM_COMPONENTS]
where
    T: Clone,
    R: From<T>,
{
    [
        R::from(cmps[0].clone()),
        R::from(cmps[1].clone()),
        R::from(cmps[2].clone()),
    ]
}
