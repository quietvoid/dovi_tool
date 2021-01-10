use bitvec::mem::BitMemory;
use bitvec::prelude::*;
use std::fmt;

#[derive(Default)]
pub struct BitVecReader {
    bs: BitVec<Msb0, u8>,
    offset: usize,
}

impl BitVecReader {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            bs: BitVec::from_vec(data),
            offset: 0,
        }
    }

    #[inline(always)]
    pub fn get(&mut self) -> bool {
        let val = self.bs.get(self.offset).unwrap();
        self.offset += 1;

        *val
    }

    #[inline(always)]
    pub fn get_n<T: BitMemory>(&mut self, n: usize) -> T {
        let val = self.bs[self.offset..self.offset + n].load_be::<T>();
        self.offset += n;

        val
    }

    // bitstring.py implementation: https://github.com/scott-griffiths/bitstring/blob/master/bitstring.py#L1706
    #[inline(always)]
    pub fn get_ue(&mut self) -> u64 {
        let oldpos = self.offset;
        let mut pos = self.offset;

        loop {
            match self.bs.get(pos) {
                Some(val) => {
                    if !val {
                        pos += 1;
                    } else {
                        break;
                    }
                }
                None => panic!("Out of bounds index: {}", pos),
            }
        }

        let leading_zeroes = pos - oldpos;
        let mut code_num = (1 << leading_zeroes) - 1;

        if leading_zeroes > 0 {
            if pos + leading_zeroes + 1 > self.bs.len() {
                panic!("Out of bounds attempt");
            }

            code_num += self.bs[pos + 1..pos + leading_zeroes + 1].load_be::<u64>();
            pos += leading_zeroes + 1;
        } else {
            assert_eq!(code_num, 0);
            pos += 1;
        }

        self.offset = pos;

        code_num
    }

    // bitstring.py implementation: https://github.com/scott-griffiths/bitstring/blob/master/bitstring.py#L1767
    #[inline(always)]
    pub fn get_se(&mut self) -> i64 {
        let code_num = self.get_ue();

        let m = ((code_num + 1) as f64 / 2.0).floor() as u64;

        if code_num % 2 == 0 {
            -(m as i64)
        } else {
            m as i64
        }
    }

    pub fn is_aligned(&self) -> bool {
        self.offset % 8 == 0
    }

    pub fn available(&self) -> usize {
        self.bs.len() - self.offset
    }
}

impl fmt::Debug for BitVecReader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "BitVecReader: {{offset: {}, len: {}}}",
            self.offset,
            self.bs.len()
        )
    }
}
