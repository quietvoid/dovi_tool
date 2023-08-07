/// Dolby Vision RPU (as found in HEVC type 62 NALUs) module
pub mod rpu;

/// Dolby Vision RPU (as found in AV1 ITU T.35 metadata OBUs)
pub mod av1;

/// SMPTE ST2094-10 metadata module
pub mod st2094_10;

/// Various utils
/// cbindgen:ignore
pub mod utils;

/// Dolby Vision XML metadata module
#[cfg(feature = "xml")]
pub mod xml;

/// C API module
#[cfg(any(cargo_c, feature = "capi"))]
pub mod capi;

/// Structs used and exposed in the C API
#[cfg(any(cargo_c, feature = "capi"))]
pub mod c_structs;
