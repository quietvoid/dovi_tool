use super::BitVecReader;
use super::profile_tier_level::ProfileTierLevel;
use super::scaling_list_data::ScalingListData;
use super::short_term_rps::ShortTermRPS;
use super::vui_parameters::VuiParameters;

#[derive(Default, Debug)]
pub struct SPSNal {
    pub nal_index: usize,

    vps_id: u8,
    max_sub_layers: u8,
    temporal_id_nesting_flag: bool,

    ptl: ProfileTierLevel,
    sps_id: u64,
    chroma_format_idc: u64,
    separate_colour_plane_flag: bool,
    width: u64,
    height: u64,

    pic_conformance_flag: bool,
    conf_win_left_offset: u64,
    conf_win_right_offset: u64,
    conf_win_top_offset: u64,
    conf_win_bottom_offset: u64,
    
    bit_depth: u64,
    bit_depth_chroma: u64,
    log2_max_poc_lsb: u64,
    sublayer_ordering_info: bool,
    max_dec_pic_buffering: Vec<u64>,
    num_reorder_pics: Vec<u64>,
    max_latency_increase: Vec<u64>,

    log2_min_cb_size: u64,
    log2_diff_max_min_coding_block_size: u64,
    log2_min_tb_size: u64,
    log2_diff_max_min_transform_block_size: u64,
    max_transform_hierarchy_depth_inter: u64,
    max_transform_hierarchy_depth_intra: u64,

    scaling_list_enabled_flag: bool,
    scaling_list_data_present_flag: bool,
    scaling_list_data: ScalingListData,

    amp_enabled_flag: bool,
    sao_enabled_flag: bool,
    pcm_enabled_flag: bool,
    pcm_bit_depth: u8,
    pcm_bit_depth_chroma: u8,
    pcm_log2_min_pcm_cb_size: u64,
    pcm_log2_max_pcm_cb_size: u64,
    pcm_loop_filter_disable_flag: bool,

    nb_st_rps: u64,
    pub(crate) short_term_ref_pic_sets: Vec<ShortTermRPS>,

    long_term_ref_pics_present_flag: bool,
    num_long_term_ref_pics_sps: u64,
    lt_ref_pic_poc_lsb_sps: Vec<u64>,
    used_by_curr_pic_lt_sps_flag: Vec<bool>,

    sps_temporal_mvp_enabled_flag: bool,
    sps_strong_intra_smoothing_enable_flag: bool,

    vui_present: bool,
    vui_parameters: VuiParameters,

    sps_extension_flag: bool,
}

impl SPSNal {
    pub fn parse(bs: &mut BitVecReader) -> SPSNal {
        let mut sps = SPSNal::default();

        sps.vps_id = bs.get_n(4);
        sps.max_sub_layers = bs.get_n::<u8>(3) + 1;
        sps.temporal_id_nesting_flag = bs.get();

        sps.ptl.parse(bs, sps.max_sub_layers);

        sps.sps_id = bs.get_ue();
        sps.chroma_format_idc = bs.get_ue();

        if sps.chroma_format_idc == 3 {
            sps.separate_colour_plane_flag = bs.get();
        }

        if sps.separate_colour_plane_flag {
            sps.chroma_format_idc = 0;
        }

        sps.width = bs.get_ue();
        sps.height = bs.get_ue();
        sps.pic_conformance_flag = bs.get();

        if sps.pic_conformance_flag {
            sps.conf_win_left_offset = bs.get_ue();
            sps.conf_win_right_offset = bs.get_ue();
            sps.conf_win_top_offset = bs.get_ue();
            sps.conf_win_bottom_offset = bs.get_ue();
        }

        sps.bit_depth = bs.get_ue() + 8;
        sps.bit_depth_chroma = bs.get_ue() + 8;
        sps.log2_max_poc_lsb = bs.get_ue() + 4;
        sps.sublayer_ordering_info = bs.get();

        let start = if sps.sublayer_ordering_info {
            0
        } else {
            sps.max_sub_layers - 1
        };

        for _ in start..sps.max_sub_layers {
            sps.max_dec_pic_buffering.push(bs.get_ue() + 1);
            sps.num_reorder_pics.push(bs.get_ue());

            let mut max_latency_increase = bs.get_ue();
            if max_latency_increase > 0 {
                max_latency_increase -= 1;
            }
        
            sps.max_latency_increase.push(max_latency_increase);
        }

        sps.log2_min_cb_size = bs.get_ue() + 3;
        sps.log2_diff_max_min_coding_block_size = bs.get_ue();
        sps.log2_min_tb_size = bs.get_ue() + 2;
        sps.log2_diff_max_min_transform_block_size = bs.get_ue();

        sps.max_transform_hierarchy_depth_inter = bs.get_ue();
        sps.max_transform_hierarchy_depth_intra = bs.get_ue();

        sps.scaling_list_enabled_flag = bs.get();

        if sps.scaling_list_enabled_flag {
            sps.scaling_list_data_present_flag = bs.get();

            if sps.scaling_list_data_present_flag {
                sps.scaling_list_data = ScalingListData::parse(bs);   
            }
        }

        sps.amp_enabled_flag = bs.get();
        sps.sao_enabled_flag = bs.get();
        sps.pcm_enabled_flag = bs.get();

        if sps.pcm_enabled_flag {
            sps.pcm_bit_depth = bs.get_n::<u8>(4) + 1;
            sps.pcm_bit_depth_chroma = bs.get_n::<u8>(4) + 1;
            sps.pcm_log2_min_pcm_cb_size = bs.get_ue() + 3;
            sps.pcm_log2_max_pcm_cb_size = bs.get_ue() + sps.pcm_log2_min_pcm_cb_size;

            sps.pcm_loop_filter_disable_flag = bs.get();
        }

        sps.nb_st_rps = bs.get_ue();
        for i in 0..sps.nb_st_rps {
            let rps = ShortTermRPS::parse(bs, &sps, i as usize, sps.nb_st_rps, false);
            sps.short_term_ref_pic_sets.push(rps);
        }

        sps.long_term_ref_pics_present_flag = bs.get();

        if sps.long_term_ref_pics_present_flag {
            sps.num_long_term_ref_pics_sps = bs.get_ue();

            for _ in 0..sps.num_long_term_ref_pics_sps {
                sps.lt_ref_pic_poc_lsb_sps.push(bs.get_n(sps.log2_max_poc_lsb as usize));
                sps.used_by_curr_pic_lt_sps_flag.push(bs.get());
            }
        }

        sps.sps_temporal_mvp_enabled_flag = bs.get();
        sps.sps_strong_intra_smoothing_enable_flag = bs.get();

        sps.vui_present = bs.get();

        if sps.vui_present {
            sps.vui_parameters = VuiParameters::parse(bs, sps.max_sub_layers);
        }

        sps.sps_extension_flag = bs.get();

        sps
    }
}