use crate::rpu::{
    rpu_data_mapping::{
        DoviMappingMethod, DoviPolynomialCurve, DoviReshapingCurve, RpuDataMapping,
    },
    NUM_COMPONENTS,
};

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

impl Profile81 {
    pub fn rpu_data_mapping() -> RpuDataMapping {
        let curves: [DoviReshapingCurve; NUM_COMPONENTS] = [
            Self::dovi_reshaping_curve(),
            Self::dovi_reshaping_curve(),
            Self::dovi_reshaping_curve(),
        ];

        RpuDataMapping {
            vdr_rpu_id: 0,
            mapping_color_space: 0,
            mapping_chroma_format_idc: 0,
            nlq_method_idc: None,
            nlq_num_pivots_minus2: None,
            nlq_pred_pivot_value: None,
            num_x_partitions_minus1: 0,
            num_y_partitions_minus1: 0,
            curves,
            nlq: None,
        }
    }

    pub fn dovi_reshaping_curve() -> DoviReshapingCurve {
        DoviReshapingCurve {
            num_pivots_minus2: 0,
            pivots: vec![0, 1023],
            mapping_idc: DoviMappingMethod::Polynomial,
            polynomial: Some(DoviPolynomialCurve::p81_default()),
            mmr: None,
        }
    }
}
