pub mod bitvec_reader;
pub mod bitvec_writer;

pub(crate) fn signed_to_unsigned(v: i64) -> u64 {
    let u = if v > 0 {
        (v * 2) - 1
    } else {
        -2 * v
    };

    u as u64
}
