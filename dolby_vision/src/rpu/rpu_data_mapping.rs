use anyhow::{bail, ensure, Result};
use bitvec_helpers::{
    bitstream_io_reader::BsIoSliceReader, bitstream_io_writer::BitstreamIoWriter,
};

#[cfg(feature = "serde")]
use serde::Serialize;

use crate::rpu::MMR_MAX_COEFFS;

use super::rpu_data_header::RpuDataHeader;
use super::rpu_data_nlq::{DoviELType, RpuDataNlq};

use super::{NLQ_NUM_PIVOTS, NUM_COMPONENTS};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum DoviMappingMethod {
    /// Not a valid value, placeholder for Default
    Invalid = 255,

    Polynomial = 0,
    MMR,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum DoviNlqMethod {
    LinearDeadzone = 0,
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct RpuDataMapping {
    // [0, 15]
    pub vdr_rpu_id: u64,
    pub mapping_color_space: u64,
    pub mapping_chroma_format_idc: u64,
    pub num_x_partitions_minus1: u64,
    pub num_y_partitions_minus1: u64,

    pub curves: [DoviReshapingCurve; NUM_COMPONENTS],

    // NLQ params
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub nlq_method_idc: Option<DoviNlqMethod>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub nlq_num_pivots_minus2: Option<u8>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub nlq_pred_pivot_value: Option<[u16; NLQ_NUM_PIVOTS]>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub nlq: Option<RpuDataNlq>,
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct DoviReshapingCurve {
    // [2, 9]
    pub num_pivots_minus2: u64,
    pub pivots: Vec<u16>,

    // Consistent for a component
    // Luma (component 0): Polynomial
    // Chroma (components 1 and 2): MMR
    pub mapping_idc: DoviMappingMethod,

    /// DoviMappingMethod::Polynomial
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub polynomial: Option<DoviPolynomialCurve>,

    /// DoviMappingMethod::MMR
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub mmr: Option<DoviMMRCurve>,
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct DoviPolynomialCurve {
    pub poly_order_minus1: Vec<u64>,
    pub linear_interp_flag: Vec<bool>,
    pub poly_coef_int: Vec<Vec<i64>>,
    pub poly_coef: Vec<Vec<u64>>,
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct DoviMMRCurve {
    pub mmr_order_minus1: Vec<u8>,
    pub mmr_constant_int: Vec<i64>,
    pub mmr_constant: Vec<u64>,
    pub mmr_coef_int: Vec<Vec<Vec<i64>>>,
    pub mmr_coef: Vec<Vec<Vec<u64>>>,
}

impl RpuDataMapping {
    pub(crate) fn parse(
        reader: &mut BsIoSliceReader,
        header: &RpuDataHeader,
    ) -> Result<RpuDataMapping> {
        let mut mapping = RpuDataMapping {
            vdr_rpu_id: reader.get_ue()?,
            mapping_color_space: reader.get_ue()?,
            mapping_chroma_format_idc: reader.get_ue()?,
            ..Default::default()
        };

        let bl_bit_depth = (header.bl_bit_depth_minus8 + 8) as u32;

        for cmp in 0..NUM_COMPONENTS {
            let mut curve = &mut mapping.curves[cmp];

            curve.num_pivots_minus2 = reader.get_ue()?;
            let num_pivots = (curve.num_pivots_minus2 + 2) as usize;

            curve.pivots = Vec::with_capacity(num_pivots);
            curve.pivots.resize(num_pivots, Default::default());

            for i in 0..num_pivots {
                curve.pivots[i] = reader.get_n(bl_bit_depth)?;
            }
        }

        // Profile 7 only
        if header.rpu_format & 0x700 == 0 && !header.disable_residual_flag {
            let nlq_method_idc = reader.get_n::<u8>(3)?;
            ensure!(nlq_method_idc == 0);

            mapping.nlq_method_idc = Some(DoviNlqMethod::from(nlq_method_idc));
            mapping.nlq_num_pivots_minus2 = Some(0);

            let mut nlq_pred_pivot_value = [0; NLQ_NUM_PIVOTS];
            for pv in &mut nlq_pred_pivot_value {
                *pv = reader.get_n(bl_bit_depth)?;
            }

            mapping.nlq_pred_pivot_value = Some(nlq_pred_pivot_value);
        }

        mapping.num_x_partitions_minus1 = reader.get_ue()?;
        mapping.num_y_partitions_minus1 = reader.get_ue()?;

        // rpu_data_mapping_param

        for cmp in 0..NUM_COMPONENTS {
            let curve = &mut mapping.curves[cmp];
            let num_pieces = (curve.num_pivots_minus2 + 1) as usize;

            for _ in 0..num_pieces {
                let mapping_idc = DoviMappingMethod::from(reader.get_ue()?);
                curve.mapping_idc = mapping_idc;

                // MAPPING_POLYNOMIAL
                if mapping_idc == DoviMappingMethod::Polynomial {
                    let poly_curve = curve
                        .polynomial
                        .get_or_insert_with(|| DoviPolynomialCurve::new(num_pieces));

                    poly_curve.parse(reader, header)?;
                } else if mapping_idc == DoviMappingMethod::MMR {
                    let mmr_curve = curve
                        .mmr
                        .get_or_insert_with(|| DoviMMRCurve::new(num_pieces));

                    mmr_curve.parse(reader, header)?;
                }
            }
        }

        if mapping.nlq_method_idc.is_some() {
            mapping.nlq = Some(RpuDataNlq::parse(reader, header, &mapping)?);
        }

        Ok(mapping)
    }

    pub fn write(&self, writer: &mut BitstreamIoWriter, header: &RpuDataHeader) -> Result<()> {
        let coefficient_log2_denom_length = header.coefficient_log2_denom_length;

        let bl_bit_depth = (header.bl_bit_depth_minus8 + 8) as u32;

        writer.write_ue(&self.vdr_rpu_id)?;
        writer.write_ue(&self.mapping_color_space)?;
        writer.write_ue(&self.mapping_chroma_format_idc)?;

        for cmp in 0..NUM_COMPONENTS {
            let curve = &self.curves[cmp];
            writer.write_ue(&curve.num_pivots_minus2)?;

            for p in &curve.pivots {
                writer.write_n(p, bl_bit_depth)?;
            }
        }

        if header.rpu_format & 0x700 == 0 && !header.disable_residual_flag {
            if let Some(nlq_method_idc) = self.nlq_method_idc {
                writer.write_n(&(nlq_method_idc as u8), 3)?;
            }

            if let Some(nlq_pred_pivot_value) = &self.nlq_pred_pivot_value {
                for pv in nlq_pred_pivot_value {
                    writer.write_n(pv, bl_bit_depth)?;
                }
            }
        }

        writer.write_ue(&self.num_x_partitions_minus1)?;
        writer.write_ue(&self.num_y_partitions_minus1)?;

        for cmp in 0..NUM_COMPONENTS {
            let curve = &self.curves[cmp];
            let num_pieces = (curve.num_pivots_minus2 + 1) as usize;

            for i in 0..num_pieces {
                writer.write_ue(&(curve.mapping_idc as u64))?;

                // MAPPING_POLYNOMIAL
                if let Some(poly_curve) = &curve.polynomial {
                    writer.write_ue(&poly_curve.poly_order_minus1[i])?;

                    let poly_order_minus1 = poly_curve.poly_order_minus1[i];
                    if poly_order_minus1 == 0 {
                        writer.write(poly_curve.linear_interp_flag[i])?;
                    }

                    if poly_order_minus1 == 0 && poly_curve.linear_interp_flag[i] {
                        unimplemented!("write: Polynomial interpolation: please open an issue");

                        /*
                        if header.coefficient_data_type == 0 {
                            writer.write_ue(
                                self.pred_linear_interp_value_int[cmp_idx][pivot_idx],
                            );
                        }

                        writer.write_n(
                            &self.pred_linear_interp_value[cmp_idx][pivot_idx].to_be_bytes(),
                            coefficient_log2_denom_length,
                        );

                        if pivot_idx as u64 == header.num_pivots_minus2[cmp_idx] {
                            if header.coefficient_data_type == 0 {
                                writer.write_ue(
                                    self.pred_linear_interp_value_int[cmp_idx][pivot_idx + 1],
                                );
                            }

                            writer.write_n(
                                &self.pred_linear_interp_value[cmp_idx][pivot_idx + 1]
                                    .to_be_bytes(),
                                coefficient_log2_denom_length,
                            );
                        }
                        */
                    } else {
                        let poly_coef_count = poly_order_minus1 as usize + 1;

                        for j in 0..=poly_coef_count {
                            if header.coefficient_data_type == 0 {
                                writer.write_se(&poly_curve.poly_coef_int[i][j])?;
                            }

                            writer.write_n(
                                &poly_curve.poly_coef[i][j],
                                coefficient_log2_denom_length,
                            )?;
                        }
                    }
                } else if let Some(mmr_curve) = &curve.mmr {
                    // MAPPING_MMR
                    writer.write_n(&mmr_curve.mmr_order_minus1[i], 2)?;

                    if header.coefficient_data_type == 0 {
                        writer.write_se(&mmr_curve.mmr_constant_int[i])?;
                    }

                    writer.write_n(&mmr_curve.mmr_constant[i], coefficient_log2_denom_length)?;

                    for j in 0..mmr_curve.mmr_order_minus1[i] as usize + 1 {
                        for k in 0..MMR_MAX_COEFFS {
                            if header.coefficient_data_type == 0 {
                                writer.write_se(&mmr_curve.mmr_coef_int[i][j][k])?;
                            }

                            writer.write_n(
                                &mmr_curve.mmr_coef[i][j][k],
                                coefficient_log2_denom_length,
                            )?;
                        }
                    }
                } else {
                    bail!("Missing mapping method");
                }
            }
        }

        if let Some(nlq) = self.nlq.as_ref() {
            nlq.write(writer, header, self)?;
        }

        Ok(())
    }

    pub fn validate(&self, profile: u8) -> Result<()> {
        match profile {
            5 => {
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
                    self.nlq_pred_pivot_value.is_some(),
                    "profile 7: nlq_pred_pivot_value should be defined"
                );

                if let Some(nlq_pred_pivot_value) = self.nlq_pred_pivot_value {
                    ensure!(
                        nlq_pred_pivot_value.iter().sum::<u16>() == 1023,
                        "profile 7: nlq_pred_pivot_value elements should add up to the BL bit depth"
                    );
                }
            }
            8 => {
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

        ensure!(
            self.mapping_color_space == 0,
            "mapping_color_space should be 0"
        );
        ensure!(
            self.mapping_chroma_format_idc == 0,
            "mapping_chroma_format_idc should be 0"
        );

        Ok(())
    }

    pub fn set_empty_p81_mapping(&mut self) {
        self.curves.iter_mut().for_each(|curve| {
            curve.mapping_idc = DoviMappingMethod::Polynomial;
            curve.mmr = None;

            if let Some(poly_curve) = curve.polynomial.as_mut() {
                poly_curve.set_p81_params();
            } else {
                curve.polynomial = Some(DoviPolynomialCurve::p81_default());
            }
        });
    }

    pub fn get_enhancement_layer_type(&self) -> Option<DoviELType> {
        self.nlq.as_ref().map(|nlq| nlq.el_type())
    }
}

impl DoviPolynomialCurve {
    fn new(num_pieces: usize) -> Self {
        DoviPolynomialCurve {
            poly_order_minus1: Vec::with_capacity(num_pieces),
            linear_interp_flag: Vec::with_capacity(num_pieces),
            poly_coef_int: Vec::with_capacity(num_pieces),
            poly_coef: Vec::with_capacity(num_pieces),
        }
    }

    fn parse(&mut self, reader: &mut BsIoSliceReader, header: &RpuDataHeader) -> Result<()> {
        let coefficient_log2_denom_length = header.coefficient_log2_denom_length;

        let poly_order_minus1 = reader.get_ue()?;
        ensure!(poly_order_minus1 <= 1);

        self.poly_order_minus1.push(poly_order_minus1);

        let linear_interp_flag = if poly_order_minus1 == 0 {
            reader.get()?
        } else {
            false
        };
        self.linear_interp_flag.push(linear_interp_flag);

        if poly_order_minus1 == 0 && linear_interp_flag {
            // Linear interpolation
            unimplemented!("parse: Polynomial interpolation: please open an issue");

            /*if header.coefficient_data_type == 0 {
                self.pred_linear_interp_value_int[i] = reader.get_ue()?;
            }

            self.pred_linear_interp_value[i] =
                reader.get_n(coefficient_log2_denom_length)?;

            if pivot_idx as u64 == header.num_pivots_minus2[cmp] {
                if header.coefficient_data_type == 0 {
                    self.pred_linear_interp_value_int[cmp][pivot_idx + 1] =
                        reader.get_ue()?;
                }

                self.pred_linear_interp_value[cmp][pivot_idx + 1] =
                    reader.get_n(coefficient_log2_denom_length)?;
            }*/
        } else {
            let poly_coef_count = poly_order_minus1 as usize + 2;
            let mut poly_coef_int = Vec::with_capacity(poly_coef_count);
            let mut poly_coef = Vec::with_capacity(poly_coef_count);

            for _j in 0..poly_coef_count {
                if header.coefficient_data_type == 0 {
                    poly_coef_int.push(reader.get_se()?);
                }

                poly_coef.push(reader.get_n(coefficient_log2_denom_length)?);
            }

            self.poly_coef_int.push(poly_coef_int);
            self.poly_coef.push(poly_coef);
        }

        Ok(())
    }

    pub fn p81_default() -> Self {
        let mut poly_curve = Self::new(1);
        poly_curve.set_p81_params();

        poly_curve
    }

    pub fn set_p81_params(&mut self) {
        self.poly_order_minus1.clear();
        self.poly_order_minus1.push(0);

        self.linear_interp_flag.clear();
        self.linear_interp_flag.push(false);

        self.poly_coef_int.clear();
        self.poly_coef_int.push(vec![0, 1]);

        self.poly_coef.clear();
        self.poly_coef.push(vec![0, 0]);
    }
}

impl DoviMMRCurve {
    fn new(num_pieces: usize) -> Self {
        DoviMMRCurve {
            mmr_order_minus1: Vec::with_capacity(num_pieces),
            mmr_constant_int: Vec::with_capacity(num_pieces),
            mmr_constant: Vec::with_capacity(num_pieces),
            mmr_coef_int: Vec::with_capacity(num_pieces),
            mmr_coef: Vec::with_capacity(num_pieces),
        }
    }

    fn parse(&mut self, reader: &mut BsIoSliceReader, header: &RpuDataHeader) -> Result<()> {
        let coefficient_log2_denom_length = header.coefficient_log2_denom_length;

        let mmr_order_minus1 = reader.get_n(2)?;
        ensure!(mmr_order_minus1 <= 2);

        self.mmr_order_minus1.push(mmr_order_minus1);

        let mmr_orders_count = mmr_order_minus1 as usize + 1;

        if header.coefficient_data_type == 0 {
            self.mmr_constant_int.push(reader.get_se()?);
        }
        self.mmr_constant
            .push(reader.get_n(coefficient_log2_denom_length)?);

        let mut mmr_coef_int = Vec::with_capacity(mmr_orders_count);
        let mut mmr_coef = Vec::with_capacity(mmr_orders_count);

        for _j in 0..mmr_orders_count {
            let mut mmr_coef_int2 = Vec::with_capacity(MMR_MAX_COEFFS);
            let mut mmr_coef2 = Vec::with_capacity(MMR_MAX_COEFFS);

            for _k in 0..MMR_MAX_COEFFS {
                if header.coefficient_data_type == 0 {
                    mmr_coef_int2.push(reader.get_se()?);
                }

                mmr_coef2.push(reader.get_n(coefficient_log2_denom_length)?);
            }

            mmr_coef_int.push(mmr_coef_int2);
            mmr_coef.push(mmr_coef2);
        }

        self.mmr_coef_int.push(mmr_coef_int);
        self.mmr_coef.push(mmr_coef);

        Ok(())
    }
}

impl Default for DoviMappingMethod {
    fn default() -> Self {
        Self::Invalid
    }
}

impl From<u64> for DoviMappingMethod {
    fn from(value: u64) -> Self {
        match value {
            0 => Self::Polynomial,
            1 => Self::MMR,
            _ => unreachable!(),
        }
    }
}

impl From<u8> for DoviNlqMethod {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::LinearDeadzone,
            _ => unreachable!(),
        }
    }
}
