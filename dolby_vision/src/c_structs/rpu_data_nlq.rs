use crate::rpu::{rpu_data_nlq::RpuDataNlq as RuRpuDataNlq, NUM_COMPONENTS};

/// C struct for rpu_data_nlq()
#[repr(C)]
pub struct RpuDataNlq {
    nlq_offset: [u16; NUM_COMPONENTS],
    vdr_in_max_int: [u64; NUM_COMPONENTS],
    vdr_in_max: [u64; NUM_COMPONENTS],
    linear_deadzone_slope_int: [u64; NUM_COMPONENTS],
    linear_deadzone_slope: [u64; NUM_COMPONENTS],
    linear_deadzone_threshold_int: [u64; NUM_COMPONENTS],
    linear_deadzone_threshold: [u64; NUM_COMPONENTS],
}

impl From<&RuRpuDataNlq> for RpuDataNlq {
    fn from(data: &RuRpuDataNlq) -> Self {
        Self {
            nlq_offset: data.nlq_offset,
            vdr_in_max_int: data.vdr_in_max_int,
            vdr_in_max: data.vdr_in_max,
            linear_deadzone_slope_int: data.linear_deadzone_slope_int,
            linear_deadzone_slope: data.linear_deadzone_slope,
            linear_deadzone_threshold_int: data.linear_deadzone_threshold_int,
            linear_deadzone_threshold: data.linear_deadzone_threshold,
        }
    }
}
