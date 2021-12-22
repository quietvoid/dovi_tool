The generator can create a profile 8.1 RPU binary.  
Any extension metadata can be added.

A JSON config example:

```json5
{
    // CM version, either V29 or V40
    // Defaults to V40
    "cm_version": CmVersion,

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

    // Shots to specify metadata for
    "shots": [
        {
            // Start frame, defaults to 0
            "start": int,
            // Shot frame length
            "duration": int,

            // List of metadata blocks to use for this shot
            // Refer to example or info JSON
            "metadata_blocks": Array,
            // Metadata to use for specific frames in the shot
            "frame_edits": Array
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

    // L6 metadata, required for profile 8.1
    "level6": {
        "max_display_mastering_luminance": int,
        "min_display_mastering_luminance": int,
        "max_content_light_level": int,
        "max_frame_average_light_level": int,
    }
}
```
