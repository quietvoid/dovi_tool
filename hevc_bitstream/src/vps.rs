use super::BitVecReader;

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

#[derive(Default, Debug)]
struct ProfileTierLevel {
    general_profile_space: u8,
    general_tier_flag: bool,
    general_profile_idc: u8,
    general_profile_compatibility_flag: Vec<bool>,
    general_progressive_source_flag: bool,
    general_interlaced_source_flag: bool,
    general_non_packed_constraint_flag: bool,
    general_frame_only_constraint_flag: bool,
    general_level_idc: u8,

    sub_layer_profile_present_flag: Vec<bool>,
    sub_layer_level_present_flag: Vec<bool>,
    sub_layer_profile_space: Vec<u8>,
    sub_layer_tier_flag: Vec<bool>,
    sub_layer_profile_idc: Vec<u8>,
    sub_layer_profile_compatibility_flag: Vec<bool>,
    sub_layer_progressive_source_flag: Vec<bool>,
    sub_layer_interlaced_source_flag: Vec<bool>,
    sub_layer_non_packed_constraint_flag: Vec<bool>,
    sub_layer_frame_only_constraint_flag: Vec<bool>,
    sub_layer_level_idc: Vec<u8>,
}

#[derive(Default)]
struct HrdParameter {
}

struct SubLayerHrdParameter {
    
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
        
                HrdParameter::parse(bs, common_inf_present, vps.vps_max_sub_layers);
            }
        }
        
        bs.skip_n(1); // vps_extension_flag

        vps
    }
}

impl ProfileTierLevel {
    fn parse(&mut self, bs: &mut BitVecReader, vps_max_sub_layers: u8) {
        self.general_profile_space = bs.get_n(2);
        self.general_tier_flag = bs.get();
        self.general_profile_idc = bs.get_n(5);

        for _ in 0..32 {
            self.general_profile_compatibility_flag.push(bs.get());
        }

        self.general_progressive_source_flag = bs.get();
        self.general_interlaced_source_flag = bs.get();
        self.general_non_packed_constraint_flag = bs.get();
        self.general_frame_only_constraint_flag = bs.get();
        bs.skip_n(32);
        bs.skip_n(12);
        self.general_level_idc = bs.get_n(8);

        let vps_max_sub_layers_minus1 = vps_max_sub_layers - 1;
        for _ in 0..vps_max_sub_layers_minus1 {
            self.sub_layer_profile_present_flag.push(bs.get());
            self.sub_layer_level_present_flag.push(bs.get());
        }

        if vps_max_sub_layers_minus1 > 0 {
            for _ in vps_max_sub_layers_minus1 .. 8 {
                bs.skip_n(2);
            }
        }

        for i in 0..vps_max_sub_layers_minus1 as usize{
            self.sub_layer_profile_space.push(bs.get_n(2));
            self.sub_layer_tier_flag.push(bs.get());
            self.sub_layer_profile_idc.push(bs.get_n(5));

            for _ in 0..32 {
                self.sub_layer_profile_compatibility_flag.push(bs.get());
            }

            self.sub_layer_progressive_source_flag.push(bs.get());
            self.sub_layer_interlaced_source_flag.push(bs.get());
            self.sub_layer_non_packed_constraint_flag.push(bs.get());
            self.sub_layer_frame_only_constraint_flag.push(bs.get());

            bs.skip_n(32);
            bs.skip_n(12);

            if self.sub_layer_level_present_flag[i] {
                self.sub_layer_level_idc.push(bs.get_n(8));
            } else {
                self.sub_layer_level_idc.push(1);
            }
        }
    }
}

impl HrdParameter {
    pub fn parse(bs: &mut BitVecReader, common_inf_present: bool, vps_max_sub_layers: u8,) {
        let mut nal_params_present = false;
        let mut vcl_params_present = false;
        let mut subpic_params_present = false;

        if common_inf_present {
            nal_params_present = bs.get();
            vcl_params_present = bs.get();

            if nal_params_present || vcl_params_present {
                subpic_params_present = bs.get();

                if subpic_params_present {
                    bs.skip_n(8); // tick_divisor_minus2
                    bs.skip_n(5); // du_cpb_removal_delay_increment_length_minus1
                    bs.skip_n(1); // sub_pic_cpb_params_in_pic_timing_sei_flag
                    bs.skip_n(5); // dpb_output_delay_du_length_minus1
                }

                bs.skip_n(4); // bit_rate_scale
                bs.skip_n(4); // cpb_size_scale

                if subpic_params_present {
                    bs.skip_n(4); // cpb_size_du_scale
                }

                bs.skip_n(5); // initial_cpb_removal_delay_length_minus1
                bs.skip_n(5); // au_cpb_removal_delay_length_minus1
                bs.skip_n(5); // dpb_output_delay_length_minus1
            }
        }

        for _ in 0..vps_max_sub_layers {
            let mut low_delay = false;
            let mut nb_cpb = 1;
            let mut fixed_rate = bs.get();

            if !fixed_rate {
                fixed_rate = bs.get();
            }

            if fixed_rate {
                bs.get_ue();
            } else {
                low_delay = bs.get();
            }

            if !low_delay {
                nb_cpb = bs.get_ue() + 1;
            }

            if nal_params_present {
                SubLayerHrdParameter::parse(bs, nb_cpb, subpic_params_present);
            }

            if vcl_params_present {
                SubLayerHrdParameter::parse(bs, nb_cpb, subpic_params_present);
            }
        }
    }
}

impl SubLayerHrdParameter {
    pub fn parse(bs: &mut BitVecReader, nb_cpb: u64, subpic_params_present: bool) {
        for _ in 0..nb_cpb {
            bs.get_ue(); // bit_rate_value_minus1
            bs.get_ue(); // cpb_size_value_minus1

            if subpic_params_present {
                bs.get_ue(); // cpb_size_du_value_minus1
                bs.get_ue(); // bit_rate_du_value_minus1
            }

            bs.skip_n(1); // cbr_flag
        }
    }
}