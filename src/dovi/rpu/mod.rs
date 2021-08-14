pub(crate) mod rpu_data;
pub(crate) mod rpu_data_header;
mod tests;
pub(crate) mod vdr_dm_data;
pub(crate) mod vdr_rpu_data;

use bitvec::{
    order::Msb0,
    prelude::{self, BitVec},
};
pub(crate) use rpu_data::DoviRpu;
use rpu_data_header::RpuDataHeader;

use super::{BitVecReader, BitVecWriter};
use hevc_parser::utils::{
    add_start_code_emulation_prevention_3_byte, clear_start_code_emulation_prevention_3_byte,
};

use serde::{ser::Serializer, Serialize};

const ST2084_Y_MAX: f64 = 10000.0;
const ST2084_M1: f64 = 2610.0 / 16384.0;
const ST2084_M2: f64 = (2523.0 / 4096.0) * 128.0;
const ST2084_C1: f64 = 3424.0 / 4096.0;
const ST2084_C2: f64 = (2413.0 / 4096.0) * 32.0;
const ST2084_C3: f64 = (2392.0 / 4096.0) * 32.0;

#[inline(always)]
pub fn parse_dovi_rpu(data: &[u8]) -> Result<DoviRpu, String> {
    if data.len() < 25 {
        return Err(format!("Invalid RPU\n{:?}", &data));
    }

    // Including 0x7C01 prepended
    let trimmed_data = match &data[..5] {
        [0, 0, 0, 1, 25] => &data[4..],
        [0, 0, 1, 25, 8] => &data[3..],
        [0, 1, 25, 8, 9] | [124, 1, 25, 8, 9] => &data[2..],
        [1, 25, 8, 9, _] => &data[1..],
        [25, 8, 9, _, _] => data,
        _ => return Err(format!("Invalid RPU data start bytes\n{:?}", &data)),
    };

    // Clear start code emulation prevention 3 byte
    let bytes: Vec<u8> = clear_start_code_emulation_prevention_3_byte(trimmed_data);

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

#[inline(always)]
pub fn nits_to_pq(nits: u16) -> f64 {
    let y = nits as f64 / ST2084_Y_MAX;

    ((ST2084_C1 + ST2084_C2 * y.powf(ST2084_M1)) / (1.0 + ST2084_C3 * y.powf(ST2084_M1)))
        .powf(ST2084_M2)
}

pub fn bitvec_ser_bits<S: Serializer>(bitvec: &BitVec<Msb0, u8>, s: S) -> Result<S::Ok, S::Error> {
    let bits: Vec<u8> = bitvec.iter().map(|b| *b as u8).collect();
    bits.serialize(s)
}
