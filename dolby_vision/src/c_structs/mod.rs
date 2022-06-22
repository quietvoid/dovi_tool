use crate::rpu::NUM_COMPONENTS;

mod buffers;
mod extension_metadata;
mod rpu;
mod rpu_data_header;
mod rpu_data_mapping;
mod rpu_data_nlq;
mod vdr_dm_data;

pub use buffers::*;
pub use extension_metadata::DmData;
pub use rpu::{RpuOpaque, RpuOpaqueList};
pub use rpu_data_header::RpuDataHeader;
pub use rpu_data_mapping::RpuDataMapping;
pub use rpu_data_nlq::RpuDataNlq;
pub use vdr_dm_data::VdrDmData;

fn components_to_cdata<T, R>(cmps: &[T; NUM_COMPONENTS]) -> [R; NUM_COMPONENTS]
where
    T: Clone,
    R: From<T>,
{
    [
        R::from(cmps[0].clone()),
        R::from(cmps[1].clone()),
        R::from(cmps[2].clone()),
    ]
}
