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

impl RpuOpaque {
    pub(crate) fn new(rpu: Option<DoviRpu>, error: Option<CString>) -> Self {
        Self { rpu, error }
    }
}

impl From<Result<DoviRpu, anyhow::Error>> for RpuOpaque {
    fn from(res: Result<DoviRpu, anyhow::Error>) -> Self {
        match res {
            Ok(rpu) => Self::new(Some(rpu), None),
            Err(e) => Self::new(
                None,
                Some(CString::new(format!("Failed parsing RPU: {e}")).unwrap()),
            ),
        }
    }
}

impl Freeable for RpuOpaqueList {
    unsafe fn free(&self) {
        unsafe {
            let list = Vec::from_raw_parts(self.list as *mut *mut RpuOpaque, self.len, self.len);
            for ptr in list {
                drop(Box::from_raw(ptr));
            }

            if !self.error.is_null() {
                drop(CString::from_raw(self.error as *mut c_char));
            }
        }
    }
}
