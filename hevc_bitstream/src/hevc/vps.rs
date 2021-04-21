use super::BitVecReader;
use super::hrd_parameters::HrdParameters;
use super::profile_tier_level::ProfileTierLevel;

#[derive(Default, Debug)]
pub struct VPSNal {
    pub nal_index: usize,

    vps_video_parameter_set_id: u8,
    vps_max_layers: u8,
    vps_max_sub_layers: u8,
    vps_temporal_id_nesting_flag: bool,
    ptl: ProfileTierLevel,
    vps_sub_layer_ordering_info_present_flag: bool,
    vps_max_dec_pic_buffering: Vec<u64>,
    vps_num_reorder_pics: Vec<u64>,
    vps_max_latency_increase: Vec<u64>,
    vps_max_layer_id: u64,
    vps_num_layer_sets: u64,
    vps_timing_info_present_flag: bool,
    vps_num_units_in_tick: u32,
    vps_time_scale: u32,
    vps_poc_proportional_to_timing_flag: bool,
    vps_num_ticks_poc_diff_one: u64,
    vps_num_hrd_parameters: u64,
}

impl VPSNal {
    pub fn parse(bs: &mut BitVecReader) -> VPSNal {
        let mut vps = VPSNal::default();

        vps.vps_video_parameter_set_id = bs.get_n(4);
        
        // vps_reserved_three_2bits
        assert!(bs.get_n::<u8>(2) == 3);
        
        vps.vps_max_layers = bs.get_n::<u8>(6) + 1;
        vps.vps_max_sub_layers = bs.get_n::<u8>(3) + 1;
        vps.vps_temporal_id_nesting_flag = bs.get();
        
        // vps_reserved_ffff_16bits
        assert!(bs.get_n::<u32>(16) == 0xFFFF);
        
        vps.ptl.parse(bs, vps.vps_max_sub_layers);
        
        vps.vps_sub_layer_ordering_info_present_flag = bs.get();
        
        let i = if vps.vps_sub_layer_ordering_info_present_flag {
            0
        } else {
            vps.vps_max_sub_layers - 1
        };
        
        for _ in i..vps.vps_max_sub_layers {
            vps.vps_max_dec_pic_buffering.push(bs.get_ue() + 1);
            vps.vps_num_reorder_pics.push(bs.get_ue());
        
            let mut vps_max_latency_increase = bs.get_ue();
            if vps_max_latency_increase > 0 {
                vps_max_latency_increase -= 1;
            }
        
            vps.vps_max_latency_increase.push(vps_max_latency_increase);
        }
        
        vps.vps_max_layer_id = bs.get_n(6);
        vps.vps_num_layer_sets = bs.get_ue() + 1;
        
        for _ in 1..vps.vps_num_layer_sets {
            for _ in 0..=vps.vps_max_layer_id {
                bs.skip_n(1); // layer_id_included_flag[i][j]
            }
        }
        
        vps.vps_timing_info_present_flag = bs.get();
        
        if vps.vps_timing_info_present_flag {
            vps.vps_num_units_in_tick = bs.get_n(32);
            vps.vps_time_scale = bs.get_n(32);
            vps.vps_poc_proportional_to_timing_flag = bs.get();
        
            if vps.vps_poc_proportional_to_timing_flag {
                vps.vps_num_ticks_poc_diff_one = bs.get_ue() + 1;
            }
            
            vps.vps_num_hrd_parameters = bs.get_ue();
        
            for i in 0..vps.vps_num_hrd_parameters {
                let mut common_inf_present = false;
                bs.get_ue(); // hrd_layer_set_idx
        
                if i > 0 {
                    common_inf_present = bs.get();
                }
        
                HrdParameters::parse(bs, common_inf_present, vps.vps_max_sub_layers);
            }
        }
        
        bs.skip_n(1); // vps_extension_flag

        vps
    }
}