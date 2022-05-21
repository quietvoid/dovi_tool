use crate::rpu::{rpu_data_header::RpuDataHeader, rpu_data_mapping::RpuDataMapping};

use super::{profile81::Profile81, DoviProfile, VdrDmData};

pub struct Profile84 {}

impl DoviProfile for Profile84 {
    fn dm_data() -> VdrDmData {
        VdrDmData {
            source_min_pq: 62,
            source_max_pq: 3079,
            ..Profile81::dm_data()
        }
    }
}

// Based on iPhone 13 polynomials and MMR
impl Profile84 {
    pub fn rpu_data_header() -> RpuDataHeader {
        RpuDataHeader {
            num_pivots_minus_2: [7, 0, 0],
            pred_pivot_value: [
                vec![63, 69, 230, 256, 256, 37, 16, 8, 7],
                vec![0, 1023],
                vec![0, 1023],
            ],
            ..RpuDataHeader::p8_default()
        }
    }

    pub fn rpu_data_mapping() -> RpuDataMapping {
        let poly_coef_int_cmp0 = vec![
            vec![-1, 1, -3],
            vec![-1, 1, -2],
            vec![0, 0, -1],
            vec![0, 0, 0],
            vec![0, -2, 1],
            vec![6, -14, 8],
            vec![13, -30, 16],
            vec![28, -62, 34],
        ];

        let poly_coef_cmp0 = vec![
            vec![7978928, 8332855, 4889184],
            vec![8269552, 5186604, 3909327],
            vec![1317527, 5338528, 7440486],
            vec![2119979, 2065496, 2288524],
            vec![7982780, 5409990, 1585336],
            vec![3460436, 3197328, 615464],
            vec![3921968, 6820672, 5546752],
            vec![1947392, 1244640, 6094272],
        ];

        let mmr_coef_int_cmp1 = vec![vec![
            vec![0; 7],
            vec![-1, -2, -5, 2, 5, 9, -12],
            vec![-1, -1, 3, -1, -5, -12, 18],
            vec![-1, 0, -2, 0, 2, 7, -19],
        ]];

        let mmr_coef_int_cmp2 = vec![vec![
            vec![0; 7],
            vec![4, 0, 5, -2, -8, -1, 1],
            vec![-4, -1, -6, 1, 12, 0, -4],
            vec![1, 0, 2, -1, -8, -1, 4],
        ]];

        let mmr_coef_cmp1 = vec![vec![
            vec![0; 7],
            vec![87355, 6228986, 642500, 1023296, 6569512, 5128216, 4317296],
            vec![
                8299905, 5819931, 2324124, 7273546, 1562484, 3679480, 6357360,
            ],
            vec![8172981, 3261951, 5970055, 927142, 3525840, 5110348, 6236848],
        ]];

        let mmr_coef_cmp2 = vec![vec![
            vec![0; 7],
            vec![193104, 5369128, 2553116, 8009648, 2772020, 3122453, 2961581],
            vec![6769788, 2565605, 7864496, 4777288, 649616, 7036536, 1666406],
            vec![406265, 2901521, 2680224, 146340, 1008052, 4366810, 5080852],
        ]];

        RpuDataMapping {
            mapping_idc: [vec![0; 8], vec![1], vec![1]],
            mapping_param_pred_flag: [vec![false; 8], vec![false], vec![false]],
            num_mapping_param_predictors: [vec![0; 8], vec![0], vec![0]],
            diff_pred_part_idx_mapping_minus1: [vec![], vec![], vec![]],
            poly_order_minus1: [vec![1; 8], vec![], vec![]],
            linear_interp_flag: [vec![], vec![], vec![]],
            pred_linear_interp_value_int: [vec![], vec![], vec![]],
            pred_linear_interp_value: [vec![], vec![], vec![]],
            poly_coef_int: [poly_coef_int_cmp0, vec![], vec![]],
            poly_coef: [poly_coef_cmp0, vec![], vec![]],
            mmr_order_minus1: [vec![0], vec![2], vec![2]],
            mmr_constant_int: [vec![0], vec![1], vec![-2]],
            mmr_constant: [vec![0], vec![1150183], vec![6266112]],
            mmr_coef_int: [vec![vec![]], mmr_coef_int_cmp1, mmr_coef_int_cmp2],
            mmr_coef: [vec![vec![]], mmr_coef_cmp1, mmr_coef_cmp2],
        }
    }
}
