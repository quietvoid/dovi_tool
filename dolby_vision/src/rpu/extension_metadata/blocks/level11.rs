use anyhow::{Result, ensure};
use bitvec_helpers::{
    bitstream_io_reader::BsIoSliceReader, bitstream_io_writer::BitstreamIoWriter,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::{ExtMetadataBlock, ExtMetadataBlockInfo};

/// Content type metadata level
#[repr(C)]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct ExtMetadataBlockLevel11 {
    pub content_type: u8,
    pub whitepoint: u8,
    pub reference_mode_flag: bool,

    #[cfg_attr(feature = "serde", serde(default))]
    pub reserved_byte2: u8,
    #[cfg_attr(feature = "serde", serde(default))]
    pub reserved_byte3: u8,
}

impl ExtMetadataBlockLevel11 {
    pub(crate) fn parse(reader: &mut BsIoSliceReader) -> Result<ExtMetadataBlock> {
        let mut l11 = Self {
            content_type: reader.read::<8, u8>()?,
            ..Default::default()
        };

        l11.decode_byte1(reader.read::<8, u8>()?);
        l11.reserved_byte2 = reader.read::<8, u8>()?;
        l11.reserved_byte3 = reader.read::<8, u8>()?;

        Ok(ExtMetadataBlock::Level11(l11))
    }

    pub fn write(&self, writer: &mut BitstreamIoWriter) -> Result<()> {
        self.validate()?;

        writer.write::<8, u8>(self.content_type)?;
        writer.write::<8, u8>(self.encode_byte1())?;
        writer.write::<8, u8>(self.reserved_byte2)?;
        writer.write::<8, u8>(self.reserved_byte3)?;

        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        ensure!(self.content_type <= 15);
        ensure!(self.whitepoint <= 15);
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

    const fn decode_byte1(&mut self, v: u8) {
        // lowest 4 bits
        self.whitepoint = v & 0x0F;

        let remaining = v >> 4;
        self.reference_mode_flag = remaining & 0x01 == 1;
    }

    const fn encode_byte1(&self) -> u8 {
        let reference_mode_flag = self.reference_mode_flag as u8;
        reference_mode_flag << 4 | self.whitepoint
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
