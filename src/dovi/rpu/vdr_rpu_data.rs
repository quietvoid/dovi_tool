use crate::bits::bitvec_reader::BitVecReader;

use super::RpuNal;

#[derive(Debug, Default)]
pub struct VdrRpuData {
    mapping_idc: Vec<Vec<u64>>,
    mapping_param_pred_flag: Vec<Vec<bool>>,
    num_mapping_param_predictors: Vec<Vec<u64>>,
    diff_pred_part_idx_mapping_minus1: Vec<Vec<u64>>,
    poly_order_minus1: Vec<Vec<u64>>,
    linear_interp_flag: Vec<Vec<bool>>,
    pred_linear_interp_value_int: Vec<Vec<u64>>,
    pred_linear_interp_value: Vec<Vec<u64>>,
    poly_coef_int: Vec<Vec<i64>>,
    poly_coef: Vec<Vec<u64>>,
    mmr_order_minus1: Vec<Vec<u8>>,
    mmr_constant_int: Vec<Vec<i64>>,
    mmr_constant: Vec<Vec<u64>>,
    mmr_coef_int: Vec<Vec<Vec<Vec<i64>>>>,
    mmr_coef: Vec<Vec<Vec<Vec<u64>>>>,
}

#[derive(Debug, Default)]
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

impl VdrRpuData {
    pub fn vdr_rpu_data_payload(reader: &mut BitVecReader, mut rpu_nal: &mut RpuNal) {
        let vdr_rpu_data = VdrRpuData::rpu_data_mapping(reader, rpu_nal);
        let nlq_data = NlqData::rpu_data_nlq(reader, rpu_nal);

        rpu_nal.vdr_rpu_data = Some(vdr_rpu_data);
        rpu_nal.nlq_data = Some(nlq_data);
    }

    pub fn rpu_data_mapping(reader: &mut BitVecReader, rpu_nal: &mut RpuNal) -> VdrRpuData {
        let num_cmps = 3;

        let mut data = VdrRpuData::default();

        let coefficient_log2_denom_length = if rpu_nal.coefficient_data_type == 0 {
            rpu_nal.coefficient_log2_denom as usize
        } else if rpu_nal.coefficient_data_type == 1 {
            32
        } else {
            panic!("Invalid coefficient_data_type value!");
        };

        // rpu_data_mapping_param

        for cmp in 0..num_cmps {
            let pivot_idx_count = (rpu_nal.num_pivots_minus_2[cmp] + 1) as usize;
            let mut predictors = 0;

            data.mapping_idc.push(vec![0; pivot_idx_count]);
            data.num_mapping_param_predictors
                .push(vec![0; pivot_idx_count]);
            data.mapping_param_pred_flag
                .push(vec![false; pivot_idx_count]);
            data.diff_pred_part_idx_mapping_minus1
                .push(vec![0; pivot_idx_count]);

            // rpu_data_mapping_param()
            data.poly_order_minus1.push(vec![0; pivot_idx_count]);
            data.linear_interp_flag.push(vec![false; pivot_idx_count]);
            data.pred_linear_interp_value_int
                .push(vec![0; pivot_idx_count]);
            data.pred_linear_interp_value.push(vec![0; pivot_idx_count]);
            data.poly_coef_int.push(vec![0; pivot_idx_count]);
            data.poly_coef.push(vec![0; pivot_idx_count]);
            data.mmr_order_minus1.push(vec![0; pivot_idx_count]);
            data.mmr_constant_int.push(vec![0; pivot_idx_count]);
            data.mmr_constant.push(vec![0; pivot_idx_count]);

            data.mmr_coef_int.push(vec![vec![]; pivot_idx_count]);
            data.mmr_coef.push(vec![vec![]; pivot_idx_count]);

            for pivot_idx in 0..pivot_idx_count {
                data.mapping_idc[cmp][pivot_idx] = reader.get_ue();

                if data.num_mapping_param_predictors[cmp][pivot_idx] > 0 {
                    data.mapping_param_pred_flag[cmp][pivot_idx] = reader.get();
                } else {
                    data.mapping_param_pred_flag[cmp][pivot_idx] = false;
                }

                // Incremented after mapping_idc if mapping_param_pred_flag is 0
                if !data.mapping_param_pred_flag[cmp][pivot_idx] {
                    data.num_mapping_param_predictors[cmp][pivot_idx] = predictors;
                    predictors += 1;
                }

                // == 0
                if !data.mapping_param_pred_flag[cmp][pivot_idx] {
                    // rpu_data_mapping_param()

                    // MAPPING_POLYNOMIAL
                    if data.mapping_idc[cmp][pivot_idx] == 0 {
                        data.poly_order_minus1[cmp][pivot_idx] = reader.get_ue();

                        if data.poly_order_minus1[cmp][pivot_idx] == 0 {
                            data.linear_interp_flag[cmp][pivot_idx] = reader.get();
                        }

                        // Linear interpolation
                        if data.poly_order_minus1[cmp][pivot_idx] == 0
                            && data.linear_interp_flag[cmp][pivot_idx]
                        {
                            if rpu_nal.coefficient_data_type == 0 {
                                data.pred_linear_interp_value_int[cmp][pivot_idx] = reader.get_ue();
                            }

                            data.pred_linear_interp_value[cmp][pivot_idx] =
                                reader.get_n(coefficient_log2_denom_length);

                            if pivot_idx as u64 == rpu_nal.num_pivots_minus_2[cmp] {
                                if rpu_nal.coefficient_data_type == 0 {
                                    data.pred_linear_interp_value_int[cmp][pivot_idx + 1] =
                                        reader.get_ue();
                                }

                                data.pred_linear_interp_value[cmp][pivot_idx + 1] =
                                    reader.get_n(coefficient_log2_denom_length);
                            }
                        } else {
                            for _ in 0..=data.poly_order_minus1[cmp][pivot_idx] + 1 {
                                if rpu_nal.coefficient_data_type == 0 {
                                    data.poly_coef_int[cmp][pivot_idx] = reader.get_se();
                                }

                                data.poly_coef[cmp][pivot_idx] =
                                    reader.get_n(coefficient_log2_denom_length);
                            }
                        }
                    } else if data.mapping_idc[cmp][pivot_idx] == 1 {
                        // MAPPING_MMR
                        data.mmr_order_minus1[cmp][pivot_idx] = reader.get_n(2);

                        assert!(data.mmr_order_minus1[cmp][pivot_idx] <= 2);

                        data.mmr_coef[cmp][pivot_idx] =
                            vec![vec![0; 7]; data.mmr_order_minus1[cmp][pivot_idx] as usize + 2];
                        data.mmr_coef_int[cmp][pivot_idx] =
                            vec![vec![0; 7]; data.mmr_order_minus1[cmp][pivot_idx] as usize + 2];

                        if rpu_nal.coefficient_data_type == 0 {
                            data.mmr_constant_int[cmp][pivot_idx] = reader.get_se();
                        }

                        data.mmr_constant[cmp][pivot_idx] =
                            reader.get_n(coefficient_log2_denom_length);

                        for i in 1..=data.mmr_order_minus1[cmp][pivot_idx] as usize + 1 {
                            for j in 0..7 as usize {
                                if rpu_nal.coefficient_data_type == 0 {
                                    data.mmr_coef_int[cmp][pivot_idx][i][j] = reader.get_se();
                                }

                                data.mmr_coef[cmp][pivot_idx][i][j] =
                                    reader.get_n(coefficient_log2_denom_length);
                            }
                        }
                    }
                } else if data.num_mapping_param_predictors[cmp][pivot_idx] > 1 {
                    data.diff_pred_part_idx_mapping_minus1[cmp][pivot_idx] = reader.get_ue();
                }
            }
        }

        data.validate();

        data
    }
}

impl NlqData {
    pub fn rpu_data_nlq(reader: &mut BitVecReader, rpu_nal: &mut RpuNal) -> NlqData {
        let num_cmps = 3;
        let pivot_idx_count = (rpu_nal.nlq_num_pivots_minus2 + 1) as usize;

        let mut data = NlqData::default();

        let coefficient_log2_denom_length = if rpu_nal.coefficient_data_type == 0 {
            rpu_nal.coefficient_log2_denom as usize
        } else if rpu_nal.coefficient_data_type == 1 {
            32
        } else {
            panic!("Invalid coefficient_data_type value!");
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

            let mut predictors = 0;

            for cmp in 0..num_cmps {
                if data.num_nlq_param_predictors[pivot_idx][cmp] > 0 {
                    data.nlq_param_pred_flag[pivot_idx][cmp] = reader.get();
                } else {
                    data.nlq_param_pred_flag[pivot_idx][cmp] = false;
                }

                // Incremented if nlq_param_pred_flag is 0
                if !data.nlq_param_pred_flag[pivot_idx][cmp] {
                    data.num_nlq_param_predictors[pivot_idx][cmp] = predictors;
                    predictors += 1;
                }

                if !data.nlq_param_pred_flag[pivot_idx][cmp] {
                    // rpu_data_nlq_param

                    data.nlq_offset[pivot_idx][cmp] =
                        reader.get_n((rpu_nal.el_bit_depth_minus8 + 8) as usize);

                    if rpu_nal.coefficient_data_type == 0 {
                        data.vdr_in_max_int[pivot_idx][cmp] = reader.get_ue();
                    }

                    data.vdr_in_max[pivot_idx][cmp] = reader.get_n(coefficient_log2_denom_length);

                    // NLQ_LINEAR_DZ
                    if rpu_nal.nlq_method_idc == 0 {
                        if rpu_nal.coefficient_data_type == 0 {
                            data.linear_deadzone_slope_int[pivot_idx][cmp] = reader.get_ue();
                        }

                        data.linear_deadzone_slope[pivot_idx][cmp] =
                            reader.get_n(coefficient_log2_denom_length);

                        if rpu_nal.coefficient_data_type == 0 {
                            data.linear_deadzone_threshold_int[pivot_idx][cmp] = reader.get_ue();
                        }

                        data.linear_deadzone_threshold[pivot_idx][cmp] =
                            reader.get_n(coefficient_log2_denom_length);
                    }
                } else if data.num_nlq_param_predictors[pivot_idx][cmp] > 1 {
                    data.diff_pred_part_idx_nlq_minus1[pivot_idx][cmp] = reader.get_ue();
                }
            }
        }

        data
    }
}
