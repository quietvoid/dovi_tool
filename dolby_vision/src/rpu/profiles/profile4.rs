use super::{DoviProfile, VdrDmData};

pub struct Profile4 {}

impl DoviProfile for Profile4 {
    fn dm_data() -> VdrDmData {
        VdrDmData {
            ycc_to_rgb_coef0: 9575,
            ycc_to_rgb_coef1: 0,
            ycc_to_rgb_coef2: 14742,
            ycc_to_rgb_coef3: 9575,
            ycc_to_rgb_coef4: -1754,
            ycc_to_rgb_coef5: -4383,
            ycc_to_rgb_coef6: 9575,
            ycc_to_rgb_coef7: 17372,
            ycc_to_rgb_coef8: 0,
            ycc_to_rgb_offset0: 67108864,
            ycc_to_rgb_offset1: 536870912,
            ycc_to_rgb_offset2: 536870912,
            rgb_to_lms_coef0: 5845,
            rgb_to_lms_coef1: 9702,
            rgb_to_lms_coef2: 837,
            rgb_to_lms_coef3: 2568,
            rgb_to_lms_coef4: 12256,
            rgb_to_lms_coef5: 1561,
            rgb_to_lms_coef6: 0,
            rgb_to_lms_coef7: 679,
            rgb_to_lms_coef8: 15705,
            signal_eotf: 39322,
            signal_eotf_param0: 15867,
            signal_eotf_param1: 228,
            signal_eotf_param2: 1383604,
            signal_bit_depth: 14,
            signal_full_range_flag: 1,
            source_diagonal: 42,
            ..Default::default()
        }
    }
}
