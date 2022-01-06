use anyhow::{bail, Result};

use bitvec::{order::Msb0, prelude::BitVec};
use bitvec_helpers::{bitvec_reader::BitVecReader, bitvec_writer::BitVecWriter};

#[cfg(feature = "serde_feature")]
use serde::{Deserialize, Serialize};

use super::{ExtMetadataBlock, ExtMetadataBlockInfo};

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde_feature", derive(Deserialize, Serialize))]
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

    pub fn write(&self, _writer: &mut BitVecWriter) -> Result<()> {
        bail!("Cannot write reserved block");
        // self.data.iter().for_each(|b| writer.write(*b));
    }
}

impl ExtMetadataBlockInfo for ReservedExtMetadataBlock {
    // TODO: Level 255 is actually definded for DM debugging purposes, we may add it.
    fn level(&self) -> u8 {
        0
    }

    fn bytes_size(&self) -> u64 {
        self.ext_block_length
    }

    fn required_bits(&self) -> u64 {
        self.data.len() as u64
    }
}
