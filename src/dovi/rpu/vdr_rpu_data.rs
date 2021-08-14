use super::{BitVecReader, BitVecWriter, RpuDataHeader};
use serde::Serialize;

#[derive(Debug, Default, Serialize)]
pub struct VdrRpuData {
    mapping_idc: Vec<Vec<u64>>,
    mapping_param_pred_flag: Vec<Vec<bool>>,
    num_mapping_param_predictors: Vec<Vec<u64>>,
    diff_pred_part_idx_mapping_minus1: Vec<Vec<u64>>,
    poly_order_minus1: Vec<Vec<u64>>,
    linear_interp_flag: Vec<Vec<bool>>,
    pred_linear_interp_value_int: Vec<Vec<u64>>,
    pred_linear_interp_value: Vec<Vec<u64>>,
    poly_coef_int: Vec<Vec<Vec<i64>>>,
    poly_coef: Vec<Vec<Vec<u64>>>,
    mmr_order_minus1: Vec<Vec<u8>>,
    mmr_constant_int: Vec<Vec<i64>>,
    mmr_constant: Vec<Vec<u64>>,
    mmr_coef_int: Vec<Vec<Vec<Vec<i64>>>>,
    mmr_coef: Vec<Vec<Vec<Vec<u64>>>>,
}

#[derive(Debug, Default, Serialize)]
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
    pub fn vdr_rpu_data_payload(
        reader: &mut BitVecReader,
        header: &mut RpuDataHeader,
    ) -> (Option<VdrRpuData>, Option<NlqData>) {
        let vdr_rpu_data = VdrRpuData::rpu_data_mapping(reader, header);

        if header.nlq_method_idc.is_some() {
            let nlq_data = NlqData::rpu_data_nlq(reader, header);
            (Some(vdr_rpu_data), Some(nlq_data))
        } else {
            (Some(vdr_rpu_data), None)
        }
    }

    pub fn rpu_data_mapping(reader: &mut BitVecReader, header: &mut RpuDataHeader) -> VdrRpuData {
        let num_cmps = 3;

        let mut data = VdrRpuData::default();

        let coefficient_log2_denom_length = if header.coefficient_data_type == 0 {
            header.coefficient_log2_denom as usize
        } else if header.coefficient_data_type == 1 {
            32
        } else {
            panic!("Invalid coefficient_data_type value!");
        };

        // rpu_data_mapping_param

        for cmp in 0..num_cmps {
            let pivot_idx_count = (header.num_pivots_minus_2[cmp] + 1) as usize;
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

            data.poly_coef_int.push(vec![vec![]; pivot_idx_count]);
            data.poly_coef.push(vec![vec![]; pivot_idx_count]);

            data.pred_linear_interp_value_int
                .push(vec![0; pivot_idx_count]);
            data.pred_linear_interp_value.push(vec![0; pivot_idx_count]);
            data.mmr_order_minus1.push(vec![0; pivot_idx_count]);
            data.mmr_constant_int.push(vec![0; pivot_idx_count]);
            data.mmr_constant.push(vec![0; pivot_idx_count]);

            data.mmr_coef_int.push(vec![vec![]; pivot_idx_count]);
            data.mmr_coef.push(vec![vec![]; pivot_idx_count]);

            for pivot_idx in 0..pivot_idx_count {
                data.mapping_idc[cmp][pivot_idx] = reader.get_ue();

                // Dolby pls. Guessing this is what they mean by "new parameters"
                if pivot_idx > 0
                    && data.mapping_idc[cmp][pivot_idx] != data.mapping_idc[cmp][pivot_idx - 1]
                {
                    predictors += 1;
                    data.num_mapping_param_predictors[cmp][pivot_idx] = predictors;
                } else {
                    predictors = 0;
                }

                if data.num_mapping_param_predictors[cmp][pivot_idx] > 0 {
                    data.mapping_param_pred_flag[cmp][pivot_idx] = reader.get();
                } else {
                    data.mapping_param_pred_flag[cmp][pivot_idx] = false;
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
                            if header.coefficient_data_type == 0 {
                                data.pred_linear_interp_value_int[cmp][pivot_idx] = reader.get_ue();
                            }

                            data.pred_linear_interp_value[cmp][pivot_idx] =
                                reader.get_n(coefficient_log2_denom_length);

                            if pivot_idx as u64 == header.num_pivots_minus_2[cmp] {
                                if header.coefficient_data_type == 0 {
                                    data.pred_linear_interp_value_int[cmp][pivot_idx + 1] =
                                        reader.get_ue();
                                }

                                data.pred_linear_interp_value[cmp][pivot_idx + 1] =
                                    reader.get_n(coefficient_log2_denom_length);
                            }
                        } else {
                            let poly_coef_count =
                                data.poly_order_minus1[cmp][pivot_idx] as usize + 1;

                            data.poly_coef_int[cmp][pivot_idx] = vec![0; poly_coef_count + 1];
                            data.poly_coef[cmp][pivot_idx] = vec![0; poly_coef_count + 1];

                            for i in 0..=poly_coef_count {
                                if header.coefficient_data_type == 0 {
                                    data.poly_coef_int[cmp][pivot_idx][i] = reader.get_se();
                                }

                                data.poly_coef[cmp][pivot_idx][i] =
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

                        if header.coefficient_data_type == 0 {
                            data.mmr_constant_int[cmp][pivot_idx] = reader.get_se();
                        }

                        data.mmr_constant[cmp][pivot_idx] =
                            reader.get_n(coefficient_log2_denom_length);

                        for i in 1..=data.mmr_order_minus1[cmp][pivot_idx] as usize + 1 {
                            for j in 0..7_usize {
                                if header.coefficient_data_type == 0 {
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

    pub fn validate(&self) {}

    pub fn write(&self, writer: &mut BitVecWriter, header: &RpuDataHeader) {
        let coefficient_log2_denom_length = if header.coefficient_data_type == 0 {
            header.coefficient_log2_denom as usize
        } else if header.coefficient_data_type == 1 {
            32
        } else {
            panic!("Invalid coefficient_data_type value!");
        };

        // rpu_data_mapping_param

        self.mapping_idc
            .iter()
            .enumerate()
            .for_each(|(cmp_idx, mapping_idc)| {
                let pivot_idx_count = (header.num_pivots_minus_2[cmp_idx] + 1) as usize;

                for (pivot_idx, mapping_idc_value) in
                    mapping_idc.iter().enumerate().take(pivot_idx_count)
                {
                    writer.write_ue(*mapping_idc_value);

                    if self.num_mapping_param_predictors[cmp_idx][pivot_idx] > 0 {
                        writer.write(self.mapping_param_pred_flag[cmp_idx][pivot_idx]);
                    }

                    // == 0
                    if !self.mapping_param_pred_flag[cmp_idx][pivot_idx] {
                        // rpu_data_mapping_param()

                        // MAPPING_POLYNOMIAL
                        if mapping_idc[pivot_idx] == 0 {
                            writer.write_ue(self.poly_order_minus1[cmp_idx][pivot_idx]);

                            if self.poly_order_minus1[cmp_idx][pivot_idx] == 0 {
                                writer.write(self.linear_interp_flag[cmp_idx][pivot_idx]);
                            }

                            // Linear interpolation
                            if self.poly_order_minus1[cmp_idx][pivot_idx] == 0
                                && self.linear_interp_flag[cmp_idx][pivot_idx]
                            {
                                if header.coefficient_data_type == 0 {
                                    writer.write_ue(
                                        self.pred_linear_interp_value_int[cmp_idx][pivot_idx],
                                    );
                                }

                                writer.write_n(
                                    &self.pred_linear_interp_value[cmp_idx][pivot_idx]
                                        .to_be_bytes(),
                                    coefficient_log2_denom_length,
                                );

                                if pivot_idx as u64 == header.num_pivots_minus_2[cmp_idx] {
                                    if header.coefficient_data_type == 0 {
                                        writer.write_ue(
                                            self.pred_linear_interp_value_int[cmp_idx]
                                                [pivot_idx + 1],
                                        );
                                    }

                                    writer.write_n(
                                        &self.pred_linear_interp_value[cmp_idx][pivot_idx + 1]
                                            .to_be_bytes(),
                                        coefficient_log2_denom_length,
                                    );
                                }
                            } else {
                                for i in 0..=self.poly_order_minus1[cmp_idx][pivot_idx] as usize + 1
                                {
                                    if header.coefficient_data_type == 0 {
                                        writer.write_se(self.poly_coef_int[cmp_idx][pivot_idx][i]);
                                    }

                                    writer.write_n(
                                        &self.poly_coef[cmp_idx][pivot_idx][i].to_be_bytes(),
                                        coefficient_log2_denom_length,
                                    );
                                }
                            }
                        } else if mapping_idc[pivot_idx] == 1 {
                            // MAPPING_MMR
                            writer.write_n(
                                &self.mmr_order_minus1[cmp_idx][pivot_idx].to_be_bytes(),
                                2,
                            );

                            if header.coefficient_data_type == 0 {
                                writer.write_se(self.mmr_constant_int[cmp_idx][pivot_idx]);
                            }

                            writer.write_n(
                                &self.mmr_constant[cmp_idx][pivot_idx].to_be_bytes(),
                                coefficient_log2_denom_length,
                            );

                            for i in 1..=self.mmr_order_minus1[cmp_idx][pivot_idx] as usize + 1 {
                                for j in 0..7_usize {
                                    if header.coefficient_data_type == 0 {
                                        writer
                                            .write_se(self.mmr_coef_int[cmp_idx][pivot_idx][i][j]);
                                    }

                                    writer.write_n(
                                        &self.mmr_coef[cmp_idx][pivot_idx][i][j].to_be_bytes(),
                                        coefficient_log2_denom_length,
                                    );
                                }
                            }
                        }
                    } else if self.num_mapping_param_predictors[cmp_idx][pivot_idx] > 1 {
                        writer.write_ue(self.diff_pred_part_idx_mapping_minus1[cmp_idx][pivot_idx]);
                    }
                }
            });
    }

    pub fn p5_to_p81(&mut self) {
        self.mapping_idc.iter_mut().for_each(|v| {
            v.truncate(1);
            v[0] = 0;
        });

        self.mapping_param_pred_flag.iter_mut().for_each(|v| {
            v.truncate(1);
            v[0] = false;
        });

        self.num_mapping_param_predictors.iter_mut().for_each(|v| {
            v.truncate(1);
            v[0] = 0;
        });

        self.diff_pred_part_idx_mapping_minus1
            .iter_mut()
            .for_each(|v| {
                v.truncate(1);
                v[0] = 0;
            });

        self.poly_order_minus1.iter_mut().for_each(|v| {
            v.truncate(1);
            v[0] = 0;
        });

        self.linear_interp_flag.iter_mut().for_each(|v| {
            v.truncate(1);
            v[0] = false;
        });

        self.pred_linear_interp_value_int.iter_mut().for_each(|v| {
            v.truncate(1);
            v[0] = 0;
        });

        self.pred_linear_interp_value.iter_mut().for_each(|v| {
            v.truncate(1);
            v[0] = 0;
        });

        self.poly_coef_int.iter_mut().for_each(|v| {
            v.truncate(1);
            v.iter_mut().for_each(|v2| {
                v2.truncate(2);
                v2[0] = 0;
                v2[1] = 1;
            });
        });

        self.poly_coef.iter_mut().for_each(|v| {
            v.truncate(1);
            v.iter_mut().for_each(|v2| {
                v2.truncate(2);
                v2[0] = 0;
                v2[1] = 0;
            });
        });

        self.mmr_order_minus1.iter_mut().for_each(|v| {
            v.truncate(1);
            v[0] = 0;
        });

        self.mmr_constant_int.iter_mut().for_each(|v| {
            v.truncate(1);
            v[0] = 0;
        });

        self.mmr_constant.iter_mut().for_each(|v| {
            v.truncate(1);
            v[0] = 0;
        });

        self.mmr_coef_int.iter_mut().for_each(|v| {
            v.truncate(1);
            v[0] = vec![];
        });

        self.mmr_coef.iter_mut().for_each(|v| {
            v.truncate(1);
            v[0] = vec![];
        });
    }

    pub fn p8_default() -> VdrRpuData {
        VdrRpuData {
            mapping_idc: vec![vec![0], vec![0], vec![0]],
            mapping_param_pred_flag: vec![vec![false], vec![false], vec![false]],
            num_mapping_param_predictors: vec![vec![0], vec![0], vec![0]],
            diff_pred_part_idx_mapping_minus1: vec![vec![0], vec![0], vec![0]],
            poly_order_minus1: vec![vec![0], vec![0], vec![0]],
            linear_interp_flag: vec![vec![false], vec![false], vec![false]],
            pred_linear_interp_value_int: vec![vec![0], vec![0], vec![0]],
            pred_linear_interp_value: vec![vec![0], vec![0], vec![0]],
            poly_coef_int: vec![vec![vec![0, 1]], vec![vec![0, 1]], vec![vec![0, 1]]],
            poly_coef: vec![vec![vec![0, 1]], vec![vec![0, 1]], vec![vec![0, 1]]],
            mmr_order_minus1: vec![vec![0], vec![0], vec![0]],
            mmr_constant_int: vec![vec![0], vec![0], vec![0]],
            mmr_constant: vec![vec![0], vec![0], vec![0]],
            mmr_coef_int: vec![vec![vec![]], vec![vec![]], vec![vec![]]],
            mmr_coef: vec![vec![vec![]], vec![vec![]], vec![vec![]]],
        }
    }
}

impl NlqData {
    pub fn rpu_data_nlq(reader: &mut BitVecReader, header: &mut RpuDataHeader) -> NlqData {
        let num_cmps = 3;
        let pivot_idx_count = if let Some(nlq_num_pivots_minus2) = header.nlq_num_pivots_minus2 {
            nlq_num_pivots_minus2 as usize + 1
        } else {
            panic!("Shouldn't be in NLQ if not profile 7!");
        };

        let mut data = NlqData::default();

        let coefficient_log2_denom_length = if header.coefficient_data_type == 0 {
            header.coefficient_log2_denom as usize
        } else if header.coefficient_data_type == 1 {
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
                // Dolby pls. Guessing this is what they mean by "new parameters"
                if cmp > 0
                    && data.nlq_param_pred_flag[pivot_idx][cmp]
                        != data.nlq_param_pred_flag[pivot_idx][cmp - 1]
                {
                    predictors += 1;
                    data.num_nlq_param_predictors[pivot_idx][cmp] = predictors;
                } else {
                    predictors = 0;
                }

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

        data.validate();

        data
    }

    pub fn validate(&self) {}

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

    pub fn write(&self, writer: &mut BitVecWriter, header: &RpuDataHeader) {
        let num_cmps = 3;
        let pivot_idx_count = if let Some(nlq_num_pivots_minus2) = header.nlq_num_pivots_minus2 {
            nlq_num_pivots_minus2 as usize + 1
        } else {
            panic!("Shouldn't be in NLQ if not profile 7!");
        };
        let coefficient_log2_denom_length = if header.coefficient_data_type == 0 {
            header.coefficient_log2_denom as usize
        } else if header.coefficient_data_type == 1 {
            32
        } else {
            panic!("Invalid coefficient_data_type value!");
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
    }
}
