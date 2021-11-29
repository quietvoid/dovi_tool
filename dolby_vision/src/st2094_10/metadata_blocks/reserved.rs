use anyhow::Result;

use bitvec::{order::Msb0, prelude::BitVec};
use bitvec_helpers::{bitvec_reader::BitVecReader, bitvec_writer::BitVecWriter};

#[cfg(feature = "serde_feature")]
use serde::{Deserialize, Serialize};

use super::ExtMetadataBlock;

#[derive(Debug, Default)]
#[cfg_attr(feature = "serde_feature", derive(Serialize, Deserialize))]
pub struct ReservedExtMetadataBlock {
    pub ext_block_length: u64,
    pub ext_block_level: u8,

    #[cfg_attr(
        feature = "serde_feature",
        serde(serialize_with = "crate::utils::bitvec_ser_bits", skip_deserializing)
    )]
    pub data: BitVec<Msb0, u8>,
}

impl ReservedExtMetadataBlock {
    pub fn parse(
        ext_block_length: u64,
        ext_block_level: u8,
        reader: &mut BitVecReader,
    ) -> Result<ExtMetadataBlock> {
        let bits = 8 * ext_block_length;
        let mut data = BitVec::new();

        for _ in 0..bits {
            data.push(reader.get()?);
        }

        Ok(ExtMetadataBlock::Reserved(Self {
            ext_block_length,
            ext_block_level,
            data,
        }))
    }

    pub fn write(&self, writer: &mut BitVecWriter) {
        self.data.iter().for_each(|b| writer.write(*b));
    }
}
