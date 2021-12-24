use super::{DoviProfile, VdrDmData};

pub struct Profile81 {}

impl DoviProfile for Profile81 {
    fn dm_data() -> VdrDmData {
        VdrDmData {
            ycc_to_rgb_coef0: 9574,
            ycc_to_rgb_coef1: 0,
            ycc_to_rgb_coef2: 13802,
            ycc_to_rgb_coef3: 9574,
            ycc_to_rgb_coef4: -1540,
            ycc_to_rgb_coef5: -5348,
            ycc_to_rgb_coef6: 9574,
            ycc_to_rgb_coef7: 17610,
            ycc_to_rgb_coef8: 0,
            ycc_to_rgb_offset0: 16777216,
            ycc_to_rgb_offset1: 134217728,
            ycc_to_rgb_offset2: 134217728,
            rgb_to_lms_coef0: 7222,
            rgb_to_lms_coef1: 8771,
            rgb_to_lms_coef2: 390,
            rgb_to_lms_coef3: 2654,
            rgb_to_lms_coef4: 12430,
            rgb_to_lms_coef5: 1300,
            rgb_to_lms_coef6: 0,
            rgb_to_lms_coef7: 422,
            rgb_to_lms_coef8: 15962,
            ..VdrDmData::default_pq()
        }
    }
}
