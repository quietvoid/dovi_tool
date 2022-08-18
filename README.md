# **dovi_tool** [![CI](https://github.com/quietvoid/dovi_tool/workflows/CI/badge.svg)](https://github.com/quietvoid/dovi_tool/actions/workflows/ci.yml) [![Artifacts](https://github.com/quietvoid/dovi_tool/workflows/Artifacts/badge.svg)](https://github.com/quietvoid/dovi_tool/actions/workflows/release.yml)

**`dovi_tool`** is a CLI tool combining multiple utilities for working with Dolby Vision.  

The **`dolby_vision`** crate is also hosted in this repo, see [README](dolby_vision/README.md) for use as a Rust/C lib.

&nbsp;

## **Building**
### **Toolchain**

The minimum Rust version to build **`dovi_tool`** is 1.61.0.

### **Release binary**
To build release binary in `target/release/dovi_tool` run:
```console
cargo build --release
```

&nbsp;

## Usage
```properties
dovi_tool [OPTIONS] <SUBCOMMAND>
```
**To get more detailed options for a subcommand**  
```properties
dovi_tool <SUBCOMMAND> --help
```


## All options
- `--help`, `--version`, `--crop`, `--drop-hdr10plus`, `--mode`, `--edit-config`, `--start-code`
## All subcommands
- Metadata utilities: **`info`**, **`generate`**, **`editor`**, **`export`**
- HEVC parsing & handling: **`convert`**, **`demux`**, **`mux`**, **`extract-rpu`**, **`inject-rpu`**

**More information and detailed examples for the subcommands below.**


&nbsp;
# **Dolby Vision metadata utilities**
**`dovi_tool`** provides an important set of tools for analyzing, editing and generating Dolby Vision metadata.
## **Commands**
* ### **info**
    Prints the parsed RPU information.
    To get the summary, use `--summary` or `-s`.

    Using `--frame`: prints the RPU data as JSON for a specific frame.
    - Frame indices start at 0.

    **Example to get metadata for frame 124**:
    ```console
    dovi_tool info -i RPU.bin -f 123
    ```
 
&nbsp;
* ### **generate**
    Allows generating a binary RPU from different sources.  
    &nbsp;
    #### **From an exported CMv2.9 or CMv4.0 Dolby Vision XML metadata file**
    - The binary RPU can be created with support for the following metadata levels:
        - **CMv2.9**: L1, L2, L5, L6
        - **CMv4.0**: **CMv2.9** + L3, L8, L9, L10, L11

        &nbsp;

        **Both per-shot and per-frame trims are supported**.  
        Level 5 metadata requires both `canvas-width` and `canvas-height` to be set.  

        **Example**:
        ```console
        dovi_tool generate --xml dolbyvision_metadata.xml -o RPU_from_xml.bin
        ```

    &nbsp;
    #### **From a generic profile 5/8.1/8.4 configuration JSON file**
    - See documentation: [generator.md](docs/generator.md) or [examples](assets/generator_examples)
 
        **Example**:
        ```console
        dovi_tool generate -j assets/generator_examples/default_cmv40.json -o RPU_generated.bin
        ```
    
    &nbsp;
    #### **From an existing HDR10+ metadata JSON file**
    - The metadata is generated from a configuration JSON file, and the L1 metadata is derived from HDR10+ metadata.  
        The HDR10+ metadata must contain scene information for proper scene cuts.
        
        **Example**:
        ```console
        dovi_tool generate -j assets/generator_examples/default_cmv40.json --hdr10plus-json hdr10plus_metadata.json -o RPU_from_hdr10plus.bin
        ```

    &nbsp;
    #### **From a madVR HDR measurement file**
    - The metadata is generated from a configuration JSON file, and the L1 metadata is derived from the madVR measurements.  
        Supports using custom targets nits from Soulnight's madMeasureHDR Optimizer, with flag `--use-custom-targets`.

        **Example**:
        ```console
        dovi_tool generate -j assets/generator_examples/default_cmv40.json --madvr-file madmeasure-output.bin -o RPU_from_madVR.bin
        ```

&nbsp;
* ### **editor**
    Allows editing a binary RPU according to a JSON config. See documentation: [editor.md](docs/editor.md) or [examples](assets/editor_examples).  
    All indices start at 0, and are inclusive.  For example, using "0-39" edits the first 40 frames.

    **Example**:
    ```console
    dovi_tool editor -i RPU.bin -j assets/editor_examples/mode.json -o RPU_mode2.bin
    ```

&nbsp;
* ### **export**
    Allows exporting a binary RPU file to JSON for simpler analysis.

    **Example**:
    ```console
    dovi_tool export -i RPU.bin -o RPU_export.json
    ```

&nbsp;

# **HEVC parsing & handling**
For working with an HEVC source file, there are multiple options that apply to most commands:
* `-m`, `--mode` Sets the mode for RPU processing.
  * Default (no mode) - Copies the RPU untouched.
  * `0` - Parses the RPU, rewrites it untouched.
  * `1` - Converts the RPU to be MEL compatible.
  * `2` - Converts the RPU to be profile 8.1 compatible.
  * `3` - Converts profile 5 to 8.1.
  * `4` - Converts to profile 8.4.
* `-c`, `--crop` Set active area offsets to 0 (meaning no letterbox bars).
* `--drop-hdr10plus` Ignore HDR10+ metadata when writing the output HEVC.
* `--edit-config` Path to editor config JSON file.
    - Limited editing capabilities when working with HEVC. See [documentation](docs/editor.md).
* `--start-code` HEVC NALU start code to use when writing HEVC.
    - Options: `four` (default), `annex-b`
    - `four` is the default, writing a 4-byte start code all the time.
    - `annex-b` varies the start code, according to spec. Almost matches `x265` behaviour.

## Commands
* ### **convert**
    Converts RPU within a single layer HEVC file.  
    The enhancement layer can be discarded using `--discard`.

    **Examples to convert to profile 8.1 and discard EL**:  
    ```console
    dovi_tool -m 2 convert --discard file.hevc
    ```
    ```console
    ffmpeg -i input.mkv -c:v copy -vbsf hevc_mp4toannexb -f hevc - | dovi_tool -m 2 convert --discard -
    ```

&nbsp;
* ### **demux**
    Rust port of yusesope's python tool. Credits goes to them.  
    Demuxes single track dual layer Dolby Vision into Base layer and Enhancement layer files.  
    Also can be used to remove the RPUs from an HEVC file.

    **Flags**:
    - `--el-only` Output the EL file only.

    **Examples**:
    ```console
    dovi_tool demux file.hevc
    ```
    ```console
    ffmpeg -i input.mkv -c:v copy -vbsf hevc_mp4toannexb -f hevc - | dovi_tool demux -
    ```

    **Example to convert RPU to profile 8.1 while demuxing**:
    ```console
    dovi_tool -m 2 demux file.hevc
    ```

&nbsp;
* ### **mux**
    Interleaves the enhancement layer into a base layer HEVC bitstream.  
    This is the inverse of **`demux`**.

    Muxing supports the base layer input as both raw HEVC bitstream and piped/streamed.

    **Flags**:
    - `--eos-before-el` Write the EOS/EOB NALUs before the EL. Defaults to `false`.  
        This flag enables the same behaviour as MakeMKV and yusesope's mux script.  
        Enabling this therefore results in identical output using **`dovi_tool`**.  

    - `--no-add-aud` Disable adding AUD NALUs between frames
    - `--discard` Discard the EL while muxing. This is equivalent to injecting the RPU, but without extracting first.

    **Examples**:
    ```console
    dovi_tool mux --bl BL.hevc --el EL.hevc
    ```
    ```console
    ffmpeg -i input.mkv -c:v copy -vbsf hevc_mp4toannexb -f hevc - | dovi_tool mux --bl - --el EL.hevc
    ```

    **Example to convert RPU to profile 8.1 while muxing**:
    ```console
    dovi_tool -m 2 mux --bl BL.hevc --el EL.hevc --discard
    ```

&nbsp;
* ### **extract-rpu**
    Extracts Dolby Vision RPU from an HEVC file.  
    This can be either a single track (BL + RPU), single track dual layer (BL+EL+RPU) or an enhancement layer (EL+RPU) video file.  
 
    **Supports profiles 4, 5, 7, and 8**.

    **Examples**:
    ```console
    dovi_tool extract-rpu video.hevc
    ```
    ```console
    ffmpeg -i input.mkv -c:v copy -vbsf hevc_mp4toannexb -f hevc - | dovi_tool extract-rpu - -o RPU.bin
    ```

    **FEL to MEL example**:  
    ```console
    dovi_tool -m 1 extract-rpu video.hevc
    ```

&nbsp;
* ### **inject-rpu**
    Interleaves RPU NAL units between slices in an HEVC encoded bitstream.  
    Global options have no effect when injecting.
    
    **Flags**:
    - `--no-add-aud` Disable adding AUD NALUs between frames

    **Example**:  
    ```console
    dovi_tool inject-rpu -i video.hevc --rpu-in RPU.bin -o injected_output.hevc
    ```

&nbsp;

Build artifacts can be found in the Github Actions.  
More features may or may not be added in the future.
