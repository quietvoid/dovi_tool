use super::BitVecReader;

#[derive(Default)]
pub struct PPSNal {
    pub nal_index: usize,
}

impl PPSNal {
    pub fn parse(bs: &mut BitVecReader) -> PPSNal {
        let mut pps = PPSNal::default();

        pps
    }
}