use anyhow::{bail, Result};
use bitvec_helpers::{bitvec_reader::BitVecReader, bitvec_writer::BitVecWriter};

#[cfg(feature = "serde_feature")]
use serde::Serialize;

use super::rpu_data_header::RpuDataHeader;

#[derive(Debug, Default)]
#[cfg_attr(feature = "serde_feature", derive(Serialize))]
pub struct NlqData {
    num_nlq_param_predictors: Vec<Vec<u64>>,
    nlq_param_pred_flag: Vec<Vec<bool>>,
    diff_pred_part_idx_nlq_minus1: Vec<Vec<u64>>,
    nlq_offset: Vec<Vec<u64>>,
    vdr_in_max_int: Vec<Vec<u64>>,
    vdr_in_max: Vec<Vec<u64>>,
    linear_deadzone_slope_int: Vec<Vec<u64>>,
    linear_deadzone_slope: Vec<Vec<u64>>,
    linear_deadzone_threshold_int: Vec<Vec<u64>>,
    linear_deadzone_threshold: Vec<Vec<u64>>,
}

impl NlqData {
    pub fn rpu_data_nlq(reader: &mut BitVecReader, header: &mut RpuDataHeader) -> Result<NlqData> {
        let num_cmps = 3;
        let pivot_idx_count = if let Some(nlq_num_pivots_minus2) = header.nlq_num_pivots_minus2 {
            nlq_num_pivots_minus2 as usize + 1
        } else {
            bail!("Shouldn't be in NLQ if not profile 7!");
        };

        let mut data = NlqData::default();

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
            data.num_nlq_param_predictors.push(vec![0; num_cmps]);
            data.nlq_param_pred_flag.push(vec![false; num_cmps]);
            data.diff_pred_part_idx_nlq_minus1.push(vec![0; num_cmps]);

            data.nlq_offset.push(vec![0; num_cmps]);
            data.vdr_in_max_int.push(vec![0; num_cmps]);
            data.vdr_in_max.push(vec![0; num_cmps]);

            data.linear_deadzone_slope_int.push(vec![0; num_cmps]);
            data.linear_deadzone_slope.push(vec![0; num_cmps]);
            data.linear_deadzone_threshold_int.push(vec![0; num_cmps]);
            data.linear_deadzone_threshold.push(vec![0; num_cmps]);

            for cmp in 0..num_cmps {
                if data.num_nlq_param_predictors[pivot_idx][cmp] > 0 {
                    data.nlq_param_pred_flag[pivot_idx][cmp] = reader.get();
                } else {
                    data.nlq_param_pred_flag[pivot_idx][cmp] = false;
                }

                if !data.nlq_param_pred_flag[pivot_idx][cmp] {
                    // rpu_data_nlq_param

                    data.nlq_offset[pivot_idx][cmp] =
                        reader.get_n((header.el_bit_depth_minus8 + 8) as usize);

                    if header.coefficient_data_type == 0 {
                        data.vdr_in_max_int[pivot_idx][cmp] = reader.get_ue();
                    }

                    data.vdr_in_max[pivot_idx][cmp] = reader.get_n(coefficient_log2_denom_length);

                    // NLQ_LINEAR_DZ
                    if let Some(nlq_method_idc) = header.nlq_method_idc {
                        if nlq_method_idc == 0 {
                            if header.coefficient_data_type == 0 {
                                data.linear_deadzone_slope_int[pivot_idx][cmp] = reader.get_ue();
                            }

                            data.linear_deadzone_slope[pivot_idx][cmp] =
                                reader.get_n(coefficient_log2_denom_length);

                            if header.coefficient_data_type == 0 {
                                data.linear_deadzone_threshold_int[pivot_idx][cmp] =
                                    reader.get_ue();
                            }

                            data.linear_deadzone_threshold[pivot_idx][cmp] =
                                reader.get_n(coefficient_log2_denom_length);
                        }
                    }
                } else if data.num_nlq_param_predictors[pivot_idx][cmp] > 1 {
                    data.diff_pred_part_idx_nlq_minus1[pivot_idx][cmp] = reader.get_ue();
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
        let num_cmps = 3;
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
            for cmp in 0..num_cmps {
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
}
