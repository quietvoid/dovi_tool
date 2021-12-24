use super::{DoviProfile, VdrDmData};

pub struct Profile5 {}

impl DoviProfile for Profile5 {
    fn dm_data() -> VdrDmData {
        VdrDmData {
            ycc_to_rgb_coef0: 8192,
            ycc_to_rgb_coef1: 799,
            ycc_to_rgb_coef2: 1681,
            ycc_to_rgb_coef3: 8192,
            ycc_to_rgb_coef4: -933,
            ycc_to_rgb_coef5: 1091,
            ycc_to_rgb_coef6: 8192,
            ycc_to_rgb_coef7: 267,
            ycc_to_rgb_coef8: -5545,
            ycc_to_rgb_offset0: 0,
            ycc_to_rgb_offset1: 134217728,
            ycc_to_rgb_offset2: 134217728,
            rgb_to_lms_coef0: 17081,
            rgb_to_lms_coef1: -349,
            rgb_to_lms_coef2: -349,
            rgb_to_lms_coef3: -349,
            rgb_to_lms_coef4: 17081,
            rgb_to_lms_coef5: -349,
            rgb_to_lms_coef6: -349,
            rgb_to_lms_coef7: -349,
            rgb_to_lms_coef8: 17081,
            signal_color_space: 2,
            ..VdrDmData::default_pq()
        }
    }

    fn backwards_compatible() -> bool {
        false
    }
}
