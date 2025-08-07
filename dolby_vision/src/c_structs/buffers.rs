use std::ptr::null;

use libc::size_t;
use tinyvec::ArrayVec;

pub trait Freeable {
    /// # Safety
    /// The pointers should all be valid.
    unsafe fn free(&self);
}

/// Struct representing a data buffer
#[repr(C)]
pub struct Data {
    /// Pointer to the data buffer
    pub data: *const u8,
    /// Data buffer size
    pub len: size_t,
}

/// Struct representing a data buffer
#[repr(C)]
pub struct U16Data {
    /// Pointer to the data buffer. Can be null if length is zero.
    pub data: *const u16,
    /// Data buffer size
    pub len: size_t,
}

/// Struct representing a data buffer
#[repr(C)]
pub struct U64Data {
    /// Pointer to the data buffer. Can be null if length is zero.
    pub data: *const u64,
    /// Data buffer size
    pub len: size_t,
}

/// Struct representing a data buffer
#[repr(C)]
pub struct I64Data {
    /// Pointer to the data buffer
    pub data: *const i64,
    /// Data buffer size
    pub len: size_t,
}

/// Struct representing a 2D data buffer
#[repr(C)]
pub struct U64Data2D {
    /// Pointer to the list of Data structs
    pub list: *const *const U64Data,
    /// List length
    pub len: size_t,
}

/// Struct representing a 2D data buffer
#[repr(C)]
pub struct I64Data2D {
    /// Pointer to the list of Data structs
    pub list: *const *const I64Data,
    /// List length
    pub len: size_t,
}

/// Struct representing a 3D data buffer
#[repr(C)]
pub struct U64Data3D {
    /// Pointer to the list of U64Data2D structs
    pub list: *const *const U64Data2D,
    /// List length
    pub len: size_t,
}

/// Struct representing a 3D data buffer
#[repr(C)]
pub struct I64Data3D {
    /// Pointer to the list of I64Data2D structs
    pub list: *const *const I64Data2D,
    /// List length
    pub len: size_t,
}

impl From<Vec<u8>> for Data {
    fn from(buf: Vec<u8>) -> Self {
        Data {
            len: buf.len(),
            data: Box::into_raw(buf.into_boxed_slice()) as *const u8,
        }
    }
}

impl From<Vec<bool>> for Data {
    fn from(buf: Vec<bool>) -> Self {
        let res: Vec<u8> = buf.into_iter().map(|e| e as u8).collect();

        Data {
            len: res.len(),
            data: Box::into_raw(res.into_boxed_slice()) as *const u8,
        }
    }
}

impl From<Vec<u16>> for U16Data {
    fn from(buf: Vec<u16>) -> Self {
        U16Data {
            len: buf.len(),
            data: Box::into_raw(buf.into_boxed_slice()) as *const u16,
        }
    }
}

impl<const N: usize> From<[bool; N]> for Data {
    fn from(array: [bool; N]) -> Self {
        let res: [u8; N] = array.map(|e| e as u8);

        Data {
            len: array.len(),
            data: Box::into_raw(Box::new(res)) as *const u8,
        }
    }
}

impl From<Vec<u64>> for U64Data {
    fn from(buf: Vec<u64>) -> Self {
        U64Data {
            len: buf.len(),
            data: Box::into_raw(buf.into_boxed_slice()) as *const u64,
        }
    }
}

impl<const N: usize> From<ArrayVec<[u64; N]>> for U64Data {
    fn from(buf: ArrayVec<[u64; N]>) -> Self {
        U64Data {
            len: buf.len(),
            data: Box::into_raw(buf.to_vec().into_boxed_slice()) as *const u64,
        }
    }
}

impl From<Vec<i64>> for I64Data {
    fn from(buf: Vec<i64>) -> Self {
        I64Data {
            len: buf.len(),
            data: Box::into_raw(buf.into_boxed_slice()) as *const i64,
        }
    }
}

impl<const N: usize> From<ArrayVec<[i64; N]>> for I64Data {
    fn from(buf: ArrayVec<[i64; N]>) -> Self {
        I64Data {
            len: buf.len(),
            data: Box::into_raw(buf.to_vec().into_boxed_slice()) as *const i64,
        }
    }
}

impl<const N: usize> From<[u16; N]> for U16Data {
    fn from(array: [u16; N]) -> Self {
        U16Data {
            len: array.len(),
            data: Box::into_raw(Box::new(array)) as *const u16,
        }
    }
}

impl<const N: usize> From<[u64; N]> for U64Data {
    fn from(array: [u64; N]) -> Self {
        U64Data {
            len: array.len(),
            data: Box::into_raw(Box::new(array)) as *const u64,
        }
    }
}

impl<const N: usize> From<Vec<ArrayVec<[u64; N]>>> for U64Data2D {
    fn from(buf_2d: Vec<ArrayVec<[u64; N]>>) -> Self {
        let list: Vec<*const U64Data> = buf_2d
            .into_iter()
            .map(|buf| Box::into_raw(Box::new(U64Data::from(buf))) as *const U64Data)
            .collect();

        U64Data2D {
            len: list.len(),
            list: Box::into_raw(list.into_boxed_slice()) as *const *const U64Data,
        }
    }
}

impl<const N: usize, const N2: usize> From<ArrayVec<[ArrayVec<[u64; N2]>; N]>> for U64Data2D {
    fn from(buf_2d: ArrayVec<[ArrayVec<[u64; N2]>; N]>) -> Self {
        let list: Vec<*const U64Data> = buf_2d
            .into_iter()
            .map(|buf| Box::into_raw(Box::new(U64Data::from(buf))) as *const U64Data)
            .collect();

        U64Data2D {
            len: list.len(),
            list: Box::into_raw(list.into_boxed_slice()) as *const *const U64Data,
        }
    }
}

impl<const N: usize> From<Vec<ArrayVec<[i64; N]>>> for I64Data2D {
    fn from(buf_2d: Vec<ArrayVec<[i64; N]>>) -> Self {
        let list: Vec<*const I64Data> = buf_2d
            .into_iter()
            .map(|buf| Box::into_raw(Box::new(I64Data::from(buf))) as *const I64Data)
            .collect();

        I64Data2D {
            len: list.len(),
            list: Box::into_raw(list.into_boxed_slice()) as *const *const I64Data,
        }
    }
}

impl<const N: usize, const N2: usize> From<ArrayVec<[ArrayVec<[i64; N2]>; N]>> for I64Data2D {
    fn from(buf_2d: ArrayVec<[ArrayVec<[i64; N2]>; N]>) -> Self {
        let list: Vec<*const I64Data> = buf_2d
            .into_iter()
            .map(|buf| Box::into_raw(Box::new(I64Data::from(buf))) as *const I64Data)
            .collect();

        I64Data2D {
            len: list.len(),
            list: Box::into_raw(list.into_boxed_slice()) as *const *const I64Data,
        }
    }
}

impl<const N: usize, const N2: usize> From<Vec<ArrayVec<[ArrayVec<[u64; N2]>; N]>>> for U64Data3D {
    fn from(buf_3d: Vec<ArrayVec<[ArrayVec<[u64; N2]>; N]>>) -> Self {
        let list: Vec<*const U64Data2D> = buf_3d
            .into_iter()
            .map(|buf| Box::into_raw(Box::new(U64Data2D::from(buf))) as *const U64Data2D)
            .collect();

        U64Data3D {
            len: list.len(),
            list: Box::into_raw(list.into_boxed_slice()) as *const *const U64Data2D,
        }
    }
}

impl<const N: usize, const N2: usize> From<Vec<ArrayVec<[ArrayVec<[i64; N2]>; N]>>> for I64Data3D {
    fn from(buf_3d: Vec<ArrayVec<[ArrayVec<[i64; N2]>; N]>>) -> Self {
        let list: Vec<*const I64Data2D> = buf_3d
            .into_iter()
            .map(|buf| Box::into_raw(Box::new(I64Data2D::from(buf))) as *const I64Data2D)
            .collect();

        I64Data3D {
            len: list.len(),
            list: Box::into_raw(list.into_boxed_slice()) as *const *const I64Data2D,
        }
    }
}

impl<const N: usize> From<Option<[u16; N]>> for U16Data {
    fn from(maybe_array: Option<[u16; N]>) -> Self {
        maybe_array.map_or(U16Data::empty(), U16Data::from)
    }
}

impl Freeable for Data {
    unsafe fn free(&self) {
        unsafe {
            Vec::from_raw_parts(self.data as *mut u8, self.len, self.len);
        }
    }
}

impl Freeable for U16Data {
    unsafe fn free(&self) {
        if !self.data.is_null() {
            unsafe {
                Vec::from_raw_parts(self.data as *mut u16, self.len, self.len);
            }
        }
    }
}

impl Freeable for U64Data {
    unsafe fn free(&self) {
        if !self.data.is_null() {
            unsafe {
                Vec::from_raw_parts(self.data as *mut u64, self.len, self.len);
            }
        }
    }
}

impl Freeable for I64Data {
    unsafe fn free(&self) {
        unsafe {
            Vec::from_raw_parts(self.data as *mut i64, self.len, self.len);
        }
    }
}

impl Freeable for U64Data2D {
    unsafe fn free(&self) {
        unsafe {
            let list = Vec::from_raw_parts(self.list as *mut *const U64Data, self.len, self.len);

            for data_ptr in list {
                let data = Box::from_raw(data_ptr as *mut U64Data);
                data.free();
            }
        }
    }
}

impl Freeable for I64Data2D {
    unsafe fn free(&self) {
        unsafe {
            let list = Vec::from_raw_parts(self.list as *mut *const I64Data, self.len, self.len);

            for data_ptr in list {
                let data = Box::from_raw(data_ptr as *mut I64Data);
                data.free();
            }
        }
    }
}

impl Freeable for U64Data3D {
    unsafe fn free(&self) {
        unsafe {
            let list = Vec::from_raw_parts(self.list as *mut *const U64Data2D, self.len, self.len);

            for data2d_ptr in list {
                let data2d = Box::from_raw(data2d_ptr as *mut U64Data2D);
                data2d.free();
            }
        }
    }
}

impl Freeable for I64Data3D {
    unsafe fn free(&self) {
        unsafe {
            let list = Vec::from_raw_parts(self.list as *mut *const I64Data2D, self.len, self.len);

            for data2d_ptr in list {
                let data2d = Box::from_raw(data2d_ptr as *mut I64Data2D);
                data2d.free();
            }
        }
    }
}

impl U16Data {
    fn empty() -> Self {
        Self {
            len: 0,
            data: null(),
        }
    }
}
