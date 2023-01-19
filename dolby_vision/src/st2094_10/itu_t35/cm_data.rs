use anyhow::Result;
use bitvec_helpers::bitstream_io_reader::BsIoSliceReader;

use super::UserDataTypeStruct;

use crate::rpu::NUM_COMPONENTS;

#[derive(Default, Debug)]
pub struct ST2094_10CmData {
    pub ccm_profile: u8,
    pub ccm_level: u8,
    pub coefficient_log2_denom: u64,
    pub bl_bit_depth_minus8: u64,
    pub el_bit_depth_minus8: u64,
    pub hdr_bit_depth_minus8: u64,
    pub disable_residual_flag: bool,

    pub num_pivots_minus2: [u64; NUM_COMPONENTS],
    pub pred_pivot_value: [Vec<u64>; NUM_COMPONENTS],

    pub mapping_idc: [Vec<u64>; NUM_COMPONENTS],
    pub poly_order_minus1: [Vec<u64>; NUM_COMPONENTS],
    pub poly_coef_int: [Vec<Vec<i64>>; NUM_COMPONENTS],
    pub poly_coef: [Vec<Vec<u64>>; NUM_COMPONENTS],
    pub mmr_order_minus1: [Vec<u8>; NUM_COMPONENTS],
    pub mmr_constant_int: [Vec<i64>; NUM_COMPONENTS],
    pub mmr_constant: [Vec<u64>; NUM_COMPONENTS],
    pub mmr_coef_int: [Vec<Vec<Vec<i64>>>; NUM_COMPONENTS],
    pub mmr_coef: [Vec<Vec<Vec<u64>>>; NUM_COMPONENTS],

    pub nlq_offset: [u64; NUM_COMPONENTS],
    pub hdr_in_max_int: [u64; NUM_COMPONENTS],
    pub hdr_in_max: [u64; NUM_COMPONENTS],
    pub linear_deadzone_slope_int: [u64; NUM_COMPONENTS],
    pub linear_deadzone_slope: [u64; NUM_COMPONENTS],
    pub linear_deadzone_threshold_int: [u64; NUM_COMPONENTS],
    pub linear_deadzone_threshold: [u64; NUM_COMPONENTS],
}

impl ST2094_10CmData {
    pub(crate) fn parse(reader: &mut BsIoSliceReader) -> Result<UserDataTypeStruct> {
        let mut meta = ST2094_10CmData {
            ccm_profile: reader.get_n(4)?,
            ccm_level: reader.get_n(4)?,
            coefficient_log2_denom: reader.get_ue()?,
            bl_bit_depth_minus8: reader.get_ue()?,
            el_bit_depth_minus8: reader.get_ue()?,
            hdr_bit_depth_minus8: reader.get_ue()?,
            disable_residual_flag: reader.get()?,
            ..Default::default()
        };

        let coefficient_log2_denom_length = meta.coefficient_log2_denom as u32;

        for cmp in 0..NUM_COMPONENTS {
            meta.num_pivots_minus2[cmp] = reader.get_ue()?;

            meta.pred_pivot_value[cmp]
                .resize_with((meta.num_pivots_minus2[cmp] as usize) + 2, Default::default);

            for pivot_idx in 0..(meta.num_pivots_minus2[cmp] as usize) + 2 {
                meta.pred_pivot_value[cmp][pivot_idx] =
                    reader.get_n((meta.el_bit_depth_minus8 as u32) + 8)?;
            }
        }

        for cmp in 0..NUM_COMPONENTS {
            meta.mapping_idc[cmp]
                .resize_with((meta.num_pivots_minus2[cmp] as usize) + 1, Default::default);
            meta.poly_order_minus1[cmp]
                .resize_with((meta.num_pivots_minus2[cmp] as usize) + 1, Default::default);
            meta.poly_coef_int[cmp]
                .resize_with((meta.num_pivots_minus2[cmp] as usize) + 1, Default::default);
            meta.poly_coef[cmp]
                .resize_with((meta.num_pivots_minus2[cmp] as usize) + 1, Default::default);

            meta.mmr_order_minus1[cmp]
                .resize_with((meta.num_pivots_minus2[cmp] as usize) + 1, Default::default);
            meta.mmr_constant_int[cmp]
                .resize_with((meta.num_pivots_minus2[cmp] as usize) + 1, Default::default);
            meta.mmr_constant[cmp]
                .resize_with((meta.num_pivots_minus2[cmp] as usize) + 1, Default::default);
            meta.mmr_coef_int[cmp]
                .resize_with((meta.num_pivots_minus2[cmp] as usize) + 1, Default::default);
            meta.mmr_coef[cmp]
                .resize_with((meta.num_pivots_minus2[cmp] as usize) + 1, Default::default);

            for pivot_idx in 0..(meta.num_pivots_minus2[cmp] as usize) + 1 {
                meta.mapping_idc[cmp][pivot_idx] = reader.get_ue()?;

                // MAPPING_POLYNOMIAL
                if meta.mapping_idc[cmp][pivot_idx] == 0 {
                    meta.poly_order_minus1[cmp][pivot_idx] = reader.get_ue()?;

                    meta.poly_coef_int[cmp][pivot_idx].resize_with(
                        (meta.poly_order_minus1[cmp][pivot_idx] as usize) + 2,
                        Default::default,
                    );
                    meta.poly_coef[cmp][pivot_idx].resize_with(
                        (meta.poly_order_minus1[cmp][pivot_idx] as usize) + 2,
                        Default::default,
                    );

                    for i in 0..=(meta.poly_order_minus1[cmp][pivot_idx] as usize) + 1 {
                        meta.poly_coef_int[cmp][pivot_idx][i] = reader.get_se()?;
                        meta.poly_coef[cmp][pivot_idx][i] =
                            reader.get_n(coefficient_log2_denom_length)?;
                    }
                } else if meta.mapping_idc[cmp][pivot_idx] == 1 {
                    // MAPPING_MMR

                    meta.mmr_order_minus1[cmp][pivot_idx] = reader.get_n(2)?;
                    meta.mmr_constant_int[cmp][pivot_idx] = reader.get_se()?;
                    meta.mmr_constant[cmp][pivot_idx] =
                        reader.get_n(coefficient_log2_denom_length)?;

                    meta.mmr_coef_int[cmp][pivot_idx].resize_with(
                        (meta.mmr_order_minus1[cmp][pivot_idx] as usize) + 2,
                        Default::default,
                    );
                    meta.mmr_coef[cmp][pivot_idx].resize_with(
                        (meta.mmr_order_minus1[cmp][pivot_idx] as usize) + 2,
                        Default::default,
                    );

                    for i in 1..=(meta.mmr_order_minus1[cmp][pivot_idx] as usize) + 1 {
                        meta.mmr_coef_int[cmp][pivot_idx][i].resize_with(8, Default::default);
                        meta.mmr_coef[cmp][pivot_idx][i].resize_with(8, Default::default);

                        for j in 0..7_usize {
                            meta.mmr_coef_int[cmp][pivot_idx][i][j] = reader.get_se()?;
                            meta.mmr_coef[cmp][pivot_idx][i][j] =
                                reader.get_n(coefficient_log2_denom_length)?;
                        }
                    }
                }
            }
        }

        if !meta.disable_residual_flag {
            for cmp in 0..NUM_COMPONENTS {
                meta.nlq_offset[cmp] = reader.get_n((meta.el_bit_depth_minus8 as u32) + 8)?;
                meta.hdr_in_max_int[cmp] = reader.get_ue()?;
                meta.hdr_in_max[cmp] = reader.get_n(coefficient_log2_denom_length)?;
                meta.linear_deadzone_slope_int[cmp] = reader.get_ue()?;
                meta.linear_deadzone_slope[cmp] = reader.get_n(coefficient_log2_denom_length)?;
                meta.linear_deadzone_threshold_int[cmp] = reader.get_ue()?;
                meta.linear_deadzone_threshold[cmp] =
                    reader.get_n(coefficient_log2_denom_length)?;
            }
        }

        Ok(UserDataTypeStruct::CMData(Box::new(meta)))
    }
}
