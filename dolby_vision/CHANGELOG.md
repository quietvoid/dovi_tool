## Unreleased
- Added `write_av1_rpu_metadata_obu_t35_complete` function to encode RPUs in complete metadata OBU payloads.
- Added support for parsing `ext_mapping_idc` in `RpuDataHeader`.
  - `ext_mapping_idc_lsb` represents the 5 lowest bits, and `ext_mapping_idc_msb` the 3 remaining bits.

C API:
- Added `dovi_write_av1_rpu_metadata_obu_t35_{payload,complete}` functions.

## 3.2.0
- Deprecated `RpuDataHeader.rpu_nal_prefix`.
- Added `av1` module for handling AV1 Dolby Vision ITU-T T.35 metadata OBU payloads.
- AV1 RPU bytes can now be encoded with `write_av1_rpu_metadata_obu_t35_payload`.
  - The payload is meant to be used for `itu_t_t35_payload_bytes`.

## 3.1.1
- Fixed RPU writing edge case that resulted in non conformant NALU bytes when using `write_hevc_unspec62_nalu`.

## 3.1.0
- Conversion mode 2 now defaults to remove luma and chroma mapping by default, only for profile 7 FEL.
- Added `ConversionMode::To81MappingPreserved` for old mode 2 behaviour.

## 3.0.0
- Breaking changes from `RpuDataMapping` refactor.
- Renamed `serde_feature` to simply `serde`.

- Moved some fields from `RpuDataHeader` into `RpuDataMapping`.
- `RpuDataNlq` is now part of `RpuDataMapping`.

- The mapping now has one curve per component, which is a `DoviReshapingCurve`.
- `DoviReshapingCurve` contains the pivots, mapping method and the respective curve params.
    - Component 0 describes the polynomial params in `DoviPolynomialCurve`.
    - Components 1 and 2 can be either polynomial or the MMR params in `DoviMMRCurve`.
    - Polynomial interpolation fields were removed as there are no existing samples.

- `RpuDataNlq` was changed to contain only one set of params, as there is no significant pivot.
- All `_minus_X` suffixed names were uniformized as `minusX`.

The changes also affect the C API.

C API:
- Added `dovi_rpu_set_active_area_offsets` function to edit L5 metadata.  
- Added `dovi_rpu_remove_mapping` function.


## 2.1.0
- Made some parsing functions private, as they were always meant to be internal only.
- Replaced `DoviRpu::trailing_bytes` by `trailing_zeroes`, which is only the count of the zero bytes.
- Changed `DoviRpu::subprofile` to `&str`.

## 2.0.1
- Added `replace_levels_from_rpu` function to `DoviRpu`.
- Added `l1_avg_pq_cm_version` to `GenerateConfig`.
    - Allows overriding the minimum L1 `avg_pq` CM version.
    - Example use case: Some grades are done in `CM v4.0` but distributed as `CM v2.9` RPU.

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
