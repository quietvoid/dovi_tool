use super::BitVecReader;

#[derive(Default)]
pub struct HrdParameters {
}

pub struct SubLayerHrdParameter {
    
}

impl HrdParameters {
    pub fn parse(bs: &mut BitVecReader, common_inf_present: bool, vps_max_sub_layers: u8) {
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