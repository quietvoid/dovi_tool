use anyhow::{ensure, Result};
use bitvec_helpers::{bitvec_reader::BitVecReader, bitvec_writer::BitVecWriter};

#[cfg(feature = "serde_feature")]
use serde::{Deserialize, Serialize};

use super::{ExtMetadataBlock, ExtMetadataBlockInfo};

const MAX_WHITEPOINT_VALUE: u8 = 15;

/// Content type metadata level
#[repr(C)]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde_feature", derive(Deserialize, Serialize))]
pub struct ExtMetadataBlockLevel11 {
    pub content_type: u8,
    pub whitepoint: u8,
    pub reference_mode_flag: bool,

    #[cfg_attr(feature = "serde_feature", serde(default))]
    pub reserved_byte2: u8,
    #[cfg_attr(feature = "serde_feature", serde(default))]
    pub reserved_byte3: u8,
}

impl ExtMetadataBlockLevel11 {
    pub fn parse(reader: &mut BitVecReader) -> Result<ExtMetadataBlock> {
        let mut l11 = Self {
            content_type: reader.get_n(8)?,
            whitepoint: reader.get_n(8)?,
            reserved_byte2: reader.get_n(8)?,
            reserved_byte3: reader.get_n(8)?,
            ..Default::default()
        };

        if l11.whitepoint > MAX_WHITEPOINT_VALUE {
            l11.reference_mode_flag = true;
            l11.whitepoint -= MAX_WHITEPOINT_VALUE + 1;
        }

        Ok(ExtMetadataBlock::Level11(l11))
    }

    pub fn write(&self, writer: &mut BitVecWriter) -> Result<()> {
        self.validate()?;

        let mut wp = self.whitepoint;

        if self.reference_mode_flag {
            wp += MAX_WHITEPOINT_VALUE + 1
        }

        writer.write_n(&self.content_type.to_be_bytes(), 8);
        writer.write_n(&wp.to_be_bytes(), 8);
        writer.write_n(&self.reserved_byte2.to_be_bytes(), 8);
        writer.write_n(&self.reserved_byte3.to_be_bytes(), 8);

        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        ensure!(self.content_type <= 15);
        ensure!(self.whitepoint <= 15);
        ensure!(self.reserved_byte2 == 0);
        ensure!(self.reserved_byte3 == 0);

        Ok(())
    }

    /// Cinema, reference mode, D65 whitepoint
    pub fn default_reference_cinema() -> Self {
        Self {
            content_type: 1,
            whitepoint: 0,
            reference_mode_flag: true,
            reserved_byte2: 0,
            reserved_byte3: 0,
        }
    }
}

impl ExtMetadataBlockInfo for ExtMetadataBlockLevel11 {
    fn level(&self) -> u8 {
        11
    }

    fn bytes_size(&self) -> u64 {
        4
    }

    fn required_bits(&self) -> u64 {
        32
    }
}
