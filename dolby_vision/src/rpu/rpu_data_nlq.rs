use anyhow::{bail, Result};
use bitvec_helpers::{bitvec_reader::BitVecReader, bitvec_writer::BitVecWriter};

#[cfg(feature = "serde_feature")]
use serde::Serialize;

use super::rpu_data_header::RpuDataHeader;

use super::NUM_COMPONENTS;

#[derive(Debug, Default)]
#[cfg_attr(feature = "serde_feature", derive(Serialize))]
pub struct RpuDataNlq {
    pub num_nlq_param_predictors: Vec<[u64; NUM_COMPONENTS]>,
    pub nlq_param_pred_flag: Vec<[bool; NUM_COMPONENTS]>,
    pub diff_pred_part_idx_nlq_minus1: Vec<[u64; NUM_COMPONENTS]>,
    pub nlq_offset: Vec<[u64; NUM_COMPONENTS]>,
    pub vdr_in_max_int: Vec<[u64; NUM_COMPONENTS]>,
    pub vdr_in_max: Vec<[u64; NUM_COMPONENTS]>,
    pub linear_deadzone_slope_int: Vec<[u64; NUM_COMPONENTS]>,
    pub linear_deadzone_slope: Vec<[u64; NUM_COMPONENTS]>,
    pub linear_deadzone_threshold_int: Vec<[u64; NUM_COMPONENTS]>,
    pub linear_deadzone_threshold: Vec<[u64; NUM_COMPONENTS]>,
}

impl RpuDataNlq {
    pub fn parse(reader: &mut BitVecReader, header: &mut RpuDataHeader) -> Result<RpuDataNlq> {
        let pivot_idx_count = if let Some(nlq_num_pivots_minus2) = header.nlq_num_pivots_minus2 {
            nlq_num_pivots_minus2 as usize + 1
        } else {
            bail!("Shouldn't be in NLQ if not profile 7!");
        };

        let mut data = RpuDataNlq::default();

        let coefficient_log2_denom_length = if header.coefficient_data_type == 0 {
            header.coefficient_log2_denom as usize
        } else if header.coefficient_data_type == 1 {
            32
        } else {
            bail!(
                "Invalid coefficient_data_type value: {}",
                header.coefficient_data_type
            );
        };

        data.num_nlq_param_predictors
            .resize_with(pivot_idx_count, Default::default);
        data.nlq_param_pred_flag
            .resize_with(pivot_idx_count, Default::default);

        for pivot_idx in 0..pivot_idx_count {
            for cmp in 0..NUM_COMPONENTS {
                if data.num_nlq_param_predictors[pivot_idx][cmp] > 0 {
                    data.nlq_param_pred_flag[pivot_idx][cmp] = reader.get()?;
                } else {
                    data.nlq_param_pred_flag[pivot_idx][cmp] = false;
                }

                if !data.nlq_param_pred_flag[pivot_idx][cmp] {
                    // rpu_data_nlq_param

                    if data.nlq_offset.is_empty() {
                        data.nlq_offset
                            .resize_with(pivot_idx_count, Default::default);
                        data.vdr_in_max
                            .resize_with(pivot_idx_count, Default::default);
                    }

                    data.nlq_offset[pivot_idx][cmp] =
                        reader.get_n((header.el_bit_depth_minus8 + 8) as usize);

                    if header.coefficient_data_type == 0 {
                        if data.vdr_in_max_int.is_empty() {
                            data.vdr_in_max_int
                                .resize_with(pivot_idx_count, Default::default);
                        }

                        data.vdr_in_max_int[pivot_idx][cmp] = reader.get_ue()?;
                    }

                    data.vdr_in_max[pivot_idx][cmp] = reader.get_n(coefficient_log2_denom_length);

                    // NLQ_LINEAR_DZ
                    if let Some(nlq_method_idc) = header.nlq_method_idc {
                        if nlq_method_idc == 0 {
                            if data.linear_deadzone_slope.is_empty() {
                                data.linear_deadzone_slope
                                    .resize_with(pivot_idx_count, Default::default);
                                data.linear_deadzone_threshold
                                    .resize_with(pivot_idx_count, Default::default);
                            }

                            if header.coefficient_data_type == 0 {
                                if data.linear_deadzone_slope_int.is_empty() {
                                    data.linear_deadzone_slope_int
                                        .resize_with(pivot_idx_count, Default::default);
                                }

                                data.linear_deadzone_slope_int[pivot_idx][cmp] = reader.get_ue()?;
                            }

                            data.linear_deadzone_slope[pivot_idx][cmp] =
                                reader.get_n(coefficient_log2_denom_length);

                            if header.coefficient_data_type == 0 {
                                if data.linear_deadzone_threshold_int.is_empty() {
                                    data.linear_deadzone_threshold_int
                                        .resize_with(pivot_idx_count, Default::default);
                                }

                                data.linear_deadzone_threshold_int[pivot_idx][cmp] =
                                    reader.get_ue()?;
                            }

                            data.linear_deadzone_threshold[pivot_idx][cmp] =
                                reader.get_n(coefficient_log2_denom_length);
                        }
                    }
                } else if data.num_nlq_param_predictors[pivot_idx][cmp] > 1 {
                    if data.diff_pred_part_idx_nlq_minus1.is_empty() {
                        data.diff_pred_part_idx_nlq_minus1
                            .resize_with(pivot_idx_count, Default::default);
                    }

                    data.diff_pred_part_idx_nlq_minus1[pivot_idx][cmp] = reader.get_ue()?;
                }
            }
        }

        Ok(data)
    }

    pub fn convert_to_mel(&mut self) {
        // Set to 0
        self.nlq_offset.iter_mut().for_each(|v| {
            v.iter_mut().for_each(|v2| *v2 = 0);
        });

        // Set to 1
        self.vdr_in_max_int.iter_mut().for_each(|v| {
            v.iter_mut().for_each(|v2| *v2 = 1);
        });

        // Set to 0
        self.vdr_in_max.iter_mut().for_each(|v| {
            v.iter_mut().for_each(|v2| *v2 = 0);
        });

        self.linear_deadzone_slope_int.iter_mut().for_each(|v| {
            v.iter_mut().for_each(|v2| *v2 = 0);
        });

        self.linear_deadzone_slope.iter_mut().for_each(|v| {
            v.iter_mut().for_each(|v2| *v2 = 0);
        });

        self.linear_deadzone_threshold_int.iter_mut().for_each(|v| {
            v.iter_mut().for_each(|v2| *v2 = 0);
        });

        self.linear_deadzone_threshold.iter_mut().for_each(|v| {
            v.iter_mut().for_each(|v2| *v2 = 0);
        });
    }

    pub fn write(&self, writer: &mut BitVecWriter, header: &RpuDataHeader) -> Result<()> {
        let pivot_idx_count = if let Some(nlq_num_pivots_minus2) = header.nlq_num_pivots_minus2 {
            nlq_num_pivots_minus2 as usize + 1
        } else {
            bail!("Shouldn't be in NLQ if not profile 7!");
        };
        let coefficient_log2_denom_length = if header.coefficient_data_type == 0 {
            header.coefficient_log2_denom as usize
        } else if header.coefficient_data_type == 1 {
            32
        } else {
            bail!(
                "Invalid coefficient_data_type value: {}",
                header.coefficient_data_type
            );
        };

        for pivot_idx in 0..pivot_idx_count {
            for cmp in 0..NUM_COMPONENTS {
                if self.num_nlq_param_predictors[pivot_idx][cmp] > 0 {
                    writer.write(self.nlq_param_pred_flag[pivot_idx][cmp]);
                }

                if !self.nlq_param_pred_flag[pivot_idx][cmp] {
                    // rpu_data_nlq_param

                    writer.write_n(
                        &self.nlq_offset[pivot_idx][cmp].to_be_bytes(),
                        (header.el_bit_depth_minus8 + 8) as usize,
                    );

                    if header.coefficient_data_type == 0 {
                        writer.write_ue(self.vdr_in_max_int[pivot_idx][cmp]);
                    }

                    writer.write_n(
                        &self.vdr_in_max[pivot_idx][cmp].to_be_bytes(),
                        coefficient_log2_denom_length,
                    );

                    if let Some(nlq_method_idc) = header.nlq_method_idc {
                        if nlq_method_idc == 0 {
                            // NLQ_LINEAR_DZ
                            if header.coefficient_data_type == 0 {
                                writer.write_ue(self.linear_deadzone_slope_int[pivot_idx][cmp]);
                            }

                            writer.write_n(
                                &self.linear_deadzone_slope[pivot_idx][cmp].to_be_bytes(),
                                coefficient_log2_denom_length,
                            );

                            if header.coefficient_data_type == 0 {
                                writer.write_ue(self.linear_deadzone_slope_int[pivot_idx][cmp]);
                            }

                            writer.write_n(
                                &self.linear_deadzone_threshold[pivot_idx][cmp].to_be_bytes(),
                                coefficient_log2_denom_length,
                            );
                        }
                    }
                } else if self.num_nlq_param_predictors[pivot_idx][cmp] > 1 {
                    writer.write_ue(self.diff_pred_part_idx_nlq_minus1[pivot_idx][cmp]);
                }
            }
        }

        Ok(())
    }

    pub fn mel_default() -> Self {
        Self {
            num_nlq_param_predictors: vec![[0; NUM_COMPONENTS]],
            nlq_param_pred_flag: vec![[false; NUM_COMPONENTS]],
            diff_pred_part_idx_nlq_minus1: vec![[0; NUM_COMPONENTS]],
            nlq_offset: vec![[0; NUM_COMPONENTS]],
            vdr_in_max_int: vec![[1; NUM_COMPONENTS]],
            vdr_in_max: vec![[0; NUM_COMPONENTS]],
            linear_deadzone_slope_int: vec![[0; NUM_COMPONENTS]],
            linear_deadzone_slope: vec![[0; NUM_COMPONENTS]],
            linear_deadzone_threshold_int: vec![[0; NUM_COMPONENTS]],
            linear_deadzone_threshold: vec![[0; NUM_COMPONENTS]],
        }
    }
}
