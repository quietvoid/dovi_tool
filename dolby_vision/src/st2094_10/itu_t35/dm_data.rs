use anyhow::Result;
use bitvec_helpers::bitvec_reader::BitVecReader;

use super::{ST2094_10Meta, UserDataTypeStruct};

#[derive(Default, Debug)]
pub struct ST2094_10DmData {
    pub app_identifier: u64,
    pub app_version: u64,
    pub metadata_refresh_flag: bool,
    pub dm_data: ST2094_10Meta,
}

impl ST2094_10DmData {
    pub fn parse(reader: &mut BitVecReader) -> Result<UserDataTypeStruct> {
        let mut meta = ST2094_10DmData {
            app_identifier: reader.get_ue()?,
            app_version: reader.get_ue()?,
            metadata_refresh_flag: reader.get()?,
            ..Default::default()
        };

        if meta.metadata_refresh_flag {
            meta.dm_data = ST2094_10Meta::parse(reader)?;
        }

        Ok(UserDataTypeStruct::DMData(meta))
    }
}
