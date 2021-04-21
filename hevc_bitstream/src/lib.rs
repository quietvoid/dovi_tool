use bits_handler::bitvec_reader::BitVecReader;

pub mod hevc;
pub mod vps;
use hevc::*;
use vps::VPSNal;

#[derive(Default)]
pub struct HevcBitstream {
    nals: Vec<NalUnit>,
    vps: Vec<VPSNal>,
    sps: Vec<SPSNal>,
    pps: Vec<PPSNal>,

    reader: BitVecReader,
}

// We don't want to parse large slices because the memory is copied
const MAX_PARSE_SIZE: usize = 8192;

impl HevcBitstream {
    pub fn parse_nal(&mut self, data: &[u8], offset: usize, size: usize) -> NalUnit {
        let mut nal = NalUnit::default();

        // Assuming [0, 0, 1] header
        let pos = offset + 3;
        let max_size = if size > MAX_PARSE_SIZE {
            MAX_PARSE_SIZE
        } else {
            size
        };

        nal.start = pos;
        nal.end = pos + size;

        let bytes = clear_start_code_emulation_prevention_3_byte(&data[pos..pos + max_size]);
        self.reader = BitVecReader::new(bytes);

        self.parse_nal_header(&mut nal);

        match nal.nal_type {
            NAL_VPS => self.parse_vps(),
            NAL_SPS => self.parse_sps(),
            NAL_PPS => self.parse_pps(),

            NAL_TRAIL_R | NAL_TRAIL_N | NAL_TSA_N | NAL_TSA_R |
            NAL_STSA_N | NAL_STSA_R | NAL_BLA_W_LP | NAL_BLA_W_RADL |
            NAL_BLA_N_LP | NAL_IDR_W_RADL | NAL_IDR_N_LP | NAL_CRA_NUT |
            NAL_RADL_N | NAL_RADL_R | NAL_RASL_N | NAL_RASL_R => {
                self.parse_slice();
            },
            _ => (),
        };

        self.nals.push(nal.clone());

        nal
    }

    pub fn parse_nal_header(&mut self, nal: &mut NalUnit) {
        // forbidden_zero_bit
        self.reader.get();

        nal.nal_type = self.reader.get_n(6);
        nal.nuh_layer_id = self.reader.get_n(6);
        nal.temporal_id = self.reader.get_n::<u8>(3) - 1;
    }
    
    pub fn parse_vps(&mut self) {
        let mut vps = VPSNal::parse(&mut self.reader);
        vps.nal_index = self.nals.len();

        self.vps.push(vps);
    }
    
    pub fn parse_sps(&mut self) {
        
    }

    pub fn parse_pps(&mut self) {
        
    }

    pub fn parse_slice(&mut self) {
        
    }
}