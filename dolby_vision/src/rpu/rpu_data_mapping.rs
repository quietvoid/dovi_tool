use anyhow::{bail, ensure, Result};
use bitvec_helpers::{bitvec_reader::BitVecReader, bitvec_writer::BitVecWriter};

#[cfg(feature = "serde_feature")]
use serde::Serialize;

use super::dovi_rpu::DoviRpu;
use super::rpu_data_header::RpuDataHeader;
use super::rpu_data_nlq::NlqData;

#[derive(Debug, Default)]
#[cfg_attr(feature = "serde_feature", derive(Serialize))]
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

impl VdrRpuData {
    pub fn parse(dovi_rpu: &mut DoviRpu) -> Result<()> {
        let reader = &mut dovi_rpu.reader;
        dovi_rpu.vdr_rpu_data = Some(VdrRpuData::rpu_data_mapping(reader, &mut dovi_rpu.header)?);

        if dovi_rpu.header.nlq_method_idc.is_some() {
            dovi_rpu.nlq_data = Some(NlqData::rpu_data_nlq(reader, &mut dovi_rpu.header)?);
        }

        Ok(())
    }

    pub fn rpu_data_mapping(
        reader: &mut BitVecReader,
        header: &mut RpuDataHeader,
    ) -> Result<VdrRpuData> {
        let num_cmps = 3;

        let mut data = VdrRpuData::default();

        let coefficient_log2_denom_length = if header.coefficient_data_type == 0 {
            header.coefficient_log2_denom as usize
        } else if header.coefficient_data_type == 1 {
            32
        } else {
            bail!("Invalid coefficient_data_type value!");
        };

        // rpu_data_mapping_param

        for cmp in 0..num_cmps {
            let pivot_idx_count = (header.num_pivots_minus_2[cmp] + 1) as usize;

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

                        ensure!(data.mmr_order_minus1[cmp][pivot_idx] <= 2);

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

        Ok(data)
    }

    pub fn write(&self, writer: &mut BitVecWriter, header: &RpuDataHeader) -> Result<()> {
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

        Ok(())
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
