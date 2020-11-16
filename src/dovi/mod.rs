pub mod demuxer;
mod rpu;

use super::bitvec_reader::BitVecReader;
use super::bitvec_writer::BitVecWriter;

#[derive(Debug, PartialEq)]
pub enum Format {
    Raw,
    RawStdin,
    Matroska,
}

pub fn clear_start_code_emulation_prevention_3_byte(data: &[u8]) -> Vec<u8> {
    data
        .iter()
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

pub fn add_start_code_emulation_prevention_3_byte(mut data: &mut Vec<u8>) {
    for i in 0 .. data.len() {
        if i > 2
            && i < data.len() - 2
            && data[i - 2] == 0 
            && data[i - 1] == 0 
            && data[i] <= 3
        {
            data.insert(i, 3);
        }
    }
}