use crate::rpu::rpu_data_nlq::RpuDataNlq as RuRpuDataNlq;

use super::buffers::*;

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
