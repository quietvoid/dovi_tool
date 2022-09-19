## ??

## 2.0.0
- Modified `extension_metadata::blocks` parsing functions to return a `Result`.

## 1.7.1
- Add `ExtMetadataBlockLevel1` constructor `new` and `from_stats_cm_version`.
- Add `clamp_values_cm_version` function to `ExtMetadataBlockLevel1`.
- Deprecated `ExtMetadataBlockLevel1` functions `from_stats` and `clamp_values`.

## 1.7.0
- Add `clamp_values` function to `ExtMetadataBlockLevel1`.
- Add `fixup_l1` function to `GenerateConfig`.
- Allow replacing `L254` extension metadata blocks.

## 1.6.7

- Add `rpu::utils` module, and `parse_rpu_file` helper function.
- Made `bitvec_ser_bits` private as it shouldn't be exposed.
- Added `dovi_parse_rpu_bin_file` to C API functions.
- Fixed memory leaks from errors in the C API.

## 1.6.6

- Add `ConversionMode` enum to use with `DoviRpu::convert_with_mode`.
- Added support to generate profile 5 RPUs.
- Added long play mode RPU generation.
    - This sets `scene_refresh_flag` to `1` for every frame.
- Deprecated `DoviRpu::convert_to_cmv4` as it can lead to playback issues.

## 1.6.5

- Breaking: Made `GenerateConfig::level6` optional.
- Added `profile` to `GenerateConfig` to support profile 8.1 and 8.4.

## 1.6.4

- Add `DoviRpu::convert_to_cmv40` helper method.
- Add `DoviRpu::subprofile` field, for profile 7 FEL or MEL.

## 1.6.3

- Add support for compressed RPU format.
- Add `RpuDataHeader::nlq_pred_pivot_value` field, parsed for profile 4 and 7.

## 1.6.2

- Updated `bitvec` dependency to 1.0.0.
- Allowed noop conversion when converting a profile 8 RPU with mode 2.
- Removed `last_byte` field from `DoviRpu`, replaced by `trailing_bytes` Vec.
    - Fixes parsing when the NAL has multiple trailing 0 bytes.

## 1.6.1

- Add support for variable length blocks: L8, L9, L10.
- Add L9 metadata by default when generating CM v4.0 RPUs.
- Add support for L255 block in DM v1 payload.

XML parser:
- Improve specific version support up to XML version 5.1.0.
- Add L10/L11 metadata parsing from XML.

## 1.6.0

- Fixed deserialize default value for `GenerateConfig`.`cm_version` field.
- Added `default_metadata_blocks` to `GenerateConfig` struct.
- Removed `target_nits` field from `GenerateConfig`. Use default blocks.

## 1.5.2

Changed DM data logic to write the number of blocks and align even if there are none.

## 1.5.1

Fix bug where metadata blocks were reordered after parsing, altering the final CRC32.

## 1.5.0

A bunch of breaking changes to add CMv4.0.
Reworked extension metadata, different DM data payloads.

C API:
- Renamed `st2094_10_metadata` to `dm_data` in `DoviVdrDmData`.
- Added more levels to `dm_data` struct.

## 1.4.0

Fixed L3/L4 metadata block sizes.

Renamed `ExtMetadataBlock` functions:
- `length` -> `length_bytes`
- `bits` -> `length_bits`

## 1.3.1

Conditional initializations of `Vec`s in `RpuDataMapping` and `RpuDataNlq` structs.  
- Represents the actual parsed metadata better, instead of being defaulted to 0.

Added `guessed_profile` field to C API `DoviRpuDataHeader` struct.
