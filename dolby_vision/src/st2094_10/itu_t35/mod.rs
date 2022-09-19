use anyhow::{bail, ensure, Result};
use bitvec_helpers::bitvec_reader::BitVecReader;

use crate::utils::clear_start_code_emulation_prevention_3_byte;

mod cm_data;
mod dm_data;

use cm_data::ST2094_10CmData;
use dm_data::ST2094_10DmData;

/// ITU T.35 SEI version of ST2094-10 metadata
#[derive(Debug)]
pub struct ST2094_10ItuT35 {
    pub user_data_type_struct: UserDataTypeStruct,
}

#[derive(Debug)]
pub enum UserDataTypeStruct {
    DMData(ST2094_10DmData),
    CMData(Box<ST2094_10CmData>),
}

impl ST2094_10ItuT35 {
    /// Implementation of https://dashif-documents.azurewebsites.net/DASH-IF-IOP/master/DASH-IF-IOP.html#codecs-dolbyvision
    pub fn parse_itu_t35_dashif(data: &[u8]) -> Result<ST2094_10ItuT35> {
        let trimmed_data = Self::validated_trimmed_data(data)?;
        let bytes = clear_start_code_emulation_prevention_3_byte(trimmed_data);

        let mut reader = BitVecReader::new(bytes);

        let itu_t_t35_country_code: u8 = reader.get_n(8)?;
        let itu_t_t35_provider_code: u16 = reader.get_n(16)?;

        ensure!(itu_t_t35_country_code == 0xB5);
        ensure!(itu_t_t35_provider_code == 0x31);

        let user_identifier: u32 = reader.get_n(32)?;
        ensure!(
            user_identifier == 0x47413934,
            "invalid user_identifier: {}",
            user_identifier
        );

        let user_data_type_code: u8 = reader.get_n(8)?;

        let meta = match user_data_type_code {
            0x08 => ST2094_10CmData::parse(&mut reader)?,
            0x09 => ST2094_10DmData::parse(&mut reader)?,
            _ => bail!("Invalid user_data_type_code: {}", user_data_type_code),
        };

        Ok(ST2094_10ItuT35 {
            user_data_type_struct: meta,
        })
    }

    pub fn validated_trimmed_data(data: &[u8]) -> Result<&[u8]> {
        let trimmed_data = match &data[..7] {
            [0x4E, 0x01, 0x04, _, 0xB5, 0x00, 0x31] => &data[4..],
            [0xB5, 0x00, 0x31, 0x47, 0x41, 0x39, 0x34] => data,
            _ => bail!("Invalid St2094-10 T-T35 SEI start bytes\n{:?}", &data[..7]),
        };

        Ok(trimmed_data)
    }
}
