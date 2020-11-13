use bitvec::prelude::*;

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

    pub fn get(&mut self) -> bool {
        let val = self.bs.get(self.offset).unwrap();
        self.offset += 1;

        *val
    }

    pub fn get_n <T: BitMemory> (&mut self, n: usize) -> T {
        let val = self.bs[self.offset .. self.offset + n].load_be::<T>();
        self.offset += n;

        val
    }

    pub fn get_ue(&mut self) -> u64 {
        let oldpos = self.offset;
        let mut pos = self.offset;

        // bitstring.py implementation: https://github.com/scott-griffiths/bitstring/blob/master/bitstring.py#L1706
        loop {
            match self.bs.get(pos) {
                Some(val) => if !val {
                    pos += 1;
                } else {
                    break;
                },
                None => panic!("Out of bounds index: {}", pos)
            }
        }

        let leading_zeroes = pos - oldpos;
        let mut code_num = (1 << leading_zeroes) - 1;

        if leading_zeroes > 0 {
            if pos + leading_zeroes + 1 > self.bs.len() {
                panic!("Out of bounds attempt");
            }

            code_num += self.bs[pos + 1 ..= pos + leading_zeroes].load_be::<u64>();
            pos += leading_zeroes + 1;
        } else {
            assert_eq!(code_num, 0);
            pos += 1;
        }

        self.offset = pos;

        code_num
    }
}