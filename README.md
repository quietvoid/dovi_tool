# dovi_tool [![Tests](https://github.com/quietvoid/dovi_tool/workflows/Tests/badge.svg)](https://github.com/quietvoid/dovi_tool/actions?query=workflow%3ATests) [![Artifacts](https://github.com/quietvoid/dovi_tool/workflows/Artifacts/badge.svg)](https://github.com/quietvoid/dovi_tool/actions?query=workflow%3AArtifacts)

`dovi_tool` is a CLI tool combining multiple utilities for working with Dolby Vision.

&nbsp;

## Dolby Vision metadata utilities
`dovi_tool` provides an important set of tools for analyzing, editing and generating Dolby Vision metadata.
### Commands
* #### info
    Prints the parsed RPU data as JSON for a specific frame.  
    Frame indices start at 0.

    * Example to get metadata for frame 123: `dovi_tool info -i RPU.bin -f 124`  
&nbsp;
* #### generate
    Allows generating a binary RPU from different sources.
    ##### From an exported CMv2.9 or CMv4.0 Dolby Vision XML metadata file  
    * The binary RPU can be created with support for the following metadata levels:
        * CMv2.9: L1, L2, L5, L6
        * CMv4.0: L1, L2, L3, L5, L6

        Level 5 metadata requires both `canvas-width` and `canvas-height` to be set.
        ###### For L1, L2, L3, and L5 metadata, both per-shot and per-frame trims are supported.
    * Example: `dovi_tool generate --xml dolbyvision_metadata.xml -o RPU_from_xml.bin`  
    &nbsp;
    ##### From a generic profile 8 configuration JSON file  
    * See documentation: [generator.md](generator.md) or [example](assets/generator_example.json)
    * Example: `dovi_tool generate -j assets/generator_example.json -o RPU_generated.bin`  
    &nbsp;
    ##### From an existing HDR10+ metadata JSON file  
    The metadata is generated from a configuration JSON file, and the L1 metadata is derived from HDR10+ metadata.
    * The HDR10+ metadata has to contain scene information for proper scene cuts.
    * Example: `dovi_tool generate -j assets/generator_example.json --hdr10plus-json hdr10plus_metadata.json -o RPU_from_hdr10plus.bin`  
&nbsp;
* #### editor
    Allows editing a binary RPU according to a JSON config.  
    See documentation: [editor.md](editor.md) or [examples](assets/editor_examples).  
    All indices start at 0, and are inclusive.  For example, using "0-39" edits the first 40 frames.
    * Example: `dovi_tool editor -i RPU.bin -j assets/editor_examples/mode.json -o RPU_mode2.bin`  
&nbsp;
* #### export
    Allows exporting a binary RPU file to JSON for simpler analysis.
    * Example: `dovi_tool export -i RPU.bin -o RPU_export.json`

&nbsp;

## HEVC parsing & handling
For working with an HEVC source file, there are multiple options that apply to most commands:
* `-m`, `--mode` Sets the mode for RPU processing.
  * Default (no mode) - Copies the RPU untouched.
  * `0` - Parses the RPU, rewrites it untouched.
  * `1` - Converts the RPU to be MEL compatible.
  * `2` - Converts the RPU to be profile 8.1 compatible.
  * `3` - Converts profile 5 to 8.
* `-c`, `--crop` Set active area offsets to 0 (meaning no letterbox bars)

### Commands
* #### convert
    Converts RPU within a single layer HEVC file.  
    The enhancement layer can be discarded using `--discard`.
    
    Examples to convert to profile 8.1 and discard EL:
    * `dovi_tool -m 2 convert --discard file.hevc`
    * `ffmpeg -i input.mkv -c:v copy -vbsf hevc_mp4toannexb -f hevc - | dovi_tool -m 2 convert --discard -`  
&nbsp;
* #### demux
    Rust port of yusesope's python tool. Credits goes to them.  
    Demuxes single track dual layer Dolby Vision into Base layer and Enhancement layer files.
        
    Examples:
    * `dovi_tool demux file.hevc`
    * `ffmpeg -i input.mkv -c:v copy -vbsf hevc_mp4toannexb -f hevc - | dovi_tool demux -`
    * Convert RPU to profile 8.1 while demuxing: `dovi_tool -m 2 demux file.hevc`  
&nbsp;
* #### extract-rpu
    Extracts Dolby Vision RPU from an single track dual layer encoded file.  
    Supports profiles 4, 5, 7, and 8.

    Examples:
    * `dovi_tool extract-rpu video.hevc`
    * `ffmpeg -i input.mkv -c:v copy -vbsf hevc_mp4toannexb -f hevc - | dovi_tool extract-rpu - -o RPU.bin`
    * FEL to MEL example: `dovi_tool -m 1 extract-rpu video.hevc`  
&nbsp;
* #### inject-rpu
    Interleaves RPU NAL units between slices in an HEVC encoded bitstream.  
    Global options have no effect when injecting.
    
    * Example: `dovi_tool inject-rpu -i video.hevc --rpu-in RPU.bin -o injected_output.hevc`  

&nbsp;

Build artifacts can be found in the Github Actions.  
More features may or may not be added in the future.
