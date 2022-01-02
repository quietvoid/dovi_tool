The generator can create a profile 8.1 RPU binary.  
Any extension metadata can be added.

A JSON config example:

```json5
{
    // CM version, either "V29" or "V40".
    // Defaults to "V40".
    "cm_version": string,

    // Number of metadata frames to generate.
    // Optional if shots are specified, as well as for HDR10+ and madVR sourced generation.
    "length": int,

    // Source min/max PQ values to override, optional.
    // If not specified, derived from L6 metadata.
    "source_min_pq": int,
    "source_max_pq": int,

    // L5 metadata, optional.
    // If not specified, L5 metadata is added with 0 offsets.
    "level5": {
        "active_area_left_offset": int,
        "active_area_right_offset": int,
        "active_area_top_offset": int,
        "active_area_bottom_offset": int,
    },

    // L6 metadata, required for profile 8.1.
    "level6": {
        "max_display_mastering_luminance": int,
        "min_display_mastering_luminance": int,
        "max_content_light_level": int,
        "max_frame_average_light_level": int,
    },

    // Metadata blocks that should be present in every RPU of the sequence.
    // Does not accept L5, L6 and L254 metadata.
    // Disallowed blocks are simply ignored.
    //
    // For HDR10+ or madVR generation, the default L1 metadata is replaced.
    //
    // Refer to assets/generator_examples/full_example.json
    "default_metadata_blocks": Array,

    // Shots to specify metadata.
    // Array of VideoShot objects.
    //
    // For HDR10+ or madVR generation:
    //   - The metadata is taken from the shots in the list order.
    //     This means that both start and duration can be 0.
    //   - It is expected that the source metadata has the same number of shots as this list.
    //     Missing or extra shots are ignored.
    //
    // Refer to generator examples.
    "shots": [
        {
            // Start frame.
            "start": int,
            // Shot frame length.
            "duration": int,

            // List of metadata blocks to use for this shot.
            "metadata_blocks": Array,

            // Metadata to use for specific frames in the shot.
            "frame_edits": [
                {
                    // Frame offset to edit in the shot.
                    "edit_offset": int,

                    // List of metadata blocks to use for the frame.
                    "metadata_blocks": Array,
                }
            ]
        }
    ],
}
```
