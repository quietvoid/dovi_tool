use std::cmp::min;
use super::BitVecReader;

#[derive(Default, Debug)]
pub struct ScalingListData {
    scaling_list_pred_mode_flag: Vec<Vec<bool>>,
    scaling_list_pred_matrix_id_delta: Vec<Vec<u64>>,
    scaling_list_dc_coef_minus8: Vec<Vec<i64>>,
    scaling_list_delta_coef: Vec<Vec<Vec<i64>>>,
}

impl ScalingListData {
    pub fn parse(bs: &mut BitVecReader) -> ScalingListData {
        let mut scl = ScalingListData::default();

        scl.scaling_list_pred_mode_flag.resize(4, Vec::new());
        scl.scaling_list_pred_matrix_id_delta.resize(4, Vec::new());
        scl.scaling_list_dc_coef_minus8.resize(2, Vec::new());
        scl.scaling_list_delta_coef.resize(4, Vec::new());
        
        for size_id in 0..4 {
            let matrix_size = if size_id == 3 {
                2
            } else {
                6
            };
        
            scl.scaling_list_pred_mode_flag[size_id].resize(matrix_size, false);
            scl.scaling_list_pred_matrix_id_delta[size_id].resize(matrix_size, 0);
            scl.scaling_list_delta_coef[size_id].resize(matrix_size, Vec::new());

            if size_id >= 2 {
                scl.scaling_list_dc_coef_minus8[size_id - 2].resize(matrix_size, 0);
            }
        
            for matrix_id in 0..matrix_size {
                scl.scaling_list_pred_mode_flag[size_id][matrix_id] = bs.get();
        
                if !scl.scaling_list_pred_mode_flag[size_id][matrix_id] {
                    scl.scaling_list_pred_matrix_id_delta[size_id][matrix_id] = bs.get_ue();
                } else {
                    let _next_coef = 8;
                    let coef_num = min(64, 1 << (4 + (size_id << 1)));
        
                    if size_id > 1 {
                        scl.scaling_list_dc_coef_minus8[size_id - 2][matrix_id] = bs.get_se();
                    }
        
                    scl.scaling_list_delta_coef[size_id][matrix_id].resize(coef_num, 0);
        
                    for i in 0.. coef_num {
                        scl.scaling_list_delta_coef[size_id][matrix_id][i] = bs.get_se();
                    }
                }
            }
        }

        scl
    }
}