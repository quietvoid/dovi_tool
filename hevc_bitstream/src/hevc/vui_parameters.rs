use super::BitVecReader;

use super::hrd_parameters::HrdParameters;

#[derive(Default, Debug)]
pub struct VuiParameters {
    sar_present: bool,
    sar_idx: u8,
    sar_num: u16,
    sar_den: u16,
    overscan_info_present_flag: bool,
    overscan_appropriate_flag: bool,
    video_signal_type_present_flag: bool,

    video_format: u8,
    video_full_range_flag: bool,
    colour_description_present_flag: bool,
    colour_primaries: u8,
    transfer_characteristic: u8,
    matrix_coeffs: u8,

    chroma_loc_info_present_flag: bool,
    chroma_sample_loc_type_top_field: u64,
    chroma_sample_loc_type_bottom_field: u64,
    neutral_chroma_indication_flag: bool,
    field_seq_flag: bool,
    frame_field_info_present_flag: bool,

    default_display_window_flag: bool,
    def_disp_win_left_offset: u64,
    def_disp_win_right_offset: u64,
    def_disp_win_top_offset: u64,
    def_disp_win_bottom_offset: u64,

    vui_timing_info_present_flag: bool,
    vui_num_units_in_tick: u32,
    vui_time_scale: u32,
    vui_poc_proportional_to_timing_flag: bool,
    vui_num_ticks_poc_diff_one_minus1: u64,
    vui_hrd_parameters_present_flag: bool,

    bitstream_restriction_flag: bool,
    tiles_fixed_structure_flag: bool,
    motion_vectors_over_pic_boundaries_flag: bool,
    restricted_ref_pic_lists_flag: bool,
    
    min_spatial_segmentation_idc: u64,
    max_bytes_per_pic_denom: u64,
    max_bits_per_min_cu_denom: u64,
    log2_max_mv_length_horizontal: u64,
    log2_max_mv_length_vertical: u64,
}

impl VuiParameters {
    pub fn parse(bs: &mut BitVecReader, max_sub_layers: u8) -> VuiParameters {
        let mut vui = VuiParameters::default();

        vui.sar_present = bs.get();
        
        if vui.sar_present {
            vui.sar_idx = bs.get_n(8);

            if vui.sar_idx == 255 {
                vui.sar_num = bs.get_n(16);
                vui.sar_den = bs.get_n(16);
            }
        }

        vui.overscan_info_present_flag = bs.get();
        if vui.overscan_info_present_flag {
            vui.overscan_appropriate_flag = bs.get();
        }

        vui.video_signal_type_present_flag = bs.get();
        if vui.video_signal_type_present_flag {
            vui.video_format = bs.get_n(3);
            vui.video_full_range_flag = bs.get();
            vui.colour_description_present_flag = bs.get();

            if vui.colour_description_present_flag {
                vui.colour_primaries = bs.get_n(8);
                vui.transfer_characteristic = bs.get_n(8);
                vui.matrix_coeffs = bs.get_n(8);
            }
        }

        vui.chroma_loc_info_present_flag = bs.get();
        if vui.chroma_loc_info_present_flag {
            vui.chroma_sample_loc_type_top_field = bs.get_ue();
            vui.chroma_sample_loc_type_bottom_field = bs.get_ue();
        }

        vui.neutral_chroma_indication_flag = bs.get();
        vui.field_seq_flag = bs.get();
        vui.frame_field_info_present_flag = bs.get();
        vui.default_display_window_flag = bs.get();

        if vui.default_display_window_flag {
            vui.def_disp_win_left_offset = bs.get_ue();
            vui.def_disp_win_right_offset = bs.get_ue();
            vui.def_disp_win_top_offset = bs.get_ue();
            vui.def_disp_win_bottom_offset = bs.get_ue();
        }

        vui.vui_timing_info_present_flag = bs.get();
        if vui.vui_timing_info_present_flag {
            vui.vui_num_units_in_tick = bs.get_n(32);
            vui.vui_time_scale = bs.get_n(32);

            vui.vui_poc_proportional_to_timing_flag = bs.get();
            if vui.vui_poc_proportional_to_timing_flag {
                vui.vui_num_ticks_poc_diff_one_minus1 = bs.get_ue();
            }

            vui.vui_hrd_parameters_present_flag = bs.get();
            if vui.vui_hrd_parameters_present_flag {
                HrdParameters::parse(bs, true, max_sub_layers);
            }
        }

        vui.bitstream_restriction_flag = bs.get();
        if vui.bitstream_restriction_flag {
            vui.tiles_fixed_structure_flag = bs.get();
            vui.motion_vectors_over_pic_boundaries_flag = bs.get();
            vui.restricted_ref_pic_lists_flag = bs.get();

            vui.min_spatial_segmentation_idc = bs.get_ue();
            vui.max_bytes_per_pic_denom = bs.get_ue();
            vui.max_bits_per_min_cu_denom = bs.get_ue();
            vui.log2_max_mv_length_horizontal = bs.get_ue();
            vui.log2_max_mv_length_vertical = bs.get_ue();
        }

        vui
    }
}