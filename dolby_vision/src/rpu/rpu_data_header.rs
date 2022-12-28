use anyhow::{ensure, Result};
use bitvec_helpers::{bitslice_reader::BitSliceReader, bitvec_writer::BitVecWriter};

#[cfg(feature = "serde_feature")]
use serde::Serialize;

use super::{dovi_rpu::DoviRpu, NUM_COMPONENTS};

const NLQ_NUM_PIVOTS: usize = 2;

#[derive(Default, Debug, Clone)]
#[cfg_attr(feature = "serde_feature", derive(Serialize))]
pub struct RpuDataHeader {
    pub rpu_nal_prefix: u8,
    pub rpu_type: u8,
    pub rpu_format: u16,
    pub vdr_rpu_profile: u8,
    pub vdr_rpu_level: u8,
    pub vdr_seq_info_present_flag: bool,
    pub chroma_resampling_explicit_filter_flag: bool,
    pub coefficient_data_type: u8,
    pub coefficient_log2_denom: u64,
    pub vdr_rpu_normalized_idc: u8,
    pub bl_video_full_range_flag: bool,
    pub bl_bit_depth_minus8: u64,
    pub el_bit_depth_minus8: u64,
    pub vdr_bit_depth_minus_8: u64,
    pub spatial_resampling_filter_flag: bool,
    pub reserved_zero_3bits: u8,
    pub el_spatial_resampling_filter_flag: bool,
    pub disable_residual_flag: bool,
    pub vdr_dm_metadata_present_flag: bool,
    pub use_prev_vdr_rpu_flag: bool,
    pub prev_vdr_rpu_id: u64,
    pub vdr_rpu_id: u64,
    pub mapping_color_space: u64,
    pub mapping_chroma_format_idc: u64,
    pub num_pivots_minus_2: [u64; NUM_COMPONENTS],
    pub pred_pivot_value: [Vec<u64>; NUM_COMPONENTS],
    pub nlq_method_idc: Option<u8>,
    pub nlq_num_pivots_minus2: Option<u8>,
    pub nlq_pred_pivot_value: Option<[u64; NLQ_NUM_PIVOTS]>,
    pub num_x_partitions_minus1: u64,
    pub num_y_partitions_minus1: u64,
}

pub(crate) fn rpu_data_header(dovi_rpu: &mut DoviRpu, reader: &mut BitSliceReader) -> Result<()> {
    dovi_rpu.header = RpuDataHeader::parse(reader)?;

    Ok(())
}

impl RpuDataHeader {
    pub(crate) fn parse(reader: &mut BitSliceReader) -> Result<RpuDataHeader> {
        let mut rpu_nal = RpuDataHeader {
            rpu_nal_prefix: reader.get_n(8)?,
            ..Default::default()
        };

        if rpu_nal.rpu_nal_prefix == 25 {
            rpu_nal.rpu_type = reader.get_n(6)?;
            rpu_nal.rpu_format = reader.get_n(11)?;

            if rpu_nal.rpu_type == 2 {
                rpu_nal.vdr_rpu_profile = reader.get_n(4)?;

                rpu_nal.vdr_rpu_level = reader.get_n(4)?;
                rpu_nal.vdr_seq_info_present_flag = reader.get()?;

                if rpu_nal.vdr_seq_info_present_flag {
                    rpu_nal.chroma_resampling_explicit_filter_flag = reader.get()?;
                    rpu_nal.coefficient_data_type = reader.get_n(2)?;

                    if rpu_nal.coefficient_data_type == 0 {
                        rpu_nal.coefficient_log2_denom = reader.get_ue()?;
                    }

                    rpu_nal.vdr_rpu_normalized_idc = reader.get_n(2)?;
                    rpu_nal.bl_video_full_range_flag = reader.get()?;

                    if rpu_nal.rpu_format & 0x700 == 0 {
                        rpu_nal.bl_bit_depth_minus8 = reader.get_ue()?;
                        rpu_nal.el_bit_depth_minus8 = reader.get_ue()?;
                        rpu_nal.vdr_bit_depth_minus_8 = reader.get_ue()?;
                        rpu_nal.spatial_resampling_filter_flag = reader.get()?;
                        rpu_nal.reserved_zero_3bits = reader.get_n(3)?;
                        rpu_nal.el_spatial_resampling_filter_flag = reader.get()?;
                        rpu_nal.disable_residual_flag = reader.get()?;
                    }
                }

                rpu_nal.vdr_dm_metadata_present_flag = reader.get()?;
                rpu_nal.use_prev_vdr_rpu_flag = reader.get()?;

                if rpu_nal.use_prev_vdr_rpu_flag {
                    rpu_nal.prev_vdr_rpu_id = reader.get_ue()?;
                } else {
                    rpu_nal.vdr_rpu_id = reader.get_ue()?;
                    rpu_nal.mapping_color_space = reader.get_ue()?;
                    rpu_nal.mapping_chroma_format_idc = reader.get_ue()?;

                    let bl_bit_depth = (rpu_nal.bl_bit_depth_minus8 + 8) as usize;

                    for cmp in 0..NUM_COMPONENTS {
                        rpu_nal.num_pivots_minus_2[cmp] = reader.get_ue()?;

                        let pivot_idx_count = (rpu_nal.num_pivots_minus_2[cmp] + 2) as usize;
                        rpu_nal.pred_pivot_value[cmp] = Vec::with_capacity(pivot_idx_count);
                        rpu_nal.pred_pivot_value[cmp]
                            .resize_with(pivot_idx_count, Default::default);

                        for pivot_idx in 0..pivot_idx_count {
                            rpu_nal.pred_pivot_value[cmp][pivot_idx] =
                                reader.get_n(bl_bit_depth)?;
                        }
                    }

                    // Profile 7 only
                    if rpu_nal.rpu_format & 0x700 == 0 && !rpu_nal.disable_residual_flag {
                        rpu_nal.nlq_method_idc = Some(reader.get_n(3)?);
                        rpu_nal.nlq_num_pivots_minus2 = Some(0);

                        let mut nlq_pred_pivot_value = [0; NLQ_NUM_PIVOTS];
                        for pv in &mut nlq_pred_pivot_value {
                            *pv = reader.get_n(bl_bit_depth)?;
                        }

                        rpu_nal.nlq_pred_pivot_value = Some(nlq_pred_pivot_value);
                    }

                    rpu_nal.num_x_partitions_minus1 = reader.get_ue()?;
                    rpu_nal.num_y_partitions_minus1 = reader.get_ue()?;
                }
            }
        }

        Ok(rpu_nal)
    }

    pub fn validate(&self, profile: u8) -> Result<()> {
        ensure!(self.rpu_nal_prefix == 25, "rpu_nal_prefix should be 25");

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
                ensure!(
                    self.nlq_method_idc.is_none(),
                    "profile 5: nlq_method_idc should be undefined"
                );
                ensure!(
                    self.nlq_num_pivots_minus2.is_none(),
                    "profile 5: nlq_num_pivots_minus2 should be undefined"
                );
                ensure!(
                    self.nlq_pred_pivot_value.is_none(),
                    "profile 5: nlq_pred_pivot_value should be undefined"
                );
            }
            7 => {
                ensure!(
                    self.vdr_rpu_profile == 1,
                    "profile 7: vdr_rpu_profile should be 1"
                );
                ensure!(
                    self.nlq_pred_pivot_value.is_some(),
                    "profile 7: nlq_pred_pivot_value should be defined"
                );

                if let Some(nlq_pred_pivot_value) = self.nlq_pred_pivot_value {
                    ensure!(
                        nlq_pred_pivot_value.iter().sum::<u64>() == 1023,
                        "profile 7: nlq_pred_pivot_value elements should add up to the BL bit depth"
                    );
                }
            }
            8 => {
                ensure!(
                    self.vdr_rpu_profile == 1,
                    "profile 8: vdr_rpu_profile should be 1"
                );
                ensure!(
                    self.nlq_method_idc.is_none(),
                    "profile 8: nlq_method_idc should be undefined"
                );
                ensure!(
                    self.nlq_num_pivots_minus2.is_none(),
                    "profile 8: nlq_num_pivots_minus2 should be undefined"
                );
                ensure!(
                    self.nlq_pred_pivot_value.is_none(),
                    "profile 8: nlq_pred_pivot_value should be undefined"
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
            self.vdr_bit_depth_minus_8 <= 6,
            "vdr_bit_depth_minus_8 should be <= 6"
        );
        ensure!(
            self.mapping_color_space == 0,
            "mapping_color_space should be 0"
        );
        ensure!(
            self.mapping_chroma_format_idc == 0,
            "mapping_chroma_format_idc should be 0"
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
                    if self.vdr_bit_depth_minus_8 == 4 {
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

    pub fn write_header(&self, writer: &mut BitVecWriter) {
        writer.write_n(&self.rpu_nal_prefix.to_be_bytes(), 8);

        if self.rpu_nal_prefix == 25 {
            writer.write_n(&self.rpu_type.to_be_bytes(), 6);
            writer.write_n(&self.rpu_format.to_be_bytes(), 11);

            if self.rpu_type == 2 {
                writer.write_n(&self.vdr_rpu_profile.to_be_bytes(), 4);
                writer.write_n(&self.vdr_rpu_level.to_be_bytes(), 4);
                writer.write(self.vdr_seq_info_present_flag);

                if self.vdr_seq_info_present_flag {
                    writer.write(self.chroma_resampling_explicit_filter_flag);
                    writer.write_n(&self.coefficient_data_type.to_be_bytes(), 2);

                    if self.coefficient_data_type == 0 {
                        writer.write_ue(self.coefficient_log2_denom);
                    }

                    writer.write_n(&self.vdr_rpu_normalized_idc.to_be_bytes(), 2);
                    writer.write(self.bl_video_full_range_flag);

                    if self.rpu_format & 0x700 == 0 {
                        writer.write_ue(self.bl_bit_depth_minus8);
                        writer.write_ue(self.el_bit_depth_minus8);
                        writer.write_ue(self.vdr_bit_depth_minus_8);
                        writer.write(self.spatial_resampling_filter_flag);
                        writer.write_n(&self.reserved_zero_3bits.to_be_bytes(), 3);
                        writer.write(self.el_spatial_resampling_filter_flag);
                        writer.write(self.disable_residual_flag);
                    }
                }

                writer.write(self.vdr_dm_metadata_present_flag);
                writer.write(self.use_prev_vdr_rpu_flag);

                if self.use_prev_vdr_rpu_flag {
                    writer.write_ue(self.prev_vdr_rpu_id);
                } else {
                    writer.write_ue(self.vdr_rpu_id);
                    writer.write_ue(self.mapping_color_space);
                    writer.write_ue(self.mapping_chroma_format_idc);

                    for cmp in 0..NUM_COMPONENTS {
                        writer.write_ue(self.num_pivots_minus_2[cmp]);

                        let pivot_idx_count = (self.num_pivots_minus_2[cmp] + 2) as usize;

                        for pivot_idx in 0..pivot_idx_count {
                            writer.write_n(
                                &self.pred_pivot_value[cmp][pivot_idx].to_be_bytes(),
                                (self.bl_bit_depth_minus8 + 8) as usize,
                            );
                        }
                    }

                    if self.rpu_format & 0x700 == 0 && !self.disable_residual_flag {
                        if let Some(nlq_method_idc) = self.nlq_method_idc {
                            writer.write_n(&nlq_method_idc.to_be_bytes(), 3);
                        }

                        if let Some(nlq_pred_pivot_value) = &self.nlq_pred_pivot_value {
                            nlq_pred_pivot_value.iter().for_each(|pv| {
                                writer.write_n(
                                    &pv.to_be_bytes(),
                                    (self.bl_bit_depth_minus8 + 8) as usize,
                                )
                            });
                        }
                    }

                    writer.write_ue(self.num_x_partitions_minus1);
                    writer.write_ue(self.num_y_partitions_minus1);
                }
            }
        }
    }

    pub fn p5_default() -> RpuDataHeader {
        RpuDataHeader {
            vdr_rpu_profile: 0,
            bl_video_full_range_flag: true,
            ..RpuDataHeader::p8_default()
        }
    }

    pub fn p8_default() -> RpuDataHeader {
        RpuDataHeader {
            rpu_nal_prefix: 25,
            rpu_type: 2,
            rpu_format: 18,
            vdr_rpu_profile: 1,
            vdr_rpu_level: 0,
            vdr_seq_info_present_flag: true,
            chroma_resampling_explicit_filter_flag: false,
            coefficient_data_type: 0,
            coefficient_log2_denom: 23,
            vdr_rpu_normalized_idc: 1,
            bl_video_full_range_flag: false,
            bl_bit_depth_minus8: 2,
            el_bit_depth_minus8: 2,
            vdr_bit_depth_minus_8: 4,
            spatial_resampling_filter_flag: false,
            reserved_zero_3bits: 0,
            el_spatial_resampling_filter_flag: false,
            disable_residual_flag: true,
            vdr_dm_metadata_present_flag: true,
            use_prev_vdr_rpu_flag: false,
            prev_vdr_rpu_id: 0,
            vdr_rpu_id: 0,
            mapping_color_space: 0,
            mapping_chroma_format_idc: 0,
            num_pivots_minus_2: [0, 0, 0],
            pred_pivot_value: [vec![0, 1023], vec![0, 1023], vec![0, 1023]],
            nlq_method_idc: None,
            nlq_num_pivots_minus2: None,
            nlq_pred_pivot_value: None,
            num_x_partitions_minus1: 0,
            num_y_partitions_minus1: 0,
        }
    }
}
