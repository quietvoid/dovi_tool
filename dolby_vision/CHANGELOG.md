## 1.4.0

Fixed L3/L4 metadata block sizes.

Renamed `ExtMetadataBlock` functions:
- `length` -> `length_bytes`
- `bits` -> `length_bits`

## 1.3.1

Conditional initializations of `Vec`s in `RpuDataMapping` and `RpuDataNlq` structs.  
- Represents the actual parsed metadata better, instead of being defaulted to 0.

Added `guessed_profile` field to C API `DoviRpuDataHeader` struct.
