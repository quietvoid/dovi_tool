use super::{DoviProfile, VdrDmData, profile81::Profile81};

pub struct Profile7 {}

impl DoviProfile for Profile7 {
    fn dm_data() -> VdrDmData {
        Profile81::dm_data()
    }
}
