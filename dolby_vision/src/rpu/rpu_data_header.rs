use anyhow::{bail, ensure, Result};
use bitvec_helpers::{
    bitstream_io_reader::BsIoSliceReader, bitstream_io_writer::BitstreamIoWriter,
};

#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Default, Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct RpuDataHeader {
    /// Must be 25
    #[deprecated(
        since = "3.2.0",
        note = "The field is not actually part of the RPU header"
    )]
    pub rpu_nal_prefix: u8,

    // Must be 2
    pub rpu_type: u8,
    pub rpu_format: u16,
    pub vdr_rpu_profile: u8,
    pub vdr_rpu_level: u8,
    pub vdr_seq_info_present_flag: bool,
    pub chroma_resampling_explicit_filter_flag: bool,
    pub coefficient_data_type: u8,
    pub coefficient_log2_denom: u64,
    /// Calculated for `coefficient_data_type`
    pub coefficient_log2_denom_length: u32,
    pub vdr_rpu_normalized_idc: u8,
    pub bl_video_full_range_flag: bool,

    // [8, 16]
    pub bl_bit_depth_minus8: u64,
    pub el_bit_depth_minus8: u64,

    /// Extended base layer inverse mapping indicator
    pub ext_mapping_idc_lsb: u8,
    /// Reserved
    pub ext_mapping_idc_msb: u8,

    // [8, 16]
    pub vdr_bit_depth_minus8: u64,

    pub spatial_resampling_filter_flag: bool,
    pub reserved_zero_3bits: u8,
    pub el_spatial_resampling_filter_flag: bool,
    pub disable_residual_flag: bool,
    pub vdr_dm_metadata_present_flag: bool,
    pub use_prev_vdr_rpu_flag: bool,

    // [0, 15]
    pub prev_vdr_rpu_id: u64,
}

impl RpuDataHeader {
    pub(crate) fn parse(reader: &mut BsIoSliceReader) -> Result<RpuDataHeader> {
        let rpu_type = reader.get_n(6)?;
        ensure!(rpu_type == 2);

        let rpu_format = reader.get_n(11)?;

        let vdr_rpu_profile = reader.get_n(4)?;
        let vdr_rpu_level = reader.get_n(4)?;

        let vdr_seq_info_present_flag = reader.get()?;

        let mut header = RpuDataHeader {
            rpu_type,
            rpu_format,
            vdr_rpu_profile,
            vdr_rpu_level,
            vdr_seq_info_present_flag,
            ..Default::default()
        };

        if vdr_seq_info_present_flag {
            header.chroma_resampling_explicit_filter_flag = reader.get()?;
            header.coefficient_data_type = reader.get_n(2)?;

            if header.coefficient_data_type == 0 {
                header.coefficient_log2_denom = reader.get_ue()?;
            }

            header.vdr_rpu_normalized_idc = reader.get_n(2)?;
            header.bl_video_full_range_flag = reader.get()?;

            if header.rpu_format & 0x700 == 0 {
                header.bl_bit_depth_minus8 = reader.get_ue()?;

                let el_bit_depth_minus8 = reader.get_ue()?;
                // 8 lowest bits
                header.el_bit_depth_minus8 = el_bit_depth_minus8 & 0xFF;

                // Next 8 bits
                let ext_mapping_idc = ((el_bit_depth_minus8 & 0xFF00) >> 8) as u8;
                // Lowest 5 bits
                header.ext_mapping_idc_lsb = ext_mapping_idc & 0x1F;
                header.ext_mapping_idc_msb = (ext_mapping_idc & 0xE0) >> 5;

                header.vdr_bit_depth_minus8 = reader.get_ue()?;
                header.spatial_resampling_filter_flag = reader.get()?;
                header.reserved_zero_3bits = reader.get_n(3)?;
                header.el_spatial_resampling_filter_flag = reader.get()?;
                header.disable_residual_flag = reader.get()?;
            }

            header.coefficient_log2_denom_length = if header.coefficient_data_type == 0 {
                header.coefficient_log2_denom as u32
            } else if header.coefficient_data_type == 1 {
                32
            } else {
                bail!(
                    "Invalid coefficient_data_type value: {}",
                    header.coefficient_data_type
                );
            };
        }

        header.vdr_dm_metadata_present_flag = reader.get()?;

        header.use_prev_vdr_rpu_flag = reader.get()?;
        if header.use_prev_vdr_rpu_flag {
            header.prev_vdr_rpu_id = reader.get_ue()?;
        }

        Ok(header)
    }

    pub fn validate(&self, profile: u8) -> Result<()> {
        match profile {
            5 => {
                ensure!(
                    self.vdr_rpu_profile == 0,
                    "profile 5: vdr_rpu_profile should be 0"
                );
                ensure!(
                    self.bl_video_full_range_flag,
                    "profile 5: bl_video_full_range_flag should be true"
                );
            }
            7 => {
                ensure!(
                    self.vdr_rpu_profile == 1,
                    "profile 7: vdr_rpu_profile should be 1"
                );
            }
            8 => {
                ensure!(
                    self.vdr_rpu_profile == 1,
                    "profile 8: vdr_rpu_profile should be 1"
                );
            }
            _ => (),
        };

        ensure!(self.vdr_rpu_level == 0, "vdr_rpu_level should be 0");
        ensure!(
            self.bl_bit_depth_minus8 == 2,
            "bl_bit_depth_minus8 should be 2"
        );
        ensure!(
            self.el_bit_depth_minus8 == 2,
            "el_bit_depth_minus8 should be 2"
        );
        ensure!(
            self.vdr_bit_depth_minus8 <= 6,
            "vdr_bit_depth_minus8 should be <= 6"
        );
        ensure!(
            self.coefficient_log2_denom <= 23,
            "coefficient_log2_denom should be <= 23"
        );

        Ok(())
    }

    pub fn get_dovi_profile(&self) -> u8 {
        match self.vdr_rpu_profile {
            0 => {
                // Profile 5 is full range
                if self.bl_video_full_range_flag {
                    5
                } else {
                    0
                }
            }
            1 => {
                // 4, 7 or 8
                if self.el_spatial_resampling_filter_flag && !self.disable_residual_flag {
                    if self.vdr_bit_depth_minus8 == 4 {
                        7
                    } else {
                        4
                    }
                } else {
                    8
                }
            }
            _ => 0,
        }
    }

    pub fn write_header(&self, writer: &mut BitstreamIoWriter) -> Result<()> {
        writer.write_n(&self.rpu_type, 6)?;
        writer.write_n(&self.rpu_format, 11)?;

        writer.write_n(&self.vdr_rpu_profile, 4)?;
        writer.write_n(&self.vdr_rpu_level, 4)?;
        writer.write(self.vdr_seq_info_present_flag)?;

        if self.vdr_seq_info_present_flag {
            writer.write(self.chroma_resampling_explicit_filter_flag)?;
            writer.write_n(&self.coefficient_data_type, 2)?;

            if self.coefficient_data_type == 0 {
                writer.write_ue(&self.coefficient_log2_denom)?;
            }

            writer.write_n(&self.vdr_rpu_normalized_idc, 2)?;
            writer.write(self.bl_video_full_range_flag)?;

            if self.rpu_format & 0x700 == 0 {
                writer.write_ue(&self.bl_bit_depth_minus8)?;

                let ext_mapping_idc =
                    ((self.ext_mapping_idc_msb << 5) | self.ext_mapping_idc_lsb) as u64;
                let el_bit_depth_minus8 = (ext_mapping_idc << 8) | self.el_bit_depth_minus8;
                writer.write_ue(&el_bit_depth_minus8)?;

                writer.write_ue(&self.vdr_bit_depth_minus8)?;
                writer.write(self.spatial_resampling_filter_flag)?;
                writer.write_n(&self.reserved_zero_3bits, 3)?;
                writer.write(self.el_spatial_resampling_filter_flag)?;
                writer.write(self.disable_residual_flag)?;
            }
        }

        writer.write(self.vdr_dm_metadata_present_flag)?;
        writer.write(self.use_prev_vdr_rpu_flag)?;

        if self.use_prev_vdr_rpu_flag {
            writer.write_ue(&self.prev_vdr_rpu_id)?;
        }

        Ok(())
    }

    pub fn p5_default() -> RpuDataHeader {
        RpuDataHeader {
            vdr_rpu_profile: 0,
            bl_video_full_range_flag: true,
            ..RpuDataHeader::p8_default()
        }
    }

    pub fn p8_default() -> RpuDataHeader {
        let mut header = RpuDataHeader {
            rpu_type: 2,
            rpu_format: 18,
            vdr_rpu_profile: 1,
            vdr_rpu_level: 0,
            vdr_seq_info_present_flag: true,
            chroma_resampling_explicit_filter_flag: false,
            coefficient_data_type: 0,
            coefficient_log2_denom: 23,
            coefficient_log2_denom_length: 23,
            vdr_rpu_normalized_idc: 1,
            bl_video_full_range_flag: false,
            bl_bit_depth_minus8: 2,
            el_bit_depth_minus8: 2,
            vdr_bit_depth_minus8: 4,
            spatial_resampling_filter_flag: false,
            reserved_zero_3bits: 0,
            el_spatial_resampling_filter_flag: false,
            disable_residual_flag: true,
            vdr_dm_metadata_present_flag: true,
            use_prev_vdr_rpu_flag: false,
            prev_vdr_rpu_id: 0,
            ..Default::default()
        };

        // FIXME: rpu_nal_prefix deprecation
        #[allow(deprecated)]
        {
            header.rpu_nal_prefix = 25;
        }

        header
    }
}
