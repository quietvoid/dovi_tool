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
    num_x_partitions_minus1: u64,
    num_y_partitions_minus1: u64,
}

pub fn rpu_data(reader: &mut BitVecReader) -> RpuNal {
    let mut rpu_nal = RpuNal::default();

    rpu_data_header(reader, &mut rpu_nal);

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

                    let mut vec = Vec::new();
                    for _ in 0..rpu_nal.num_pivots_minus_2[cmp] + 2 {
                        vec.push(reader.get_n((rpu_nal.bl_bit_depth_minus8 + 8) as usize));
                    }

                    rpu_nal.pred_pivot_value.push(vec);
                }

                if rpu_nal.rpu_format & 0x700 == 0 {
                    rpu_nal.nlq_method_idc = reader.get_n(3);
                    rpu_nal.num_x_partitions_minus1 = reader.get_ue();
                    rpu_nal.num_y_partitions_minus1 = reader.get_ue();
                }
            }
        }
    }

    assert_eq!(rpu_nal.bl_bit_depth_minus8, 2);
    assert_eq!(rpu_nal.el_bit_depth_minus8, 2);
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
}
