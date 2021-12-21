use bitvec_helpers::{bitvec_reader::BitVecReader, bitvec_writer::BitVecWriter};

#[cfg(feature = "serde_feature")]
use serde::{Deserialize, Serialize};

use super::{ExtMetadataBlock, ExtMetadataBlockInfo};

/// Content type metadata level
#[repr(C)]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde_feature", derive(Deserialize, Serialize))]
pub struct ExtMetadataBlockLevel11 {
    pub content_type: u8,
    pub content_sub_type: u8,
    pub whitepoint: u8,
    pub reference_mode_flag: bool,
    pub reserved_3_bits: u8,

    // Desired enhancements
    pub sharpness: u8,
    pub noise_reduction: u8,
    pub mpeg_noise_reduction: u8,
    pub frame_rate_conversion: u8,
    pub brightness: u8,
    pub color: u8,

    pub reserved_2_bits1: u8,
    pub reserved_2_bits2: u8,
}

impl ExtMetadataBlockLevel11 {
    pub fn parse(reader: &mut BitVecReader) -> ExtMetadataBlock {
        ExtMetadataBlock::Level11(Self {
            content_type: reader.get_n(4),
            content_sub_type: reader.get_n(4),
            whitepoint: reader.get_n(4),
            reference_mode_flag: reader.get().unwrap_or(false),
            reserved_3_bits: reader.get_n(3),
            sharpness: reader.get_n(2),
            noise_reduction: reader.get_n(2),
            mpeg_noise_reduction: reader.get_n(2),
            frame_rate_conversion: reader.get_n(2),
            brightness: reader.get_n(2),
            color: reader.get_n(2),
            reserved_2_bits1: reader.get_n(2),
            reserved_2_bits2: reader.get_n(2),
        })
    }

    pub fn write(&self, writer: &mut BitVecWriter) {
        writer.write_n(&self.content_type.to_be_bytes(), 4);
        writer.write_n(&self.content_sub_type.to_be_bytes(), 4);
        writer.write_n(&self.whitepoint.to_be_bytes(), 4);
        writer.write(self.reference_mode_flag);
        writer.write_n(&self.reserved_3_bits.to_be_bytes(), 3);
        writer.write_n(&self.sharpness.to_be_bytes(), 2);
        writer.write_n(&self.noise_reduction.to_be_bytes(), 2);
        writer.write_n(&self.mpeg_noise_reduction.to_be_bytes(), 2);
        writer.write_n(&self.frame_rate_conversion.to_be_bytes(), 2);
        writer.write_n(&self.brightness.to_be_bytes(), 2);
        writer.write_n(&self.color.to_be_bytes(), 2);
        writer.write_n(&self.reserved_2_bits1.to_be_bytes(), 2);
        writer.write_n(&self.reserved_2_bits2.to_be_bytes(), 2);
    }

    /// Cinema, reference mode, D65 whitepoint, enhancements disabled
    pub fn default_reference_cinema() -> Self {
        Self {
            content_type: 1,
            content_sub_type: 0,
            whitepoint: 4,
            reference_mode_flag: true,
            reserved_3_bits: 0,
            sharpness: 1,
            noise_reduction: 1,
            mpeg_noise_reduction: 1,
            frame_rate_conversion: 1,
            brightness: 0,
            color: 0,
            reserved_2_bits1: 0,
            reserved_2_bits2: 0,
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
