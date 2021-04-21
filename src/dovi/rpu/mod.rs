mod rpu_data;
mod rpu_data_header;
mod tests;
mod vdr_dm_data;
mod vdr_rpu_data;

use bitvec::prelude;
use rpu_data::DoviRpu;
use rpu_data_header::RpuDataHeader;

use super::{BitVecReader, BitVecWriter};
use hevc_bitstream::utils::{
    add_start_code_emulation_prevention_3_byte, clear_start_code_emulation_prevention_3_byte,
};

#[inline(always)]
pub fn parse_dovi_rpu(data: &[u8]) -> Result<DoviRpu, String> {
    // Clear start code emulation prevention 3 byte
    let bytes: Vec<u8> = clear_start_code_emulation_prevention_3_byte(&data[2..]);
    let len = bytes.len();

    let mut received_crc32 = DoviRpu::compute_crc32(&bytes[1..len - 5]);
    let last_byte = bytes[len - 1];

    // Final RPU exception
    if last_byte == 0 && bytes[len - 2] == 0x80 {
        received_crc32 = DoviRpu::compute_crc32(&bytes[1..len - 6]);
    } else if last_byte != 0x80 {
        return Err(format!("Invalid RPU\n{:?}", &bytes));
    }

    let mut dovi_rpu = DoviRpu::read_rpu_data(bytes, last_byte);
    assert_eq!(received_crc32, dovi_rpu.rpu_data_crc32);

    dovi_rpu.dovi_profile = dovi_rpu.header.get_dovi_profile();

    Ok(dovi_rpu)
}
