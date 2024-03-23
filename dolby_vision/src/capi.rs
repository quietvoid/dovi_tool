#![deny(missing_docs)]

use libc::{c_char, size_t};
use std::{
    ffi::{CStr, CString},
    path::PathBuf,
    ptr::{null, null_mut},
    slice,
};

use crate::rpu::{dovi_rpu::DoviRpu, utils::parse_rpu_file, ConversionMode};

use super::c_structs::*;

/// # Safety
/// The pointer to the data must be valid.
///
/// Parse a Dolby Vision RPU from unescaped byte buffer.
/// Adds an error if the parsing fails.
#[no_mangle]
pub unsafe extern "C" fn dovi_parse_rpu(buf: *const u8, len: size_t) -> *mut RpuOpaque {
    assert!(!buf.is_null());

    let data = slice::from_raw_parts(buf, len);
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

    let data = slice::from_raw_parts(buf, len);
    let res = DoviRpu::parse_unspec62_nalu(data);

    Box::into_raw(Box::new(RpuOpaque::from(res)))
}

/// # Safety
/// The pointer to the opaque struct must be valid.
/// Avoid using on opaque pointers obtained through `dovi_parse_rpu_bin_file`.
///
/// Free the RpuOpaque
#[no_mangle]
pub unsafe extern "C" fn dovi_rpu_free(ptr: *mut RpuOpaque) {
    if !ptr.is_null() {
        drop(Box::from_raw(ptr));
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
        Some(s) => s.as_ptr(),
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
/// The struct pointer must be valid.
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
                opaque.error =
                    Some(CString::new(format!("Failed writing byte buffer: {e}")).unwrap());
                null_mut()
            }
        }
    } else {
        null_mut()
    }
}

/// # Safety
/// The struct pointer must be valid.
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
                opaque.error =
                    Some(CString::new(format!("Failed writing byte buffer: {e}")).unwrap());
                null_mut()
            }
        }
    } else {
        null_mut()
    }
}

/// # Safety
/// The struct pointer must be valid.
/// The mode must be between 0 and 4.
///
/// Converts the RPU to be compatible with a different Dolby Vision profile.
/// Possible modes:
///     - 0: Don't modify the RPU
///     - 1: Converts the RPU to be MEL compatible
///     - 2: Converts the RPU to be profile 8.1 compatible. Both luma and chroma mapping curves are set to no-op.
///          This mode handles source profiles 5, 7 and 8.
///     - 3: Converts to static profile 8.4
///     - 4: Converts to profile 8.1 preserving luma and chroma mapping. Old mode 2 behaviour.
///
/// If an error occurs, it is logged to RpuOpaque.error.
/// Returns 0 if successful, -1 otherwise.
#[no_mangle]
pub unsafe extern "C" fn dovi_convert_rpu_with_mode(ptr: *mut RpuOpaque, mode: u8) -> i32 {
    if ptr.is_null() {
        return -1;
    }

    let opaque = &mut *ptr;

    if let Some(rpu) = &mut opaque.rpu {
        let mode = ConversionMode::from(mode);

        match rpu.convert_with_mode(mode) {
            Ok(_) => 0,
            Err(e) => {
                opaque.error =
                    Some(CString::new(format!("Failed converting with mode {mode}: {e}")).unwrap());
                -1
            }
        }
    } else {
        -1
    }
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

        if let Some(el_type) = rpu.el_type.as_ref() {
            header.el_type = el_type.as_cstr().as_ptr();
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
        drop(Box::from_raw(ptr as *mut RpuDataHeader));
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

/// # Safety
/// The pointer to the file path must be valid.
///
/// Parses an existing RPU binary file.
///
/// Returns the heap allocated `DoviRpuList` as a pointer.
/// The returned pointer may be null, or the list could be empty if an error occurred.
#[no_mangle]
pub unsafe extern "C" fn dovi_parse_rpu_bin_file(path: *const c_char) -> *const RpuOpaqueList {
    if !path.is_null() {
        let mut rpu_list = RpuOpaqueList {
            list: null(),
            len: 0,
            error: null(),
        };
        let mut error = None;

        if let Ok(str) = CStr::from_ptr(path).to_str() {
            let path = PathBuf::from(str);

            if path.is_file() {
                match parse_rpu_file(path) {
                    Ok(rpus) => {
                        rpu_list.len = rpus.len();

                        let opaque_list: Vec<*mut RpuOpaque> = rpus
                            .into_iter()
                            .map(|rpu| Box::into_raw(Box::new(RpuOpaque::new(Some(rpu), None))))
                            .collect();

                        rpu_list.list =
                            Box::into_raw(opaque_list.into_boxed_slice()) as *const *mut RpuOpaque;
                    }
                    Err(e) => {
                        error = Some(format!("parse_rpu_bin_file: Errored while parsing: {e}"))
                    }
                }
            } else {
                error = Some("parse_rpu_bin_file: Input file does not exist".to_string());
            }
        } else {
            error =
                Some("parse_rpu_bin_file: Failed parsing the input path as a string".to_string());
        }

        if let Some(err) = error {
            rpu_list.error = CString::new(err).unwrap().into_raw();
        }

        return Box::into_raw(Box::new(rpu_list));
    }

    null()
}

/// # Safety
/// The pointer to the struct must be valid.
///
/// Frees the memory used by the DoviRpuOpaqueList struct.
#[no_mangle]
pub unsafe extern "C" fn dovi_rpu_list_free(ptr: *const RpuOpaqueList) {
    if !ptr.is_null() {
        let rpu_opaque_list = Box::from_raw(ptr as *mut RpuOpaqueList);
        rpu_opaque_list.free();
    }
}

/// # Safety
/// The struct pointer must be valid.
///
/// Sets the L5 metadata active area offsets.
/// If there is no L5 block present, it is created with the offsets.
#[no_mangle]
pub unsafe extern "C" fn dovi_rpu_set_active_area_offsets(
    ptr: *mut RpuOpaque,
    left: u16,
    right: u16,
    top: u16,
    bottom: u16,
) -> i32 {
    if ptr.is_null() {
        return -1;
    }

    let opaque = &mut *ptr;

    if let Some(rpu) = &mut opaque.rpu {
        match rpu.set_active_area_offsets(left, right, top, bottom) {
            Ok(_) => 0,
            Err(e) => {
                opaque.error =
                    Some(CString::new(format!("Failed editing active area offsets: {e}")).unwrap());
                -1
            }
        }
    } else {
        -1
    }
}

/// # Safety
/// The struct pointer must be valid.
///
/// Converts the existing reshaping/mapping to become no-op.
#[no_mangle]
pub unsafe extern "C" fn dovi_rpu_remove_mapping(ptr: *mut RpuOpaque) -> i32 {
    if ptr.is_null() {
        return -1;
    }

    let opaque = &mut *ptr;

    if let Some(rpu) = &mut opaque.rpu {
        rpu.remove_mapping();

        0
    } else {
        -1
    }
}

/// # Safety
/// The struct pointer must be valid.
///
/// Writes the encoded RPU as `itu_t_t35_payload_bytes` for AV1 ITU-T T.35 metadata OBU
/// If an error occurs in the writing, it is logged to RpuOpaque.error
#[no_mangle]
pub unsafe extern "C" fn dovi_write_av1_rpu_metadata_obu_t35_payload(
    ptr: *mut RpuOpaque,
) -> *const Data {
    if ptr.is_null() {
        return null_mut();
    }

    let opaque = &mut *ptr;

    if let Some(rpu) = &opaque.rpu {
        match rpu.write_av1_rpu_metadata_obu_t35_payload() {
            Ok(buf) => Box::into_raw(Box::new(Data::from(buf))),
            Err(e) => {
                opaque.error =
                    Some(CString::new(format!("Failed writing byte buffer: {e}")).unwrap());
                null_mut()
            }
        }
    } else {
        null_mut()
    }
}

/// # Safety
/// The struct pointer must be valid.
///
/// Writes the encoded RPU a complete AV1 `metadata_itut_t35()` OBU
/// If an error occurs in the writing, it is logged to RpuOpaque.error
#[no_mangle]
pub unsafe extern "C" fn dovi_write_av1_rpu_metadata_obu_t35_complete(
    ptr: *mut RpuOpaque,
) -> *const Data {
    if ptr.is_null() {
        return null_mut();
    }

    let opaque = &mut *ptr;

    if let Some(rpu) = &opaque.rpu {
        match rpu.write_av1_rpu_metadata_obu_t35_complete() {
            Ok(buf) => Box::into_raw(Box::new(Data::from(buf))),
            Err(e) => {
                opaque.error =
                    Some(CString::new(format!("Failed writing byte buffer: {e}")).unwrap());
                null_mut()
            }
        }
    } else {
        null_mut()
    }
}
