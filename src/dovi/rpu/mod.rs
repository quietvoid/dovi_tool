mod rpu_data;
mod rpu_data_header;
mod tests;
mod vdr_dm_data;
mod vdr_rpu_data;

use bitvec::prelude;
use rpu_data::DoviRpu;
use rpu_data_header::RpuDataHeader;

use super::{
    add_start_code_emulation_prevention_3_byte, clear_start_code_emulation_prevention_3_byte,
    BitVecReader, BitVecWriter,
};

#[inline(always)]
pub fn parse_dovi_rpu(data: &[u8], mode: u8) -> DoviRpu {
    // Clear start code emulation prevention 3 byte
    let bytes: Vec<u8> = clear_start_code_emulation_prevention_3_byte(&data[2..]);

    let mut dovi_rpu = DoviRpu::read_rpu_data(bytes);

    match mode {
        1 => dovi_rpu.convert_to_mel(),
        2 => dovi_rpu.convert_to_81(),
        _ => (),
    }

    // Doesn't work for now..
    //dovi_rpu.validate_crc32(&mut reader);

    dovi_rpu
}
