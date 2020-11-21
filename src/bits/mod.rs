pub mod bitvec_reader;
pub mod bitvec_writer;

pub(crate) fn signed_to_unsigned(v: i64) -> u64 {
    let u = if v.is_positive() { v } else { -2 * v };

    u as u64
}
