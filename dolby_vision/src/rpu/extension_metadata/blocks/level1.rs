use anyhow::{ensure, Result};
use bitvec_helpers::{bitslice_reader::BitSliceReader, bitvec_writer::BitVecWriter};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::rpu::vdr_dm_data::CmVersion;

use super::{ExtMetadataBlock, ExtMetadataBlockInfo};

/// cbindgen:ignore
pub const L1_MIN_PQ_MAX_VALUE: u16 = 12;
/// cbindgen:ignore
pub const L1_MAX_PQ_MIN_VALUE: u16 = 2081;
/// cbindgen:ignore
pub const L1_MAX_PQ_MAX_VALUE: u16 = 4095;
/// cbindgen:ignore
pub const L1_AVG_PQ_MIN_VALUE: u16 = 819;
/// cbindgen:ignore
pub const L1_AVG_PQ_MIN_VALUE_CMV40: u16 = 1229;

/// Statistical analysis of the frame: min, max, avg brightness.
#[repr(C)]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct ExtMetadataBlockLevel1 {
    pub min_pq: u16,
    pub max_pq: u16,
    pub avg_pq: u16,
}

impl ExtMetadataBlockLevel1 {
    pub(crate) fn parse(reader: &mut BitSliceReader) -> Result<ExtMetadataBlock> {
        Ok(ExtMetadataBlock::Level1(Self {
            min_pq: reader.get_n(12)?,
            max_pq: reader.get_n(12)?,
            avg_pq: reader.get_n(12)?,
        }))
    }

    pub fn write(&self, writer: &mut BitVecWriter) -> Result<()> {
        self.validate()?;

        writer.write_n(&self.min_pq, 12);
        writer.write_n(&self.max_pq, 12);
        writer.write_n(&self.avg_pq, 12);

        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        ensure!(self.min_pq <= L1_MAX_PQ_MAX_VALUE);
        ensure!(self.max_pq <= L1_MAX_PQ_MAX_VALUE);
        ensure!(self.avg_pq <= L1_MAX_PQ_MAX_VALUE);

        Ok(())
    }

    pub fn new(min_pq: u16, max_pq: u16, avg_pq: u16) -> ExtMetadataBlockLevel1 {
        ExtMetadataBlockLevel1 {
            min_pq,
            max_pq,
            avg_pq,
        }
    }

    fn clamp_values_int(&mut self, cm_version: CmVersion) {
        let avg_min_value = match cm_version {
            CmVersion::V29 => L1_AVG_PQ_MIN_VALUE,
            CmVersion::V40 => L1_AVG_PQ_MIN_VALUE_CMV40,
        };

        self.min_pq = self.min_pq.clamp(0, L1_MIN_PQ_MAX_VALUE);
        self.max_pq = self.max_pq.clamp(L1_MAX_PQ_MIN_VALUE, L1_MAX_PQ_MAX_VALUE);
        self.avg_pq = self.avg_pq.clamp(avg_min_value, self.max_pq - 1);
    }

    // Returns a L1 metadata block clamped to valid values
    pub fn from_stats_cm_version(
        min_pq: u16,
        max_pq: u16,
        avg_pq: u16,
        cm_version: CmVersion,
    ) -> ExtMetadataBlockLevel1 {
        let mut block = Self::new(min_pq, max_pq, avg_pq);
        block.clamp_values_int(cm_version);

        block
    }

    pub fn clamp_values_cm_version(&mut self, cm_version: CmVersion) {
        self.clamp_values_int(cm_version);
    }

    #[deprecated(
        since = "1.7.1",
        note = "Replaced by `from_stats_cm_version` with CmVersion::V29"
    )]
    pub fn from_stats(min_pq: u16, max_pq: u16, avg_pq: u16) -> ExtMetadataBlockLevel1 {
        Self::from_stats_cm_version(min_pq, max_pq, avg_pq, CmVersion::V29)
    }

    #[deprecated(
        since = "1.7.1",
        note = "Replaced by `clamp_values_cm_version` with CmVersion::V29"
    )]
    pub fn clamp_values(&mut self) {
        self.clamp_values_int(CmVersion::V29);
    }
}

impl ExtMetadataBlockInfo for ExtMetadataBlockLevel1 {
    fn level(&self) -> u8 {
        1
    }

    fn bytes_size(&self) -> u64 {
        5
    }

    fn required_bits(&self) -> u64 {
        36
    }
}
