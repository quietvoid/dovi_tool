use std::ptr::null_mut;

use crate::rpu::rpu_data_mapping::{
    DoviMMRCurve, DoviPolynomialCurve, DoviReshapingCurve, RpuDataMapping as RuRpuDataMapping,
};

use super::{NUM_COMPONENTS, RpuDataNlq, buffers::*};

/// C struct for rpu_data_mapping()
#[repr(C)]
pub struct RpuDataMapping {
    vdr_rpu_id: u64,
    mapping_color_space: u64,
    mapping_chroma_format_idc: u64,
    num_x_partitions_minus1: u64,
    num_y_partitions_minus1: u64,

    curves: [ReshapingCurve; NUM_COMPONENTS],

    /// Set to -1 to represent Option::None
    nlq_method_idc: i32,
    /// Set to -1 to represent Option::None
    nlq_num_pivots_minus2: i32,
    /// Length of zero when not present. Only present in profile 4 and 7.
    nlq_pred_pivot_value: U16Data,
    /// Pointer to `RpuDataNlq` struct, null if not dual layer profile
    nlq: *const RpuDataNlq,
}

#[repr(C)]
pub struct ReshapingCurve {
    /// [2, 9]
    pub num_pivots_minus2: u64,
    pub pivots: U16Data,

    /// Consistent for a component
    /// Luma (component 0): Polynomial = 0
    /// Chroma (components 1 and 2): MMR = 1
    pub mapping_idc: u8,

    /// mapping_idc = 0, null pointer otherwise
    pub polynomial: *const PolynomialCurve,

    /// mapping_idc = 1, null pointer otherwise
    pub mmr: *const MMRCurve,
}

#[repr(C)]
pub struct PolynomialCurve {
    poly_order_minus1: U64Data,
    linear_interp_flag: Data,
    poly_coef_int: I64Data2D,
    poly_coef: U64Data2D,
}

#[repr(C)]
pub struct MMRCurve {
    mmr_order_minus1: Data,
    mmr_constant_int: I64Data,
    mmr_constant: U64Data,
    mmr_coef_int: I64Data3D,
    mmr_coef: U64Data3D,
}

impl RpuDataMapping {
    /// # Safety
    /// The buffer pointers should be valid.
    pub unsafe fn free(&self) {
        unsafe {
            self.curves.iter().for_each(|curve| curve.free());
            self.nlq_pred_pivot_value.free();

            if !self.nlq.is_null() {
                drop(Box::from_raw(self.nlq as *mut RpuDataNlq))
            }
        }
    }
}

impl From<&RuRpuDataMapping> for RpuDataMapping {
    fn from(mapping: &RuRpuDataMapping) -> Self {
        let curves = [
            ReshapingCurve::from(&mapping.curves[0]),
            ReshapingCurve::from(&mapping.curves[1]),
            ReshapingCurve::from(&mapping.curves[2]),
        ];

        Self {
            vdr_rpu_id: mapping.vdr_rpu_id,
            mapping_color_space: mapping.mapping_color_space,
            mapping_chroma_format_idc: mapping.mapping_chroma_format_idc,
            num_x_partitions_minus1: mapping.num_x_partitions_minus1,
            num_y_partitions_minus1: mapping.num_y_partitions_minus1,
            curves,
            nlq_method_idc: mapping
                .nlq_method_idc
                .as_ref()
                .map_or(-1, |e| (*e as u8) as i32),
            nlq_num_pivots_minus2: mapping.nlq_num_pivots_minus2.map_or(-1, |e| e as i32),
            nlq_pred_pivot_value: U16Data::from(mapping.nlq_pred_pivot_value),
            nlq: mapping.nlq.as_ref().map_or(null_mut(), |nlq| {
                Box::into_raw(Box::new(RpuDataNlq::from(nlq)))
            }),
        }
    }
}

impl ReshapingCurve {
    /// # Safety
    /// The buffer pointers should be valid.
    pub unsafe fn free(&self) {
        unsafe {
            self.pivots.free();

            if !self.polynomial.is_null() {
                let poly_curve = Box::from_raw(self.polynomial as *mut PolynomialCurve);
                poly_curve.free();
            } else if !self.mmr.is_null() {
                let mmr_curve = Box::from_raw(self.mmr as *mut MMRCurve);
                mmr_curve.free();
            }
        }
    }
}

impl PolynomialCurve {
    /// # Safety
    /// The buffer pointers should be valid.
    pub unsafe fn free(&self) {
        unsafe {
            self.poly_order_minus1.free();
            self.linear_interp_flag.free();
            self.poly_coef_int.free();
            self.poly_coef.free();
        }
    }
}

impl MMRCurve {
    /// # Safety
    /// The buffer pointers should be valid.
    pub unsafe fn free(&self) {
        unsafe {
            self.mmr_order_minus1.free();
            self.mmr_constant_int.free();
            self.mmr_constant.free();
            self.mmr_coef_int.free();
            self.mmr_coef.free();
        }
    }
}

impl From<&DoviReshapingCurve> for ReshapingCurve {
    fn from(curve: &DoviReshapingCurve) -> Self {
        Self {
            num_pivots_minus2: curve.num_pivots_minus2,
            pivots: U16Data::from(curve.pivots.clone()),
            mapping_idc: curve.mapping_idc as u8,
            polynomial: curve.polynomial.as_ref().map_or(null_mut(), |poly_curve| {
                Box::into_raw(Box::new(PolynomialCurve::from(poly_curve)))
            }),
            mmr: curve.mmr.as_ref().map_or(null_mut(), |mmr_curve| {
                Box::into_raw(Box::new(MMRCurve::from(mmr_curve)))
            }),
        }
    }
}

impl From<&DoviPolynomialCurve> for PolynomialCurve {
    fn from(poly_curve: &DoviPolynomialCurve) -> Self {
        PolynomialCurve {
            poly_order_minus1: U64Data::from(poly_curve.poly_order_minus1.clone()),
            linear_interp_flag: Data::from(poly_curve.linear_interp_flag.clone()),
            poly_coef_int: I64Data2D::from(poly_curve.poly_coef_int.clone()),
            poly_coef: U64Data2D::from(poly_curve.poly_coef.clone()),
        }
    }
}

impl From<&DoviMMRCurve> for MMRCurve {
    fn from(mmr_curve: &DoviMMRCurve) -> Self {
        MMRCurve {
            mmr_order_minus1: Data::from(mmr_curve.mmr_order_minus1.clone()),
            mmr_constant_int: I64Data::from(mmr_curve.mmr_constant_int.clone()),
            mmr_constant: U64Data::from(mmr_curve.mmr_constant.clone()),
            mmr_coef_int: I64Data3D::from(mmr_curve.mmr_coef_int.clone()),
            mmr_coef: U64Data3D::from(mmr_curve.mmr_coef.clone()),
        }
    }
}
