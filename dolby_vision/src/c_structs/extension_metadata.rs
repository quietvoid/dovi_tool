use libc::size_t;
use std::ptr::null;

use crate::rpu::extension_metadata::DmData as RuDmData;
use crate::rpu::extension_metadata::WithExtMetadataBlocks;
use crate::rpu::extension_metadata::blocks::*;
use crate::rpu::vdr_dm_data::CmVersion;

/// C struct for the list of ext_metadata_block()
#[repr(C)]
pub struct DmData {
    /// Number of metadata blocks
    num_ext_blocks: u64,

    level1: *const ExtMetadataBlockLevel1,
    level2: Level2BlockList,
    level3: *const ExtMetadataBlockLevel3,
    level4: *const ExtMetadataBlockLevel4,
    level5: *const ExtMetadataBlockLevel5,
    level6: *const ExtMetadataBlockLevel6,
    level8: Level8BlockList,
    level9: *const ExtMetadataBlockLevel9,
    level10: Level10BlockList,
    level11: *const ExtMetadataBlockLevel11,
    level254: *const ExtMetadataBlockLevel254,
    level255: *const ExtMetadataBlockLevel255,
}

#[repr(C)]
pub struct Level2BlockList {
    /// Pointer to the list of ExtMetadataBlockLevel2 structs
    pub list: *const *const ExtMetadataBlockLevel2,
    /// List length
    pub len: size_t,
}

#[repr(C)]
pub struct Level8BlockList {
    /// Pointer to the list of ExtMetadataBlockLevel8 structs
    pub list: *const *const ExtMetadataBlockLevel8,
    /// List length
    pub len: size_t,
}

#[repr(C)]
pub struct Level10BlockList {
    /// Pointer to the list of ExtMetadataBlockLevel10 structs
    pub list: *const *const ExtMetadataBlockLevel10,
    /// List length
    pub len: size_t,
}

impl DmData {
    pub fn combine_dm_data(
        cmv29_metadata: Option<&RuDmData>,
        cmv40_metadata: Option<&RuDmData>,
    ) -> Self {
        let mut dm_data = Self::default();

        if let Some(RuDmData::V29(cmv29)) = cmv29_metadata {
            dm_data.num_ext_blocks += cmv29.num_ext_blocks();

            dm_data.set_blocks(cmv29.blocks_ref(), CmVersion::V29);
        }

        if let Some(RuDmData::V40(cmv40)) = cmv40_metadata {
            dm_data.num_ext_blocks += cmv40.num_ext_blocks();

            dm_data.set_blocks(cmv40.blocks_ref(), CmVersion::V40);
        }

        dm_data
    }

    fn set_blocks(&mut self, blocks: &[ExtMetadataBlock], cm_version: CmVersion) {
        for block in blocks {
            match block {
                ExtMetadataBlock::Level1(b) => {
                    self.level1 =
                        Box::into_raw(Box::new(b.clone())) as *const ExtMetadataBlockLevel1
                }
                ExtMetadataBlock::Level2(_) => {}
                ExtMetadataBlock::Level3(b) => {
                    self.level3 =
                        Box::into_raw(Box::new(b.clone())) as *const ExtMetadataBlockLevel3
                }
                ExtMetadataBlock::Level4(b) => {
                    self.level4 =
                        Box::into_raw(Box::new(b.clone())) as *const ExtMetadataBlockLevel4
                }
                ExtMetadataBlock::Level5(b) => {
                    self.level5 =
                        Box::into_raw(Box::new(b.clone())) as *const ExtMetadataBlockLevel5
                }
                ExtMetadataBlock::Level6(b) => {
                    self.level6 =
                        Box::into_raw(Box::new(b.clone())) as *const ExtMetadataBlockLevel6
                }
                ExtMetadataBlock::Level8(_) => {}
                ExtMetadataBlock::Level9(b) => {
                    self.level9 =
                        Box::into_raw(Box::new(b.clone())) as *const ExtMetadataBlockLevel9
                }
                ExtMetadataBlock::Level10(_) => {}
                ExtMetadataBlock::Level11(b) => {
                    self.level11 =
                        Box::into_raw(Box::new(b.clone())) as *const ExtMetadataBlockLevel11
                }
                ExtMetadataBlock::Level15(_) => {
                    // TODO
                }
                ExtMetadataBlock::Level16(_) => {
                    // TODO
                }
                ExtMetadataBlock::Level254(b) => {
                    self.level254 =
                        Box::into_raw(Box::new(b.clone())) as *const ExtMetadataBlockLevel254
                }
                ExtMetadataBlock::Level255(b) => {
                    self.level255 =
                        Box::into_raw(Box::new(b.clone())) as *const ExtMetadataBlockLevel255
                }
                ExtMetadataBlock::Reserved(_) => {}
            };
        }

        // Don't overwrite over previously set data
        match cm_version {
            CmVersion::V29 => self.level2 = Level2BlockList::from(blocks),
            CmVersion::V40 => {
                self.level8 = Level8BlockList::from(blocks);
                self.level10 = Level10BlockList::from(blocks);
            }
        }
    }

    /// # Safety
    pub unsafe fn free(&self) {
        unsafe {
            drop(Box::from_raw(self.level1 as *mut ExtMetadataBlockLevel1));
            self.level2.free();
            drop(Box::from_raw(self.level3 as *mut ExtMetadataBlockLevel3));
            drop(Box::from_raw(self.level4 as *mut ExtMetadataBlockLevel4));
            drop(Box::from_raw(self.level5 as *mut ExtMetadataBlockLevel5));
            drop(Box::from_raw(self.level6 as *mut ExtMetadataBlockLevel6));
            self.level8.free();
            drop(Box::from_raw(self.level9 as *mut ExtMetadataBlockLevel9));
            self.level10.free();
            drop(Box::from_raw(self.level11 as *mut ExtMetadataBlockLevel11));
            drop(Box::from_raw(
                self.level254 as *mut ExtMetadataBlockLevel254,
            ));
            drop(Box::from_raw(
                self.level255 as *mut ExtMetadataBlockLevel255,
            ));
        }
    }
}

impl Default for DmData {
    fn default() -> Self {
        Self {
            num_ext_blocks: Default::default(),
            level1: null(),
            level2: Default::default(),
            level3: null(),
            level4: null(),
            level5: null(),
            level6: null(),
            level8: Default::default(),
            level9: null(),
            level10: Default::default(),
            level11: null(),
            level254: null(),
            level255: null(),
        }
    }
}

impl Level2BlockList {
    /// # Safety
    pub unsafe fn free(&self) {
        unsafe {
            let list = Vec::from_raw_parts(
                self.list as *mut *const ExtMetadataBlockLevel2,
                self.len,
                self.len,
            );

            for data_ptr in list {
                drop(Box::from_raw(data_ptr as *mut ExtMetadataBlockLevel2));
            }
        }
    }
}

impl Level8BlockList {
    /// # Safety
    pub unsafe fn free(&self) {
        unsafe {
            let list = Vec::from_raw_parts(
                self.list as *mut *const ExtMetadataBlockLevel8,
                self.len,
                self.len,
            );

            for data_ptr in list {
                drop(Box::from_raw(data_ptr as *mut ExtMetadataBlockLevel8));
            }
        }
    }
}

impl Level10BlockList {
    /// # Safety
    pub unsafe fn free(&self) {
        unsafe {
            let list = Vec::from_raw_parts(
                self.list as *mut *const ExtMetadataBlockLevel10,
                self.len,
                self.len,
            );

            for data_ptr in list {
                drop(Box::from_raw(data_ptr as *mut ExtMetadataBlockLevel10));
            }
        }
    }
}

impl From<&[ExtMetadataBlock]> for Level2BlockList {
    fn from(blocks: &[ExtMetadataBlock]) -> Self {
        let level2_blocks: Vec<*const ExtMetadataBlockLevel2> = blocks
            .iter()
            .filter(|b| matches!(b, ExtMetadataBlock::Level2(_)))
            .map(|b| match b {
                ExtMetadataBlock::Level2(e) => {
                    Box::into_raw(Box::new(e.clone())) as *const ExtMetadataBlockLevel2
                }
                _ => null(),
            })
            .collect();

        Self {
            len: level2_blocks.len(),
            list: Box::into_raw(level2_blocks.into_boxed_slice())
                as *const *const ExtMetadataBlockLevel2,
        }
    }
}

impl From<&[ExtMetadataBlock]> for Level8BlockList {
    fn from(blocks: &[ExtMetadataBlock]) -> Self {
        let level8_blocks: Vec<*const ExtMetadataBlockLevel8> = blocks
            .iter()
            .filter(|b| matches!(b, ExtMetadataBlock::Level8(_)))
            .map(|b| match b {
                ExtMetadataBlock::Level8(e) => {
                    Box::into_raw(Box::new(e.clone())) as *const ExtMetadataBlockLevel8
                }
                _ => null(),
            })
            .collect();

        Self {
            len: level8_blocks.len(),
            list: Box::into_raw(level8_blocks.into_boxed_slice())
                as *const *const ExtMetadataBlockLevel8,
        }
    }
}

impl From<&[ExtMetadataBlock]> for Level10BlockList {
    fn from(blocks: &[ExtMetadataBlock]) -> Self {
        let level10_blocks: Vec<*const ExtMetadataBlockLevel10> = blocks
            .iter()
            .filter(|b| matches!(b, ExtMetadataBlock::Level10(_)))
            .map(|b| match b {
                ExtMetadataBlock::Level10(e) => {
                    Box::into_raw(Box::new(e.clone())) as *const ExtMetadataBlockLevel10
                }
                _ => null(),
            })
            .collect();

        Self {
            len: level10_blocks.len(),
            list: Box::into_raw(level10_blocks.into_boxed_slice())
                as *const *const ExtMetadataBlockLevel10,
        }
    }
}

impl Default for Level2BlockList {
    fn default() -> Self {
        let len = 0;
        let list: Vec<*const ExtMetadataBlockLevel2> = Vec::new();

        Self {
            list: Box::into_raw(list.into_boxed_slice()) as *const *const ExtMetadataBlockLevel2,
            len,
        }
    }
}

impl Default for Level8BlockList {
    fn default() -> Self {
        let len = 0;
        let list: Vec<*const ExtMetadataBlockLevel8> = Vec::new();

        Self {
            list: Box::into_raw(list.into_boxed_slice()) as *const *const ExtMetadataBlockLevel8,
            len,
        }
    }
}

impl Default for Level10BlockList {
    fn default() -> Self {
        let len = 0;
        let list: Vec<*const ExtMetadataBlockLevel10> = Vec::new();

        Self {
            list: Box::into_raw(list.into_boxed_slice()) as *const *const ExtMetadataBlockLevel10,
            len,
        }
    }
}
