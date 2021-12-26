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
