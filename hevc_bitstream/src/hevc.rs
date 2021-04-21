// https://github.com/virinext/hevcesbrowser/blob/master/hevcparser/include/Hevc.h
pub const NAL_TRAIL_N: u8    = 0;
pub const NAL_TRAIL_R: u8    = 1;
pub const NAL_TSA_N: u8      = 2;
pub const NAL_TSA_R: u8      = 3;
pub const NAL_STSA_N: u8     = 4;
pub const NAL_STSA_R: u8     = 5;
pub const NAL_RADL_N: u8     = 6;
pub const NAL_RADL_R: u8     = 7;
pub const NAL_RASL_N: u8     = 8;
pub const NAL_RASL_R: u8     = 9;
pub const NAL_BLA_W_LP: u8   = 16;
pub const NAL_BLA_W_RADL: u8 = 17;
pub const NAL_BLA_N_LP: u8   = 18;
pub const NAL_IDR_W_RADL: u8= 19;
pub const NAL_IDR_N_LP: u8   = 20;
pub const NAL_CRA_NUT: u8    = 21;
pub const NAL_IRAP_VCL23: u8 = 23;
pub const NAL_VPS: u8        = 32;
pub const NAL_SPS: u8        = 33;
pub const NAL_PPS: u8        = 34;
pub const NAL_AUD: u8        = 35;
pub const NAL_EOS_NUT: u8    = 36;
pub const NAL_EOB_NUT: u8    = 37;
pub const NAL_FD_NUT: u8     = 38;
pub const NAL_SEI_PREFIX: u8 = 39;
pub const NAL_SEI_SUFFIX: u8 = 40;
pub const NAL_UNSPEC62: u8   = 62;
pub const NAL_UNSPEC63: u8   = 63;

#[derive(Default, Clone)]
pub struct NalUnit {
    pub start: usize,
    pub end: usize,

    pub nal_type: u8,
    pub nuh_layer_id: u8,
    pub temporal_id: u8,
}

#[derive(Default)]
pub struct SPSNal {
    pub nal_index: usize,
}

#[derive(Default)]
pub struct PPSNal {
    pub nal_index: usize,
}

pub fn clear_start_code_emulation_prevention_3_byte(data: &[u8]) -> Vec<u8> {
    data.iter()
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
        .collect::<Vec<u8>>()
}

pub fn add_start_code_emulation_prevention_3_byte(data: &mut Vec<u8>) {
    let mut count = data.len();
    let mut i = 0;

    while i < count {
        if i > 2 && i < count - 2 && data[i - 2] == 0 && data[i - 1] == 0 && data[i] <= 3 {
            data.insert(i, 3);
            count += 1;
        }

        i += 1;
    }
}