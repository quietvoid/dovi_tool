use bitvec::prelude::*;
use super::signed_to_unsigned;

#[derive(Debug)]
pub struct BitVecWriter {
    bs: BitVec<Msb0, u8>,
    offset: usize,
}

impl BitVecWriter {
    pub fn new() -> Self {
        Self {
            bs: BitVec::new(),
            offset: 0,
        }
    }

    #[inline(always)]
    pub fn write(&mut self, v: bool) {
        self.bs.push(v);
        self.offset += 1;
    }

    #[inline(always)]
    pub fn write_n(&mut self, v: &[u8], n: usize) {
        let slice = v.view_bits();

        self.bs.extend_from_bitslice(&slice[slice.len() - n..]);

        self.offset += n;
    }

    #[inline(always)]
    pub fn write_signed_n(&mut self, v: i64, n: usize) {
        let v = signed_to_unsigned(v).to_be_bytes();
        let slice = v.view_bits();

        self.bs.extend_from_bitslice(&slice[slice.len() - n..]);

        self.offset += n;
    }

    #[inline(always)]
    pub fn write_ue(&mut self, v: u64) {
        if v == 0 {
            self.bs.push(true);
            self.offset += 1;
        } else {
            let mut vec: BitVec<Msb0, u8> = BitVec::new();
            let mut tmp = v + 1;
            let mut leading_zeroes: i64 = -1;

            while tmp > 0 {
                tmp >>= 1;
                leading_zeroes += 1;
            }

            let remaining = (v + 1 - (1 << leading_zeroes)).to_be_bytes();

            for _ in 0..leading_zeroes {
                vec.push(false);
            }

            vec.push(true);

            self.bs.extend_from_bitslice(&vec);
            self.offset += vec.len();

            self.write_n(&remaining, leading_zeroes as usize);
        }
    }

    #[inline(always)]
    pub fn write_se(&mut self, v: i64) {
        self.write_ue(signed_to_unsigned(v) as u64);
    }

    pub fn is_aligned(&self) -> bool {
        self.offset % 8 == 0
    }

    pub fn len(&self) -> usize {
        self.bs.len()
    }

    pub fn remaining(&self) -> usize {
        self.bs.len() - self.offset
    }

    pub fn pos(&self) -> usize {
        self.offset
    }

    pub fn as_slice(&self) -> &[u8] {
        self.bs.as_slice()
    }

    pub fn inner_mut(&mut self) -> &mut BitVec<Msb0, u8> {
        &mut self.bs
    }
}
