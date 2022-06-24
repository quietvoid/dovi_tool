use crate::rpu::rpu_data_mapping::RpuDataMapping as RuRpuDataMapping;

use super::{buffers::*, components_to_cdata, NUM_COMPONENTS};

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
