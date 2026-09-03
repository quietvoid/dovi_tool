use anyhow::{Result, ensure};
use bitvec_helpers::{
    bitstream_io_reader::BsIoSliceReader, bitstream_io_writer::BitstreamIoWriter,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::{ExtMetadataBlock, ExtMetadataBlockInfo};

// Unknown - filler bytes?
#[repr(C)]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct ExtMetadataBlockLevel253 {
    pub bytes: Vec<u8>,
}

pub const LEVEL253_MAX_LENGTH: usize = 66;
pub const LEVEL253_FILLER_BYTE: u8 = 0x55;

impl ExtMetadataBlockLevel253 {
    pub(crate) fn parse(reader: &mut BsIoSliceReader, length: u64) -> Result<ExtMetadataBlock> {
        Self::ensure_valid_length(length as usize)?;

        let mut bytes = vec![0; length as usize];
        reader.read_bytes(&mut bytes)?;

        Ok(ExtMetadataBlock::Level253(Self { bytes }))
    }

    pub fn validate(&self) -> Result<()> {
        Self::ensure_valid_length(self.bytes.len())?;

        ensure!(
            self.bytes.iter().all(|e| *e == LEVEL253_FILLER_BYTE),
            "Level 253 filler bytes are expected to equal {LEVEL253_FILLER_BYTE:#x}"
        );

        Ok(())
    }

    pub fn write(&self, writer: &mut BitstreamIoWriter) -> Result<()> {
        self.validate()?;

        writer.write_bytes(&self.bytes)?;

        Ok(())
    }

    fn ensure_valid_length(length: usize) -> Result<()> {
        ensure!(
            length <= LEVEL253_MAX_LENGTH,
            "Level 253 block should be at most {LEVEL253_MAX_LENGTH} bytes"
        );

        Ok(())
    }
}

impl ExtMetadataBlockInfo for ExtMetadataBlockLevel253 {
    fn level(&self) -> u8 {
        253
    }

    fn bytes_size(&self) -> u64 {
        self.bytes.len() as u64
    }

    fn required_bits(&self) -> u64 {
        self.bytes_size() * 8
    }
}
