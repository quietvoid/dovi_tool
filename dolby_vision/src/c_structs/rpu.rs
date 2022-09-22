use std::ffi::CString;

use libc::{c_char, size_t};

use crate::rpu::dovi_rpu::DoviRpu;

use super::Freeable;

/// Opaque Dolby Vision RPU.
///
/// Use dovi_rpu_free to free.
/// It should be freed regardless of whether or not an error occurred.
pub struct RpuOpaque {
    /// Optional parsed RPU, present when parsing is successful.
    pub rpu: Option<DoviRpu>,
    /// Error String of the parsing, in cases of failure.
    pub error: Option<CString>,
}

/// Heap allocated list of valid RPU pointers
#[repr(C)]
pub struct RpuOpaqueList {
    pub list: *const *mut RpuOpaque,
    pub len: size_t,

    pub error: *const c_char,
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
                error: Some(CString::new(format!("Failed parsing RPU: {}", e)).unwrap()),
            },
        }
    }
}

impl Freeable for RpuOpaqueList {
    unsafe fn free(&self) {
        let list = Vec::from_raw_parts(
            self.list as *mut *mut RpuOpaque,
            self.len as usize,
            self.len as usize,
        );
        for ptr in list {
            drop(Box::from_raw(ptr));
        }

        if !self.error.is_null() {
            drop(CString::from_raw(self.error as *mut c_char));
        }
    }
}
