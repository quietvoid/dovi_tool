use anyhow::{Result, ensure};
use bitvec_helpers::{
    bitstream_io_reader::BsIoSliceReader, bitstream_io_writer::BitstreamIoWriter,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::{ExtMetadataBlock, ExtMetadataBlockInfo};

/// Content type metadata level
#[repr(C)]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct ExtMetadataBlockLevel11 {
    // byte0
    pub content_type: u8,

    // byte1
    pub whitepoint: u8,
    pub reference_mode_flag: bool,

    #[cfg_attr(feature = "serde", serde(default))]
    pub reserved_byte2: u8,

    /// Deprecated
    /// Ignored, use `de_judder_strength` and `smoothness_strength` fields.
    #[deprecated = "Ignored, use `de_judder_strength` and `smoothness_strength` fields."]
    #[cfg_attr(feature = "serde", serde(default))]
    pub reserved_byte3: u8,

    // byte3
    #[cfg_attr(feature = "serde", serde(default))]
    pub de_judder_strength: u8,
    #[cfg_attr(feature = "serde", serde(default))]
    pub smoothness_strength: u8,

    // part of byte1, at the end for repr(C)
    #[cfg_attr(feature = "serde", serde(default))]
    pub motion_control: bool,
}

impl ExtMetadataBlockLevel11 {
    pub(crate) fn parse(reader: &mut BsIoSliceReader) -> Result<ExtMetadataBlock> {
        let mut l11 = Self {
            content_type: reader.read::<8, u8>()?,
            ..Default::default()
        };

        l11.decode_byte1(reader.read::<8, u8>()?);
        l11.reserved_byte2 = reader.read::<8, u8>()?;
        l11.decode_byte3(reader.read::<8, u8>()?);

        Ok(ExtMetadataBlock::Level11(l11))
    }

    pub fn write(&self, writer: &mut BitstreamIoWriter) -> Result<()> {
        self.validate()?;

        writer.write::<8, u8>(self.content_type)?;
        writer.write::<8, u8>(self.encode_byte1())?;
        writer.write::<8, u8>(self.reserved_byte2)?;
        writer.write::<8, u8>(self.encode_byte3())?;

        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        ensure!(self.content_type <= 15);
        ensure!(self.whitepoint <= 15);
        ensure!(self.de_judder_strength <= 15);
        ensure!(self.smoothness_strength <= 15);

        Ok(())
    }

    /// Cinema, D65 whitepoint
    pub const fn default_cinema() -> Self {
        // FIXME: byte3 deprecation
        #[allow(deprecated)]
        Self {
            content_type: 1,
            whitepoint: 0,
            reference_mode_flag: false,
            motion_control: false,
            reserved_byte2: 0,
            reserved_byte3: 0,
            de_judder_strength: 0,
            smoothness_strength: 0,
        }
    }

    /// Cinema, reference mode, D65 whitepoint
    pub const fn default_reference_cinema() -> Self {
        let mut meta = Self::default_cinema();
        meta.reference_mode_flag = true;

        meta
    }

    const fn decode_byte1(&mut self, v: u8) {
        // lowest 4 bits
        self.whitepoint = v & 0x0F;

        // Verifier: "Level 11 byte 1 two most significant bits must be zero"
        let remaining = v >> 4;

        // last 2 bits
        self.motion_control = (remaining >> 1) & 0x01 == 1;
        self.reference_mode_flag = remaining & 0x01 == 1;
    }

    const fn encode_byte1(&self) -> u8 {
        let reference_mode_flag = self.reference_mode_flag as u8;
        let motion_control = self.motion_control as u8;
        let msb = (motion_control << 1) | reference_mode_flag;

        msb << 4 | self.whitepoint
    }

    const fn decode_byte3(&mut self, v: u8) {
        // 4 bits per field
        self.de_judder_strength = v & 0x0F;
        self.smoothness_strength = v >> 4;
    }

    const fn encode_byte3(&self) -> u8 {
        self.smoothness_strength << 4 | self.de_judder_strength
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

#[cfg(test)]
mod tests {
    use super::{
        BitstreamIoWriter, BsIoSliceReader, ExtMetadataBlock, ExtMetadataBlockLevel11, Result,
    };

    fn assert_parsed_metadata_equality(meta: ExtMetadataBlockLevel11, slice: &[u8]) -> Result<()> {
        let mut reader = BsIoSliceReader::from_slice(slice);
        let ExtMetadataBlock::Level11(parsed) = ExtMetadataBlockLevel11::parse(&mut reader)? else {
            unreachable!();
        };
        assert_eq!(meta, parsed);

        Ok(())
    }

    #[test]
    fn byte1_all_roundtrip() -> Result<()> {
        let meta = ExtMetadataBlockLevel11 {
            content_type: 1,
            whitepoint: 3,
            reference_mode_flag: true,
            motion_control: true,
            ..Default::default()
        };

        let mut writer = BitstreamIoWriter::with_capacity(4);
        meta.write(&mut writer)?;

        let slice = writer.as_slice().unwrap();
        assert_eq!(slice, &[1, 51, 0, 0]);

        assert_parsed_metadata_equality(meta, slice)?;

        Ok(())
    }

    #[test]
    fn byte1_reference_mode_roundtrip() -> Result<()> {
        let meta = ExtMetadataBlockLevel11 {
            content_type: 1,
            reference_mode_flag: true,
            motion_control: false,
            ..Default::default()
        };

        let mut writer = BitstreamIoWriter::with_capacity(4);
        meta.write(&mut writer)?;

        let slice = writer.as_slice().unwrap();
        assert_eq!(slice, &[1, 16, 0, 0]);

        assert_parsed_metadata_equality(meta, slice)?;

        Ok(())
    }

    #[test]
    fn byte1_motion_control_roundtrip() -> Result<()> {
        let meta = ExtMetadataBlockLevel11 {
            content_type: 1,
            reference_mode_flag: false,
            motion_control: true,
            ..Default::default()
        };

        let mut writer = BitstreamIoWriter::with_capacity(4);
        meta.write(&mut writer)?;

        let slice = writer.as_slice().unwrap();
        assert_eq!(slice, &[1, 32, 0, 0]);

        assert_parsed_metadata_equality(meta, slice)?;

        Ok(())
    }

    #[test]
    fn byte3_all_roundtrip() -> Result<()> {
        let meta = ExtMetadataBlockLevel11 {
            content_type: 4,
            whitepoint: 2,
            motion_control: true,
            de_judder_strength: 15,
            smoothness_strength: 15,
            ..Default::default()
        };

        let mut writer = BitstreamIoWriter::with_capacity(4);
        meta.write(&mut writer)?;

        let slice = writer.as_slice().unwrap();
        assert_eq!(slice, &[4, 34, 0, 255]);

        assert_parsed_metadata_equality(meta, slice)?;

        Ok(())
    }
}
