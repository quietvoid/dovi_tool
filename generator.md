The generator can create a profile 8, limited metadata RPU binary.  
A JSON config example:

```json5
{
    // Number of metadata frames to generate
    "length": int,

    // Target nits for L2 metadata (0 to 10000).
    // Usually 600, 1000, 2000
    // Optional if specific L2 targets are present
    "target_nits": int,

    // Source min/max PQ values to override, optional
    // If not specified, derived from L6 metadata
    "source_min_pq": int,
    "source_max_pq": int,

    // L2 metadata, optional
    // If not specified, nothing added
    // By default, trim adjustments are set to 2048 (no adjust)
    "level2": [
        {
            "target_nits": 600,
            "trim_slope": 2000,
            "trim_saturation_gain": 2096
        },
        {
            "target_nits": 1000
        },
        {
            "target_nits": 2000
        }
    ],

    // L5 metadata, optional
    // If not specified, L5 metadata is added with 0 offsets
    "level5": {
        "active_area_left_offset": int,
        "active_area_right_offset": int,
        "active_area_top_offset": int,
        "active_area_bottom_offset": int,
    },

    // L6 metadata, optional
    "level6": {
        "max_display_mastering_luminance": int,
        "min_display_mastering_luminance": int,
        "max_content_light_level": int,
        "max_frame_average_light_level": int,
    }
}
```