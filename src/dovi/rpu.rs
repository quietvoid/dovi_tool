use bitvec_reader::BitVecReader;

#[derive(Default, Debug)]
pub struct RpuNal {
    rpu_nal_prefix: u8,
    rpu_type: u8,
    rpu_format: u16,
    vdr_rpu_profile: u8,
    vdr_rpu_level: u8,
    vdr_seq_info_present_flag: bool,
    chroma_resampling_explicit_filter_flag: bool,
    coefficient_data_type: u8,
    coefficient_log2_denom: u64,
    vdr_rpu_normalized_idc: u8,
    bl_video_full_range_flag: bool,
    bl_bit_depth_minus8: u64,
    el_bit_depth_minus8: u64,
    vdr_bit_depth_minus_8: u64,
    spatial_resampling_filter_flag: bool,
    reserved_zero_3bits: u8,
    el_spatial_resampling_filter_flag: bool,
    disable_residual_flag: bool,
    vdr_dm_metadata_present_flag: bool,
    use_prev_vdr_rpu_flag: bool,
    prev_vdr_rpu_id: u64,
    vdr_rpu_id: u64,
    mapping_color_space: u64,
    mapping_chroma_format_idc: u64,
    num_pivots_minus_2: [u64; 3],
    pred_pivot_value: Vec<Vec<u64>>,
    nlq_method_idc: u8,
    nlq_num_pivots_minus2: u8,
    num_x_partitions_minus1: u64,
    num_y_partitions_minus1: u64,
    vdr_rpu_data: Option<VdrRpuData>,
    nlq_data: Option<NlqData>,
    vdr_dm_data: Option<VdrDmData>,
}

#[derive(Debug, Default)]
pub struct VdrRpuData {
    mapping_idc: Vec<Vec<u64>>,
    mapping_param_pred_flag: Vec<Vec<bool>>,
    num_mapping_param_predictors: Vec<Vec<u64>>,
    diff_pred_part_idx_mapping_minus1: Vec<Vec<u64>>,
    poly_order_minus1: Vec<Vec<u64>>,
    linear_interp_flag: Vec<Vec<bool>>,
    pred_linear_interp_value_int: Vec<Vec<u64>>,
    pred_linear_interp_value: Vec<Vec<u64>>,
    poly_coef_int: Vec<Vec<i64>>,
    poly_coef: Vec<Vec<u64>>,
    mmr_order_minus1: Vec<Vec<u8>>,
    mmr_constant_int: Vec<Vec<i64>>,
    mmr_constant: Vec<Vec<u64>>,
    mmr_coef_int: Vec<Vec<Vec<Vec<i64>>>>,
    mmr_coef: Vec<Vec<Vec<Vec<u64>>>>,
}

#[derive(Debug, Default)]
pub struct NlqData {
    num_nlq_param_predictors: Vec<Vec<u64>>,
    nlq_param_pred_flag: Vec<Vec<bool>>,
    diff_pred_part_idx_nlq_minus1: Vec<Vec<u64>>,
    nlq_offset: Vec<Vec<u64>>,
    vdr_in_max_int: Vec<Vec<u64>>,
    vdr_in_max: Vec<Vec<u64>>,
    linear_deadzone_slope_int: Vec<Vec<u64>>,
    linear_deadzone_slope: Vec<Vec<u64>>,
    linear_deadzone_threshold_int: Vec<Vec<u64>>,
    linear_deadzone_threshold: Vec<Vec<u64>>,
}

#[derive(Debug, Default)]
pub struct VdrDmData {
    affected_dm_metadata_id: u64,
    current_dm_metadata_id: u64,
    scene_refresh_flag: u64,
    ycc_to_rgb_coef0: i16,
    ycc_to_rgb_coef1: i16,
    ycc_to_rgb_coef2: i16,
    ycc_to_rgb_coef3: i16,
    ycc_to_rgb_coef4: i16,
    ycc_to_rgb_coef5: i16,
    ycc_to_rgb_coef6: i16,
    ycc_to_rgb_coef7: i16,
    ycc_to_rgb_coef8: i16,
    ycc_to_rgb_offset0: u32,
    ycc_to_rgb_offset1: u32,
    ycc_to_rgb_offset2: u32,
    rgb_to_lms_coef0: i16,
    rgb_to_lms_coef1: i16,
    rgb_to_lms_coef2: i16,
    rgb_to_lms_coef3: i16,
    rgb_to_lms_coef4: i16,
    rgb_to_lms_coef5: i16,
    rgb_to_lms_coef6: i16,
    rgb_to_lms_coef7: i16,
    rgb_to_lms_coef8: i16,
    signal_eotf: u16,
    signal_eotf_param0: u16,
    signal_eotf_param1: u16,
    signal_eotf_param2: u32,
    signal_bit_depth: u8,
    signal_color_space: u8,
    signal_chroma_format: u8,
    signal_full_range_flag: u8,
    source_min_pq: u16,
    source_max_pq: u16,
    source_diagonal: u16,
    num_ext_blocks: u64,
}

pub fn parse_dovi_rpu(data: &[u8]) {
    // Clear start code emulation prevention 3 byte
    let bytes: Vec<u8> = data
        .iter()
        .enumerate()
        .filter_map(|(index, value)| {
            if index > 2
                && index < data.len() - 2
                && data[index - 2] == 0
                && data[index - 1] == 0
                && data[index] <= 3
            {
                None
            } else {
                Some(*value)
            }
        })
        .collect::<Vec<u8>>();

    let mut reader = BitVecReader::new(bytes);
    let rpu_nal = rpu_data(&mut reader);

    println!("{:#?}", rpu_nal);
    println!("{} {} {}", reader.pos(), reader.len(), reader.remaining());
}


pub fn rpu_data(reader: &mut BitVecReader) -> RpuNal {
    let mut rpu_nal = RpuNal::default();

    rpu_data_header(reader, &mut rpu_nal);

    if rpu_nal.rpu_type == 2 {
        if !rpu_nal.use_prev_vdr_rpu_flag {
            vdr_rpu_data_payload(reader, &mut rpu_nal);
        }

        if rpu_nal.vdr_dm_metadata_present_flag {
            rpu_nal.vdr_dm_data = Some(vdr_dm_data_payload(reader));
        }
    }

    rpu_nal
}

pub fn rpu_data_header(reader: &mut BitVecReader, mut rpu_nal: &mut RpuNal) {
    rpu_nal.rpu_nal_prefix = reader.get_n(8);

    if rpu_nal.rpu_nal_prefix == 25 {
        rpu_nal.rpu_type = reader.get_n(6);
        rpu_nal.rpu_format = reader.get_n(11);

        if rpu_nal.rpu_type == 2 {
            rpu_nal.vdr_rpu_profile = reader.get_n(4);
            rpu_nal.vdr_rpu_level = reader.get_n(4);
            rpu_nal.vdr_seq_info_present_flag = reader.get();

            if rpu_nal.vdr_seq_info_present_flag {
                rpu_nal.chroma_resampling_explicit_filter_flag = reader.get();
                rpu_nal.coefficient_data_type = reader.get_n(2);

                if rpu_nal.coefficient_data_type == 0 {
                    rpu_nal.coefficient_log2_denom = reader.get_ue();
                }

                rpu_nal.vdr_rpu_normalized_idc = reader.get_n(2);
                rpu_nal.bl_video_full_range_flag = reader.get();

                if rpu_nal.rpu_format & 0x700 == 0 {
                    rpu_nal.bl_bit_depth_minus8 = reader.get_ue();
                    rpu_nal.el_bit_depth_minus8 = reader.get_ue();
                    rpu_nal.vdr_bit_depth_minus_8 = reader.get_ue();
                    rpu_nal.spatial_resampling_filter_flag = reader.get();
                    rpu_nal.reserved_zero_3bits = reader.get_n(3);
                    rpu_nal.el_spatial_resampling_filter_flag = reader.get();
                    rpu_nal.disable_residual_flag = reader.get();
                }
            }

            rpu_nal.vdr_dm_metadata_present_flag = reader.get();
            rpu_nal.use_prev_vdr_rpu_flag = reader.get();

            if rpu_nal.use_prev_vdr_rpu_flag {
                rpu_nal.prev_vdr_rpu_id = reader.get_ue();
            } else {
                rpu_nal.vdr_rpu_id = reader.get_ue();
                rpu_nal.mapping_color_space = reader.get_ue();
                rpu_nal.mapping_chroma_format_idc = reader.get_ue();

                for cmp in 0..3 {
                    rpu_nal.num_pivots_minus_2[cmp] = reader.get_ue();

                    let pivot_idx_count = (rpu_nal.num_pivots_minus_2[cmp] + 2) as usize;

                    let mut vec = vec![0; pivot_idx_count];
                    for pivot_idx in 0 .. pivot_idx_count {
                        vec[pivot_idx] = reader.get_n((rpu_nal.bl_bit_depth_minus8 + 8) as usize);
                    }

                    rpu_nal.pred_pivot_value.push(vec);
                }

                if rpu_nal.rpu_format & 0x700 == 0 {
                    rpu_nal.nlq_method_idc = reader.get_n(3);
                    rpu_nal.nlq_num_pivots_minus2 = 0;
                }

                rpu_nal.num_x_partitions_minus1 = reader.get_ue();
                rpu_nal.num_y_partitions_minus1 = reader.get_ue();
            }
        }
    }

    assert_eq!(rpu_nal.bl_bit_depth_minus8, 2);
    assert_eq!(rpu_nal.el_bit_depth_minus8, 2);
}

pub fn vdr_rpu_data_payload(reader: &mut BitVecReader, mut rpu_nal: &mut RpuNal) {
    let vdr_rpu_data = rpu_data_mapping(reader, rpu_nal);
    let nlq_data = rpu_data_nlq(reader, rpu_nal);

    rpu_nal.vdr_rpu_data = Some(vdr_rpu_data);
    rpu_nal.nlq_data = Some(nlq_data);
}

pub fn rpu_data_mapping(reader: &mut BitVecReader, rpu_nal: &mut RpuNal) -> VdrRpuData {
    let num_cmps = 3;

    let mut data = VdrRpuData::default();

    let coefficient_log2_denom_length = if rpu_nal.coefficient_data_type == 0 {
        rpu_nal.coefficient_log2_denom as usize
    } else if rpu_nal.coefficient_data_type == 1 {
        32
    } else {
        panic!("Invalid coefficient_data_type value!");
    };

    // rpu_data_mapping_param

    for cmp in 0..num_cmps {
        let pivot_idx_count = (rpu_nal.num_pivots_minus_2[cmp] + 1) as usize;
        let mut predictors = 0;

        data.mapping_idc.push(vec![0; pivot_idx_count]);
        data.num_mapping_param_predictors.push(vec![0; pivot_idx_count]);
        data.mapping_param_pred_flag.push(vec![false; pivot_idx_count]);
        data.diff_pred_part_idx_mapping_minus1.push(vec![0; pivot_idx_count]);

        // rpu_data_mapping_param()
        data.poly_order_minus1.push(vec![0; pivot_idx_count]);
        data.linear_interp_flag.push(vec![false; pivot_idx_count]);
        data.pred_linear_interp_value_int.push(vec![0; pivot_idx_count]);
        data.pred_linear_interp_value.push(vec![0; pivot_idx_count]);
        data.poly_coef_int.push(vec![0; pivot_idx_count]);
        data.poly_coef.push(vec![0; pivot_idx_count]);
        data.mmr_order_minus1.push(vec![0; pivot_idx_count]);
        data.mmr_constant_int.push(vec![0; pivot_idx_count]);
        data.mmr_constant.push(vec![0; pivot_idx_count]);

        data.mmr_coef_int.push(vec![vec![]; pivot_idx_count]);
        data.mmr_coef.push(vec![vec![]; pivot_idx_count]);

        for pivot_idx in 0 .. pivot_idx_count {
            data.mapping_idc[cmp][pivot_idx] = reader.get_ue();

            // Incremented after mapping_idc if mapping_param_pred_flag is 0
            if !data.mapping_param_pred_flag[cmp][pivot_idx] {
                data.num_mapping_param_predictors[cmp][pivot_idx] = predictors;
                predictors += 1;
            }

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
                    if data.poly_order_minus1[cmp][pivot_idx] == 0 && data.linear_interp_flag[cmp][pivot_idx] {
                        if rpu_nal.coefficient_data_type == 0 {
                            data.pred_linear_interp_value_int[cmp][pivot_idx] = reader.get_ue();
                        }

                        data.pred_linear_interp_value[cmp][pivot_idx] = reader.get_n(coefficient_log2_denom_length);

                        if pivot_idx as u64 == rpu_nal.num_pivots_minus_2[cmp] {
                            if rpu_nal.coefficient_data_type == 0 {
                                data.pred_linear_interp_value_int[cmp][pivot_idx + 1] = reader.get_ue();
                            }

                            data.pred_linear_interp_value[cmp][pivot_idx + 1] = reader.get_ue();
                        }
                    } else {
                        for i in 0 ..= data.poly_order_minus1[cmp][pivot_idx] + 1 {
                            if rpu_nal.coefficient_data_type == 0 {
                                data.poly_coef_int[cmp][pivot_idx] = reader.get_se();
                            }

                            data.poly_coef[cmp][pivot_idx] = reader.get_n(coefficient_log2_denom_length);
                        }
                    }
                } else if data.mapping_idc[cmp][pivot_idx] == 1 { // MAPPING_MMR
                    data.mmr_order_minus1[cmp][pivot_idx] = reader.get_n(2);

                    assert!(data.mmr_order_minus1[cmp][pivot_idx] <= 2);

                    data.mmr_coef[cmp][pivot_idx] = vec![vec![0; 7]; data.mmr_order_minus1[cmp][pivot_idx] as usize + 2];
                    data.mmr_coef_int[cmp][pivot_idx] = vec![vec![0; 7]; data.mmr_order_minus1[cmp][pivot_idx] as usize + 2];

                    if rpu_nal.coefficient_data_type == 0 {
                        data.mmr_constant_int[cmp][pivot_idx] = reader.get_se();
                    }

                    data.mmr_constant[cmp][pivot_idx] = reader.get_n(coefficient_log2_denom_length);

                    for i in 1 ..= data.mmr_order_minus1[cmp][pivot_idx] as usize + 1 {
                        for j in 0 .. 7 as usize {
                            if rpu_nal.coefficient_data_type == 0 {
                                data.mmr_coef_int[cmp][pivot_idx][i][j] = reader.get_se();
                            }

                            data.mmr_coef[cmp][pivot_idx][i][j] = reader.get_n(coefficient_log2_denom_length);
                        }
                    }
                }
            } else if data.num_mapping_param_predictors[cmp][pivot_idx] > 1 {
                data.diff_pred_part_idx_mapping_minus1[cmp][pivot_idx] = reader.get_ue();
            }
        }
    }

    data
}

pub fn rpu_data_nlq(reader: &mut BitVecReader, mut rpu_nal: &mut RpuNal) -> NlqData {
    let num_cmps = 3;
    let pivot_idx_count = (rpu_nal.nlq_num_pivots_minus2 + 1) as usize;

    let mut data = NlqData::default();

    let coefficient_log2_denom_length = if rpu_nal.coefficient_data_type == 0 {
        rpu_nal.coefficient_log2_denom as usize
    } else if rpu_nal.coefficient_data_type == 1 {
        32
    } else {
        panic!("Invalid coefficient_data_type value!");
    };

    for pivot_idx in 0 .. pivot_idx_count {
        data.num_nlq_param_predictors.push(vec![0; num_cmps]);
        data.nlq_param_pred_flag.push(vec![false; num_cmps]);
        data.diff_pred_part_idx_nlq_minus1.push(vec![0; num_cmps]);
    
        data.nlq_offset.push(vec![0; num_cmps]);
        data.vdr_in_max_int.push(vec![0; num_cmps]);
        data.vdr_in_max.push(vec![0; num_cmps]);
    
        data.linear_deadzone_slope_int.push(vec![0; num_cmps]);
        data.linear_deadzone_slope.push(vec![0; num_cmps]);
        data.linear_deadzone_threshold_int.push(vec![0; num_cmps]);
        data.linear_deadzone_threshold.push(vec![0; num_cmps]);
        
    
        let mut predictors = 0;
    
        for cmp in 0 .. num_cmps {
            // Incremented if nlq_param_pred_flag is 0
            if !data.nlq_param_pred_flag[pivot_idx][cmp] {
                data.num_nlq_param_predictors[pivot_idx][cmp] = predictors;
                predictors += 1;
            }

            if data.num_nlq_param_predictors[pivot_idx][cmp] > 0 {
                data.nlq_param_pred_flag[pivot_idx][cmp] = reader.get();
            } else {
                data.nlq_param_pred_flag[pivot_idx][cmp] = false;
            }
    
            if !data.nlq_param_pred_flag[pivot_idx][cmp] {
                // rpu_data_nlq_param
    
                data.nlq_offset[pivot_idx][cmp] = reader.get_n((rpu_nal.el_bit_depth_minus8 + 8) as usize);
    
                if rpu_nal.coefficient_data_type == 0 {
                    data.vdr_in_max_int[pivot_idx][cmp] = reader.get_ue();
                }
    
                data.vdr_in_max[pivot_idx][cmp] = reader.get_n(coefficient_log2_denom_length);
    
                // NLQ_LINEAR_DZ
                if rpu_nal.nlq_method_idc == 0 {
                    if rpu_nal.coefficient_data_type == 0 {
                        data.linear_deadzone_slope_int[pivot_idx][cmp] = reader.get_ue();
                    }
    
                    data.linear_deadzone_slope[pivot_idx][cmp] = reader.get_n(coefficient_log2_denom_length);
    
                    if rpu_nal.coefficient_data_type == 0 {
                        data.linear_deadzone_threshold_int[pivot_idx][cmp] = reader.get_ue();
                    }
    
                    data.linear_deadzone_threshold[pivot_idx][cmp] = reader.get_n(coefficient_log2_denom_length);
                }
            } else if data.num_nlq_param_predictors[pivot_idx][cmp] > 1 {
                data.diff_pred_part_idx_nlq_minus1[pivot_idx][cmp] = reader.get_ue();
            }
        }
    }

    data
}

pub fn vdr_dm_data_payload(reader: &mut BitVecReader) -> VdrDmData {
    let mut data = VdrDmData::default();
    data.affected_dm_metadata_id = reader.get_ue();

    assert!(data.affected_dm_metadata_id <= 15);

    data.current_dm_metadata_id = reader.get_ue();
    data.scene_refresh_flag = reader.get_ue();
    data.ycc_to_rgb_coef0 = reader.get_n::<u16>(16) as i16;
    data.ycc_to_rgb_coef1 = reader.get_n::<u16>(16) as i16;
    data.ycc_to_rgb_coef2 = reader.get_n::<u16>(16) as i16;
    data.ycc_to_rgb_coef3 = reader.get_n::<u16>(16) as i16;
    data.ycc_to_rgb_coef4 = reader.get_n::<u16>(16) as i16;
    data.ycc_to_rgb_coef5 = reader.get_n::<u16>(16) as i16;
    data.ycc_to_rgb_coef6 = reader.get_n::<u16>(16) as i16;
    data.ycc_to_rgb_coef7 = reader.get_n::<u16>(16) as i16;
    data.ycc_to_rgb_coef8 = reader.get_n::<u16>(16) as i16;
    data.ycc_to_rgb_offset0 = reader.get_n(32);
    data.ycc_to_rgb_offset1 = reader.get_n(32);
    data.ycc_to_rgb_offset2 = reader.get_n(32);
    data.rgb_to_lms_coef0 = reader.get_n::<u16>(16) as i16;
    data.rgb_to_lms_coef1 = reader.get_n::<u16>(16) as i16;
    data.rgb_to_lms_coef2 = reader.get_n::<u16>(16) as i16;
    data.rgb_to_lms_coef3 = reader.get_n::<u16>(16) as i16;
    data.rgb_to_lms_coef4 = reader.get_n::<u16>(16) as i16;
    data.rgb_to_lms_coef5 = reader.get_n::<u16>(16) as i16;
    data.rgb_to_lms_coef6 = reader.get_n::<u16>(16) as i16;
    data.rgb_to_lms_coef7 = reader.get_n::<u16>(16) as i16;
    data.rgb_to_lms_coef8 = reader.get_n::<u16>(16) as i16;
    data.signal_eotf = reader.get_n(16);
    data.signal_eotf_param0 = reader.get_n(16);
    data.signal_eotf_param1 = reader.get_n(16);
    data.signal_eotf_param2 = reader.get_n(32);
    data.signal_bit_depth = reader.get_n(5);

    assert!(data.signal_bit_depth >= 8 && data.signal_bit_depth <= 16);

    data.signal_color_space = reader.get_n(2);
    data.signal_chroma_format = reader.get_n(2);
    data.signal_full_range_flag = reader.get_n(2);
    data.source_min_pq = reader.get_n(12);
    data.source_max_pq = reader.get_n(12);
    data.source_diagonal = reader.get_n(10);
    data.num_ext_blocks = reader.get_ue();

    data
}