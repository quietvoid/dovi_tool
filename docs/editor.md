The RPUs can be edited in two ways:
- Using the **`editor`** subcommand.
- Using the **`--edit-config`** option when using HEVC commands.

When doing HEVC operations, some capabilities are not supported:
- Editing the active area for specific ranges of frames. Only `"all"` edit is supported.
- Removing or duplicating RPUs.
- Editing scene cuts.

&nbsp;

The editor expects a JSON config like the example below:
```json5
{
    // Mode to convert the RPU (refer to README)
    "mode": int,

    // Removes CM v4.0 from the RPU:
    //   - L3, L8, L9, L10 and L11 are removed
    //   - DM v2 metadata is removed, along with L254
    "remove_cmv4": boolean,

    // Whether to remove polynomial/MMR mapping coefficients from the metadata
    "remove_mapping": boolean,

    // Source min/max PQ values to override
    "min_pq": int,
    "max_pq": int,

    // Configuration for active area edits
    // If no L5 metadata is present in the RPU, L5 metadata is inserted
    "active_area": {
        // Should be set to true when final video has no letterbox bars
        "crop": boolean,

        // Optional, specifies whether to drop some or all L5 metadata.
        // This produces spec non conformant RPUs.
        // Possible options: "all", "zeroes"
        //   "zeroes" drops the L5 metadata blocks which have all offsets set to zero.
        "drop_l5": string,

        // List of presets to add letterbox bars
        "presets": [
            {
                "id": int,
                "left": int,
                "right": int,
                "top": int,
                "bottom": int
            }
        ],

        // List of edits
        "edits": {
            // All or a specific range of frames (inclusive) to use preset for
            // Edits before an "all" key can be overriden
            "all": presetId,
            "0-39": presetId
        }
    },

    // List of frames or frame ranges to remove (inclusive)
    // Frames are removed before the duplicate passes
    "remove": [
        "0-39"
    ],

    // List of duplicate operations
    "duplicate": [
        {
            // Frame to use as metadata source
            "source": int,
            // Index at which the duplicated frames are added (inclusive)
            "offset": int,
            // Number of frames to duplicate
            "length": int
        }
    ],

    // Set the scene cut (scene_refresh_flag) flag for specific frame ranges
    // Range options: "all" or formatted as "start-end"
    "scene_cuts": {
        "all": true,
        "0-39": false
    },

    // Level 6, ST2086 fallback metadata
    // Optional
    //   Replaces existing L6 metadata values.
    //   Otherwise, creates the L6 metadata block.
    "level6": {
        "max_display_mastering_luminance": int,
        "min_display_mastering_luminance": int,
        "max_content_light_level": int,
        "max_frame_average_light_level": int
    },


    // Level 9 Mastering Display Primaries
    // Optional, replaces existing L9.
    // The RPU must already be CM v4.0 for this to have any effect
    //
    // String value, must match enum.
    // Default: "DCIP3D65".
    "level9": MasteringDisplayPrimaries,

    // Level 11 Content type metadata
    // Optional, replaces existing L11
    // The RPU must already be CM v4.0 for this to have any effect
    "level11": {
        // 1 = Cinema, 2 = Games, 3 = Sports, 4 = User generated content
        "content_type": int,

        // WP * 375 + 6504
        // D65 = 0
        "whitepoint": int,

        // Whether to force reference mode or not.
        "reference_mode_flag": boolean
    },

    // Level 255 extension block structure
    "level255": ExtMetadataBlockLevel255
}
```
