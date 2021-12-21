use std::cmp::{max, min};

use bitvec_helpers::{bitvec_reader::BitVecReader, bitvec_writer::BitVecWriter};

#[cfg(feature = "serde_feature")]
use serde::{Deserialize, Serialize};

use super::{ExtMetadataBlock, ExtMetadataBlockInfo};

pub const L1_MIN_PQ_MAX_VALUE: u16 = 12;

pub const L1_MAX_PQ_MIN_VALUE: u16 = 2081;
pub const L1_MAX_PQ_MAX_VALUE: u16 = 4095;

pub const L1_AVG_PQ_MIN_VALUE: u16 = 819;

/// Statistical analysis of the frame: min, max, avg brightness.
#[repr(C)]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde_feature", derive(Deserialize, Serialize))]
pub struct ExtMetadataBlockLevel1 {
    pub min_pq: u16,
    pub max_pq: u16,
    pub avg_pq: u16,
}

impl ExtMetadataBlockLevel1 {
    pub fn parse(reader: &mut BitVecReader) -> ExtMetadataBlock {
        ExtMetadataBlock::Level1(Self {
            min_pq: reader.get_n(12),
            max_pq: reader.get_n(12),
            avg_pq: reader.get_n(12),
        })
    }

    pub fn write(&self, writer: &mut BitVecWriter) {
        writer.write_n(&self.min_pq.to_be_bytes(), 12);
        writer.write_n(&self.max_pq.to_be_bytes(), 12);
        writer.write_n(&self.avg_pq.to_be_bytes(), 12);
    }

    pub fn from_stats(min_pq: u16, max_pq: u16, avg_pq: u16) -> ExtMetadataBlockLevel1 {
        let max_pq = min(max(max_pq, L1_MAX_PQ_MIN_VALUE), L1_MAX_PQ_MAX_VALUE);
        let avg_pq = min(max(avg_pq, L1_AVG_PQ_MIN_VALUE), max_pq - 1);

        ExtMetadataBlockLevel1 {
            min_pq,
            max_pq,
            avg_pq,
        }
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