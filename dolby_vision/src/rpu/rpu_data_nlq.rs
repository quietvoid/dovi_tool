use std::fmt::Display;
use std::io;

use anyhow::{ensure, Result};
use bitvec_helpers::{
    bitstream_io_reader::BitstreamIoReader, bitstream_io_writer::BitstreamIoWriter,
};

#[cfg(feature = "serde")]
use serde::Serialize;

use super::rpu_data_header::RpuDataHeader;
use super::rpu_data_mapping::{DoviNlqMethod, RpuDataMapping};

use super::NUM_COMPONENTS;

const FEL_STR: &str = "FEL";
const MEL_STR: &str = "MEL";

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum DoviELType {
    MEL,
    FEL,
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct RpuDataNlq {
    // [0, 512]
    pub nlq_offset: [u16; NUM_COMPONENTS],
    pub vdr_in_max_int: [u64; NUM_COMPONENTS],
    pub vdr_in_max: [u64; NUM_COMPONENTS],
    pub linear_deadzone_slope_int: [u64; NUM_COMPONENTS],
    pub linear_deadzone_slope: [u64; NUM_COMPONENTS],
    pub linear_deadzone_threshold_int: [u64; NUM_COMPONENTS],
    pub linear_deadzone_threshold: [u64; NUM_COMPONENTS],
}

impl RpuDataNlq {
    pub(crate) fn parse<R: io::Read + io::Seek>(
        reader: &mut BitstreamIoReader<R>,
        header: &RpuDataHeader,
        mapping: &RpuDataMapping,
    ) -> Result<RpuDataNlq> {
        ensure!(
            mapping.nlq_num_pivots_minus2.is_some(),
            "Shouldn't be in NLQ if not profile 7!"
        );

        let num_pivots = mapping.nlq_num_pivots_minus2.unwrap() as usize + 1;
        ensure!(num_pivots == 1, "NLQ should only have 1 significant pivot");

        let mut data = RpuDataNlq::default();

        let coefficient_log2_denom_length = header.coefficient_log2_denom_length;

        for cmp in 0..NUM_COMPONENTS {
            // rpu_data_nlq_param

            data.nlq_offset[cmp] = reader.get_n((header.el_bit_depth_minus8 + 8) as u32)?;

            if header.coefficient_data_type == 0 {
                data.vdr_in_max_int[cmp] = reader.get_ue()?;
            }

            data.vdr_in_max[cmp] = reader.get_n(coefficient_log2_denom_length)?;

            // NLQ_LINEAR_DZ
            if let Some(nlq_method_idc) = mapping.nlq_method_idc {
                if nlq_method_idc == DoviNlqMethod::LinearDeadzone {
                    if header.coefficient_data_type == 0 {
                        data.linear_deadzone_slope_int[cmp] = reader.get_ue()?;
                    }

                    data.linear_deadzone_slope[cmp] =
                        reader.get_n(coefficient_log2_denom_length)?;

                    if header.coefficient_data_type == 0 {
                        data.linear_deadzone_threshold_int[cmp] = reader.get_ue()?;
                    }

                    data.linear_deadzone_threshold[cmp] =
                        reader.get_n(coefficient_log2_denom_length)?;
                }
            }
        }

        Ok(data)
    }

    pub fn convert_to_mel(&mut self) {
        // Set to 0
        self.nlq_offset.fill(0);
        // Set to 1
        self.vdr_in_max_int.fill(1);
        // Set to 0
        self.vdr_in_max.fill(0);

        self.linear_deadzone_slope_int.fill(0);
        self.linear_deadzone_slope.fill(0);
        self.linear_deadzone_threshold_int.fill(0);
        self.linear_deadzone_threshold.fill(0);
    }

    pub fn write(
        &self,
        writer: &mut BitstreamIoWriter,
        header: &RpuDataHeader,
        mapping: &RpuDataMapping,
    ) -> Result<()> {
        let coefficient_log2_denom_length = header.coefficient_log2_denom_length;

        for cmp in 0..NUM_COMPONENTS {
            // rpu_data_nlq_param

            writer.write_n(
                &self.nlq_offset[cmp],
                (header.el_bit_depth_minus8 + 8) as u32,
            )?;

            if header.coefficient_data_type == 0 {
                writer.write_ue(&self.vdr_in_max_int[cmp])?;
            }

            writer.write_n(&self.vdr_in_max[cmp], coefficient_log2_denom_length)?;

            if let Some(nlq_method_idc) = mapping.nlq_method_idc {
                if nlq_method_idc == DoviNlqMethod::LinearDeadzone {
                    // NLQ_LINEAR_DZ
                    if header.coefficient_data_type == 0 {
                        writer.write_ue(&self.linear_deadzone_slope_int[cmp])?;
                    }

                    writer.write_n(
                        &self.linear_deadzone_slope[cmp],
                        coefficient_log2_denom_length,
                    )?;

                    if header.coefficient_data_type == 0 {
                        writer.write_ue(&self.linear_deadzone_slope_int[cmp])?;
                    }

                    writer.write_n(
                        &self.linear_deadzone_threshold[cmp],
                        coefficient_log2_denom_length,
                    )?;
                }
            }
        }

        Ok(())
    }

    pub fn mel_default() -> Self {
        let zeroed_cmps = [0_u64; NUM_COMPONENTS];
        let vdr_in_max_int = [1; NUM_COMPONENTS];

        Self {
            nlq_offset: [0_u16; NUM_COMPONENTS],
            vdr_in_max_int,
            vdr_in_max: zeroed_cmps,
            linear_deadzone_slope_int: zeroed_cmps,
            linear_deadzone_slope: zeroed_cmps,
            linear_deadzone_threshold_int: zeroed_cmps,
            linear_deadzone_threshold: zeroed_cmps,
        }
    }

    pub fn is_mel(&self) -> bool {
        let zero_nlq_offset = self.nlq_offset.iter().all(|e| *e == 0);
        let one_vdr_in_max_int = self.vdr_in_max_int.iter().all(|e| *e == 1);
        let one_vdr_in_max = self.vdr_in_max.iter().all(|e| *e == 0);
        let zero_dz_slope_int = self.linear_deadzone_slope_int.iter().all(|e| *e == 0);
        let zero_dz_slope = self.linear_deadzone_slope.iter().all(|e| *e == 0);
        let zero_dz_threshold_int = self.linear_deadzone_threshold_int.iter().all(|e| *e == 0);
        let zero_dz_threshold = self.linear_deadzone_threshold.iter().all(|e| *e == 0);

        zero_nlq_offset
            && one_vdr_in_max_int
            && one_vdr_in_max
            && zero_dz_slope_int
            && zero_dz_slope
            && zero_dz_threshold_int
            && zero_dz_threshold
    }

    pub fn el_type(&self) -> DoviELType {
        if self.is_mel() {
            DoviELType::MEL
        } else {
            DoviELType::FEL
        }
    }
}

impl DoviELType {
    pub const fn as_str(&self) -> &'static str {
        match self {
            DoviELType::MEL => MEL_STR,
            DoviELType::FEL => FEL_STR,
        }
    }
}

impl Display for DoviELType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
