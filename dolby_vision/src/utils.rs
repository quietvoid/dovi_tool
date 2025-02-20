#[cfg(feature = "serde")]
use {
    bitvec::prelude::*,
    serde::{Serialize, ser::Serializer},
};

pub const ST2084_Y_MAX: f64 = 10000.0;
pub const ST2084_M1: f64 = 2610.0 / 16384.0;
pub const ST2084_M2: f64 = (2523.0 / 4096.0) * 128.0;
pub const ST2084_C1: f64 = 3424.0 / 4096.0;
pub const ST2084_C2: f64 = (2413.0 / 4096.0) * 32.0;
pub const ST2084_C3: f64 = (2392.0 / 4096.0) * 32.0;

#[inline(always)]
pub fn pq_to_nits(x: f64) -> f64 {
    if x > 0.0 {
        let xpow = x.powf(1.0 / ST2084_M2);
        let num = (xpow - ST2084_C1).max(0.0);
        let den = (ST2084_C2 - ST2084_C3 * xpow).max(f64::NEG_INFINITY);

        (num / den).powf(1.0 / ST2084_M1) * ST2084_Y_MAX
    } else {
        0.0
    }
}

/// Helper function to calculate PQ codes from nits (cd/m2) values
#[inline(always)]
pub fn nits_to_pq(nits: f64) -> f64 {
    let y = nits / ST2084_Y_MAX;

    ((ST2084_C1 + ST2084_C2 * y.powf(ST2084_M1)) / (1.0 + ST2084_C3 * y.powf(ST2084_M1)))
        .powf(ST2084_M2)
}

/// Copied from hevc_parser for convenience, and to avoid a dependency
/// Unescapes a byte slice from annexb.
/// Allocates a new Vec.
pub fn clear_start_code_emulation_prevention_3_byte(data: &[u8]) -> Vec<u8> {
    let len = data.len();

    if len > 2 {
        let mut unescaped_bytes: Vec<u8> = Vec::with_capacity(len);
        unescaped_bytes.push(data[0]);
        unescaped_bytes.push(data[1]);

        for i in 2..len {
            if !(data[i - 2] == 0 && data[i - 1] == 0 && data[i] == 3) {
                unescaped_bytes.push(data[i]);
            }
        }

        unescaped_bytes
    } else {
        data.to_owned()
    }
}

/// Escapes the vec to annexb to avoid emulating a start code by accident
pub fn add_start_code_emulation_prevention_3_byte(data: &mut Vec<u8>) {
    let mut count = data.len();
    let mut i = 0;

    while i < count {
        if i > 2 && data[i - 2] == 0 && data[i - 1] == 0 && data[i] <= 3 {
            data.insert(i, 3);
            count += 1;
        }

        i += 1;
    }
}

/// Serializing a bitvec as a vec of bits
#[cfg(feature = "serde")]
pub(crate) fn bitvec_ser_bits<S: Serializer>(
    bitvec: &BitVec<u8, Msb0>,
    s: S,
) -> Result<S::Ok, S::Error> {
    let bits: Vec<u8> = bitvec.iter().map(|b| *b as u8).collect();
    bits.serialize(s)
}

/// Serializing an optional bitvec as a vec of bits
#[cfg(feature = "serde")]
pub(crate) fn opt_bitvec_ser_bits<S: Serializer>(
    bitvec: &Option<BitVec<u8, Msb0>>,
    s: S,
) -> Result<S::Ok, S::Error> {
    let bits: Vec<u8> = if let Some(vec) = bitvec {
        vec.iter().map(|b| *b as u8).collect()
    } else {
        Vec::new()
    };
    bits.serialize(s)
}
