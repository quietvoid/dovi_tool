mod tests;
mod rpu_data;
mod vdr_dm_data;
mod vdr_rpu_data;

use bitvec::prelude;
use rpu_data::RpuNal;

use super::{
    add_start_code_emulation_prevention_3_byte, clear_start_code_emulation_prevention_3_byte,
    BitVecReader, BitVecWriter,
};

#[inline(always)]
pub fn parse_dovi_rpu(data: &[u8], mode: u8) -> Vec<u8> {
    // Clear start code emulation prevention 3 byte
    let bytes: Vec<u8> = clear_start_code_emulation_prevention_3_byte(&data[2..]);

    let mut reader = BitVecReader::new(bytes);
    let mut rpu_nal = RpuNal::read_rpu_data(&mut reader);

    match mode {
        1 => rpu_nal.convert_to_mel(),
        2 => rpu_nal.convert_to_81(),
        _ => (),
    }

    // Doesn't work for now..
    //rpu_nal.validate_crc32(&mut reader);

    let mut writer = BitVecWriter::new();

    // Write RPU data to writer
    RpuNal::write_rpu_data(rpu_nal, &mut writer);

    // Write whatever is left
    let rest = &reader.get_inner()[reader.pos()..];
    let inner_w = writer.inner_mut();
    inner_w.extend_from_bitslice(&rest);

    // Back to a u8 slice
    let mut data_to_write = inner_w.as_slice().to_vec();
    add_start_code_emulation_prevention_3_byte(&mut data_to_write);

    data_to_write
}
