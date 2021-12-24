use super::{profile81::Profile81, DoviProfile, VdrDmData};

pub struct Profile7 {}

impl DoviProfile for Profile7 {
    fn dm_data() -> VdrDmData {
        Profile81::dm_data()
    }
}
