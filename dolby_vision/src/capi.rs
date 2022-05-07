#![deny(missing_docs)]

use libc::{c_char, size_t};
use std::{
    ptr::{null, null_mut},
    slice,
};

use crate::rpu::dovi_rpu::DoviRpu;

use super::c_structs::*;

/// Opaque Dolby Vision RPU.
///
/// Use dovi_rpu_free to free.
pub struct RpuOpaque {
    /// Optional parsed RPU, present when parsing is successful.
    rpu: Option<DoviRpu>,
    // Error String of the parsing, in cases of failure.
    error: Option<String>,
}

/// # Safety
/// The pointer to the data must be valid.
///
/// Parse a Dolby Vision RPU from unescaped byte buffer.
/// Adds an error if the parsing fails.
#[no_mangle]
pub unsafe extern "C" fn dovi_parse_rpu(buf: *const u8, len: size_t) -> *mut RpuOpaque {
    assert!(!buf.is_null());

    let data = slice::from_raw_parts(buf, len as usize);
    let res = DoviRpu::parse_rpu(data);

    Box::into_raw(Box::new(RpuOpaque::from(res)))
}

/// # Safety
/// The pointer to the data must be valid.
///
/// Parse a Dolby Vision from a (possibly) escaped HEVC UNSPEC 62 NAL unit byte buffer.
/// Adds an error if the parsing fails.
#[no_mangle]
pub unsafe extern "C" fn dovi_parse_unspec62_nalu(buf: *const u8, len: size_t) -> *mut RpuOpaque {
    assert!(!buf.is_null());

    let data = slice::from_raw_parts(buf, len as usize);
    let res = DoviRpu::parse_unspec62_nalu(data);

    Box::into_raw(Box::new(RpuOpaque::from(res)))
}

/// # Safety
/// The pointer to the opaque struct must be valid.
///
/// Free the RpuOpaque
#[no_mangle]
pub unsafe extern "C" fn dovi_rpu_free(ptr: *mut RpuOpaque) {
    if !ptr.is_null() {
        Box::from_raw(ptr);
    }
}

/// # Safety
/// The pointer to the opaque struct must be valid.
///
/// Get the last logged error for the RpuOpaque operations.
///
/// On invalid parsing, an error is added.
/// The user should manually verify if there is an error, as the parsing does not return an error code.
#[no_mangle]
pub unsafe extern "C" fn dovi_rpu_get_error(ptr: *const RpuOpaque) -> *const c_char {
    if ptr.is_null() {
        return null();
    }

    let opaque = &*ptr;

    match &opaque.error {
        Some(s) => s.as_ptr() as *const c_char,
        None => null(),
    }
}

/// # Safety
/// The data pointer should exist, and be allocated by Rust.
///
/// Free a Data buffer
#[no_mangle]
pub unsafe extern "C" fn dovi_data_free(data: *const Data) {
    if !data.is_null() {
        let data = Box::from_raw(data as *mut Data);
        data.free();
    }
}

/// # Safety
/// The struct pointer should be valid.
///
/// Writes the encoded RPU as a byte buffer.
/// If an error occurs in the writing, it is logged to RpuOpaque.error
#[no_mangle]
pub unsafe extern "C" fn dovi_write_rpu(ptr: *mut RpuOpaque) -> *const Data {
    if ptr.is_null() {
        return null_mut();
    }

    let opaque = &mut *ptr;

    if let Some(rpu) = &opaque.rpu {
        match rpu.write_rpu() {
            Ok(buf) => Box::into_raw(Box::new(Data::from(buf))),
            Err(e) => {
                opaque.error = Some(format!("Failed writing byte buffer: {}", e));
                null_mut()
            }
        }
    } else {
        null_mut()
    }
}

/// # Safety
/// The struct pointer should be valid.
///
/// Writes the encoded RPU, escapes the bytes for HEVC and prepends the buffer with 0x7C01.
/// If an error occurs in the writing, it is logged to RpuOpaque.error
#[no_mangle]
pub unsafe extern "C" fn dovi_write_unspec62_nalu(ptr: *mut RpuOpaque) -> *const Data {
    if ptr.is_null() {
        return null_mut();
    }

    let opaque = &mut *ptr;

    if let Some(rpu) = &opaque.rpu {
        match rpu.write_hevc_unspec62_nalu() {
            Ok(buf) => Box::into_raw(Box::new(Data::from(buf))),
            Err(e) => {
                opaque.error = Some(format!("Failed writing byte buffer: {}", e));
                null_mut()
            }
        }
    } else {
        null_mut()
    }
}

/// # Safety
/// The struct pointer should be valid.
/// The mode should be between 0 and 3.
///
/// Converts the RPU to be compatible with a different Dolby Vision profile.
/// Possible modes:
///     0: Don't modify the RPU
///     1: Converts the RPU to be MEL compatible
///     2: Converts the RPU to be profile 8.1 compatible
///     3: Converts profile 5 to 8
///
/// If an error occurs, it is logged to RpuOpaque.error.
/// Returns 0 if successful, -1 otherwise.
#[no_mangle]
pub unsafe extern "C" fn dovi_convert_rpu_with_mode(ptr: *mut RpuOpaque, mode: u8) -> i32 {
    if ptr.is_null() {
        return -1;
    }

    let opaque = &mut *ptr;

    let ret = if let Some(rpu) = &mut opaque.rpu {
        match rpu.convert_with_mode(mode) {
            Ok(_) => 0,
            Err(e) => {
                opaque.error = Some(format!("Failed converting with mode {}: {}", mode, e));
                -1
            }
        }
    } else {
        -1
    };

    ret
}

/// # Safety
/// The pointer to the opaque struct must be valid.
///
/// Get the DoVi RPU header struct.
#[no_mangle]
pub unsafe extern "C" fn dovi_rpu_get_header(ptr: *const RpuOpaque) -> *const RpuDataHeader {
    if ptr.is_null() {
        return null_mut();
    }

    let opaque = &*ptr;

    if let Some(rpu) = &opaque.rpu {
        let mut header = RpuDataHeader::from(&rpu.header);

        if let Some(subprofile) = &rpu.subprofile {
            header.subprofile = subprofile.as_ptr() as *const c_char
        }

        Box::into_raw(Box::new(header))
    } else {
        null_mut()
    }
}

/// # Safety
/// The pointer to the struct must be valid.
///
/// Frees the memory used by the RPU header.
#[no_mangle]
pub unsafe extern "C" fn dovi_rpu_free_header(ptr: *const RpuDataHeader) {
    if !ptr.is_null() {
        let header = Box::from_raw(ptr as *mut RpuDataHeader);
        header.free();
    }
}

/// # Safety
/// The pointer to the opaque struct must be valid.
///
/// Get the DoVi RpuDataMapping struct.
#[no_mangle]
pub unsafe extern "C" fn dovi_rpu_get_data_mapping(ptr: *const RpuOpaque) -> *const RpuDataMapping {
    if ptr.is_null() {
        return null_mut();
    }

    let opaque = &*ptr;

    if let Some(rpu) = &opaque.rpu {
        if let Some(rpu_data_mapping) = &rpu.rpu_data_mapping {
            Box::into_raw(Box::new(RpuDataMapping::from(rpu_data_mapping)))
        } else {
            null_mut()
        }
    } else {
        null_mut()
    }
}

/// # Safety
/// The pointer to the struct must be valid.
///
/// Frees the memory used by the RpuDataMapping.
#[no_mangle]
pub unsafe extern "C" fn dovi_rpu_free_data_mapping(ptr: *const RpuDataMapping) {
    if !ptr.is_null() {
        let rpu_data_mapping = Box::from_raw(ptr as *mut RpuDataMapping);
        rpu_data_mapping.free();
    }
}

/// # Safety
/// The pointer to the opaque struct must be valid.
///
/// Get the DoVi RpuDataNlq struct.
#[no_mangle]
pub unsafe extern "C" fn dovi_rpu_get_data_nlq(ptr: *const RpuOpaque) -> *const RpuDataNlq {
    if ptr.is_null() {
        return null_mut();
    }

    let opaque = &*ptr;

    if let Some(rpu) = &opaque.rpu {
        if let Some(rpu_data_nlq) = &rpu.rpu_data_nlq {
            Box::into_raw(Box::new(RpuDataNlq::from(rpu_data_nlq)))
        } else {
            null_mut()
        }
    } else {
        null_mut()
    }
}

/// # Safety
/// The pointer to the struct must be valid.
///
/// Frees the memory used by the RpuDataNlq struct.
#[no_mangle]
pub unsafe extern "C" fn dovi_rpu_free_data_nlq(ptr: *const RpuDataNlq) {
    if !ptr.is_null() {
        let rpu_data_nlq = Box::from_raw(ptr as *mut RpuDataNlq);
        rpu_data_nlq.free();
    }
}

/// # Safety
/// The pointer to the opaque struct must be valid.
///
/// Get the DoVi VdrDmData struct.
#[no_mangle]
pub unsafe extern "C" fn dovi_rpu_get_vdr_dm_data(ptr: *const RpuOpaque) -> *const VdrDmData {
    if ptr.is_null() {
        return null_mut();
    }

    let opaque = &*ptr;

    if let Some(rpu) = &opaque.rpu {
        if let Some(vdr_dm_data) = &rpu.vdr_dm_data {
            Box::into_raw(Box::new(VdrDmData::from(vdr_dm_data)))
        } else {
            null_mut()
        }
    } else {
        null_mut()
    }
}

/// # Safety
/// The pointer to the struct must be valid.
///
/// Frees the memory used by the VdrDmData struct.
#[no_mangle]
pub unsafe extern "C" fn dovi_rpu_free_vdr_dm_data(ptr: *const VdrDmData) {
    if !ptr.is_null() {
        let vdr_dm_data = Box::from_raw(ptr as *mut VdrDmData);
        vdr_dm_data.free();
    }
}

impl From<Result<DoviRpu, anyhow::Error>> for RpuOpaque {
    fn from(res: Result<DoviRpu, anyhow::Error>) -> Self {
        match res {
            Ok(parsed_rpu) => Self {
                rpu: Some(parsed_rpu),
                error: None,
            },
            Err(e) => Self {
                rpu: None,
                error: Some(format!("Failed parsing RPU: {}", e)),
            },
        }
    }
}
