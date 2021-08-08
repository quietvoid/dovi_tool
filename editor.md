The editor expects a JSON config like the example below:
```json5
{
    // Mode to convert the RPU (refer to README)
    "mode": int,
    
    // Configuration for active area edits
    "active_area": {
        // Should be set to true when final video has no letterbox bars
        "crop": boolean,

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
            // Range of frames (inclusive) to use preset for
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

    // Source min/max PQ values to override
    "min_pq": int,
    "max_pq": int
}
```