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
pub fn parse_dovi_rpu(data: &[u8]) -> DoviRpu {
    // Clear start code emulation prevention 3 byte
    let bytes: Vec<u8> = clear_start_code_emulation_prevention_3_byte(&data[2..]);

    let received_crc32 = DoviRpu::compute_crc32(&bytes[1..bytes.len() - 5]);

    let mut dovi_rpu = DoviRpu::read_rpu_data(bytes);

    assert_eq!(received_crc32, dovi_rpu.rpu_data_crc32);

    dovi_rpu.dovi_profile = dovi_rpu.header.get_dovi_profile();

    dovi_rpu
}
