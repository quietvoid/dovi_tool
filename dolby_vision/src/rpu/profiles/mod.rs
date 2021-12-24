use super::vdr_dm_data::VdrDmData;

pub mod profile4;
pub mod profile5;
pub mod profile7;
pub mod profile81;

pub trait DoviProfile {
    fn dm_data() -> VdrDmData {
        VdrDmData::default_pq()
    }

    fn backwards_compatible() -> bool {
        true
    }
}
