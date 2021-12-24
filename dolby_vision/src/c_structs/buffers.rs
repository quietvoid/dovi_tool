use libc::size_t;

use crate::rpu::NUM_COMPONENTS;

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
pub struct U64Data {
    /// Pointer to the data buffer
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
pub struct Data2D {
    /// Pointer to the list of Data structs
    pub list: *const *const Data,
    /// List length
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
    /// Pointer to the list of Data2D structs
    pub list: *const *const U64Data2D,
    /// List length
    pub len: size_t,
}

/// Struct representing a 3D data buffer
#[repr(C)]
pub struct I64Data3D {
    /// Pointer to the list of Data2D structs
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

impl From<[bool; NUM_COMPONENTS]> for Data {
    fn from(array: [bool; NUM_COMPONENTS]) -> Self {
        let res: [u8; NUM_COMPONENTS] = array.map(|e| e as u8);

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

impl From<Vec<i64>> for I64Data {
    fn from(buf: Vec<i64>) -> Self {
        I64Data {
            len: buf.len(),
            data: Box::into_raw(buf.into_boxed_slice()) as *const i64,
        }
    }
}

impl From<[u64; NUM_COMPONENTS]> for U64Data {
    fn from(array: [u64; NUM_COMPONENTS]) -> Self {
        U64Data {
            len: array.len(),
            data: Box::into_raw(Box::new(array)) as *const u64,
        }
    }
}

impl From<&Vec<[bool; NUM_COMPONENTS]>> for Data2D {
    fn from(buf_2d: &Vec<[bool; NUM_COMPONENTS]>) -> Self {
        let list: Vec<*const Data> = buf_2d
            .clone()
            .into_iter()
            .map(|buf| Box::into_raw(Box::new(Data::from(buf))) as *const Data)
            .collect();

        Data2D {
            len: list.len(),
            list: Box::into_raw(list.into_boxed_slice()) as *const *const Data,
        }
    }
}

impl From<Vec<Vec<u64>>> for U64Data2D {
    fn from(buf_2d: Vec<Vec<u64>>) -> Self {
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

impl From<Vec<Vec<i64>>> for I64Data2D {
    fn from(buf_2d: Vec<Vec<i64>>) -> Self {
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

impl From<Vec<Vec<Vec<u64>>>> for U64Data3D {
    fn from(buf_3d: Vec<Vec<Vec<u64>>>) -> Self {
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

impl From<Vec<Vec<Vec<i64>>>> for I64Data3D {
    fn from(buf_3d: Vec<Vec<Vec<i64>>>) -> Self {
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

impl From<&Vec<[u64; NUM_COMPONENTS]>> for U64Data2D {
    fn from(buf_2d: &Vec<[u64; NUM_COMPONENTS]>) -> Self {
        let list: Vec<*const U64Data> = buf_2d
            .clone()
            .into_iter()
            .map(|buf| Box::into_raw(Box::new(U64Data::from(buf))) as *const U64Data)
            .collect();

        U64Data2D {
            len: list.len(),
            list: Box::into_raw(list.into_boxed_slice()) as *const *const U64Data,
        }
    }
}

impl Freeable for Data {
    unsafe fn free(&self) {
        Vec::from_raw_parts(self.data as *mut u8, self.len as usize, self.len as usize);
    }
}

impl Freeable for U64Data {
    unsafe fn free(&self) {
        Vec::from_raw_parts(self.data as *mut u64, self.len as usize, self.len as usize);
    }
}

impl Freeable for I64Data {
    unsafe fn free(&self) {
        Vec::from_raw_parts(self.data as *mut i64, self.len as usize, self.len as usize);
    }
}

impl Freeable for Data2D {
    unsafe fn free(&self) {
        let list = Vec::from_raw_parts(
            self.list as *mut *const Data,
            self.len as usize,
            self.len as usize,
        );

        for data_ptr in list {
            let data = Box::from_raw(data_ptr as *mut Data);
            data.free();
        }
    }
}

impl Freeable for U64Data2D {
    unsafe fn free(&self) {
        let list = Vec::from_raw_parts(
            self.list as *mut *const U64Data,
            self.len as usize,
            self.len as usize,
        );

        for data_ptr in list {
            let data = Box::from_raw(data_ptr as *mut U64Data);
            data.free();
        }
    }
}

impl Freeable for I64Data2D {
    unsafe fn free(&self) {
        let list = Vec::from_raw_parts(
            self.list as *mut *const I64Data,
            self.len as usize,
            self.len as usize,
        );

        for data_ptr in list {
            let data = Box::from_raw(data_ptr as *mut I64Data);
            data.free();
        }
    }
}

impl Freeable for U64Data3D {
    unsafe fn free(&self) {
        let list = Vec::from_raw_parts(
            self.list as *mut *const U64Data2D,
            self.len as usize,
            self.len as usize,
        );

        for data2d_ptr in list {
            let data2d = Box::from_raw(data2d_ptr as *mut U64Data2D);
            data2d.free();
        }
    }
}

impl Freeable for I64Data3D {
    unsafe fn free(&self) {
        let list = Vec::from_raw_parts(
            self.list as *mut *const I64Data2D,
            self.len as usize,
            self.len as usize,
        );

        for data2d_ptr in list {
            let data2d = Box::from_raw(data2d_ptr as *mut I64Data2D);
            data2d.free();
        }
    }
}
