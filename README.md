# **dovi_tool** [![CI](https://github.com/quietvoid/dovi_tool/workflows/CI/badge.svg)](https://github.com/quietvoid/dovi_tool/actions/workflows/ci.yml) [![Artifacts](https://github.com/quietvoid/dovi_tool/workflows/Artifacts/badge.svg)](https://github.com/quietvoid/dovi_tool/actions/workflows/release.yml)

**`dovi_tool`** is a CLI tool combining multiple utilities for working with Dolby Vision.  

The **`dolby_vision`** crate is also hosted in this repo, see [README](dolby_vision/README.md) for use as a Rust/C lib.  
The C compatible library is also known as **`libdovi`**, refer to the same document for building/installing.

&nbsp;

## **Building**
### **Toolchain**

The minimum Rust version to build **`dovi_tool`** is 1.85.0.

### **Dependencies**
On Linux systems, [fontconfig](https://github.com/yeslogic/fontconfig-rs#dependencies) is required.  
Alternatively, system fonts can be bypassed by building with `--no-default-features --features internal-font`.

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
- Metadata utilities: **`info`**, **`generate`**, **`editor`**, **`export`**, **`plot`**
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

        **Flags**:
        - `--hdr10plus-peak-source` How to extract the peak brightness for the metadata [default: `histogram`]      
            Possible values: `histogram`, `histogram99`, `max-scl`, `max-scl-luminance`

        
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
    Allows exporting a binary RPU file to text files containing relevant information.  
    The command allows specifying the desired data to export as file.  
    **Default**: `export` outputs the full RPU serialized to JSON (equivalent to `--data all`).

    * `-d`, `--data`: List of key-value export parameters formatted as `key=output,key2...`
      * `all` - Exports the list of RPUs as a JSON file
      * `scenes` - Exports the frame indices at which `scene_refresh_flag` is set to 1
      * `level5` - Exports the video's L5 metadata in the form of an `editor` config JSON

    &nbsp;

    **Example to export the whole RPU list to JSON**:
    ```console
    dovi_tool export -i RPU.bin -d all=RPU_export.json
    ```

    **Example to export both scene change frames and L5 metadata (with specific path)**
    ```console
    dovi_tool export -i RPU.bin -d scenes,level5=L5.json
    ```

&nbsp;
* ### **plot**
    Allows plotting the RPU metadata into a graph.  
    The output is a PNG image.

    **Flags**:
    - `-t`, `--title` The title to set at the top of the plot
    - `-s`, `--start` Set frame range start
    - `-e`, `--end` Set frame range end (inclusive)

    Plot options:
    - `-p`, `--plot-type` Sets the DV metadata level to plot [default: `l1`, brightness metadata]  
        Possible values: `l1`, `l2`, `l8`, `l8-saturation`, `l8-hue`

    - `--target-nits` Target brightness in nits for L2/L8 plots [default: `100`]  
        Possible values: `100`, `300`, `600`, `1000`, `2000`, `4000`

    - `--trims` Trim parameters to include in L2/L8 trims plots. By default all are included.  
        Possible values: `slope`, `offset`, `power`, `chroma`, `saturation`, `ms`, `mid`, `clip`  
        `L8` only: `mid` and `clip`.

    **Example**:
    ```console
    dovi_tool plot RPU.bin -t "Dolby Vision L1 plot" -o L1_plot.png

    # L2 plot
    dovi_tool plot RPU.bin -p l2
    ```

&nbsp;

# **HEVC parsing & handling**
For working with an HEVC source file, there are multiple options that apply to most commands:

### Conversion modes
* `-m`, `--mode` Sets the mode for RPU processing.
  * Default (no mode) - Copies the RPU untouched.
  * `0` - Parses the RPU, rewrites it untouched.
  * `1` - Converts the RPU to be MEL compatible.
  * `2` - Converts the RPU to be profile 8.1 compatible.
      - Removes luma/chroma mapping for profile 7 FEL.
  * `3` - Converts profile 5 to 8.1.
  * `4` - Converts to profile 8.4.
  * `5` - Converts to profile 8.1, preserving mapping.
      - Old mode 2.

### Other options
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
    ffmpeg -i input.mkv -c:v copy -bsf:v hevc_mp4toannexb -f hevc - | dovi_tool -m 2 convert --discard -
    ```

&nbsp;
* ### **demux**
    Rust port of yusesope's python tool. Credits goes to them.  
    Demuxes single track dual layer Dolby Vision into Base layer and Enhancement layer files.  
    The base layer file output is equivalent to using the `remove` subcommand.

    **Flags**:
    - `--el-only` Output the EL file only.

    **Examples**:
    ```console
    dovi_tool demux file.hevc
    ```
    ```console
    ffmpeg -i input.mkv -c:v copy -bsf:v hevc_mp4toannexb -f hevc - | dovi_tool demux -
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

    - `--no-add-aud` Disable adding AUD NALUs between frames
    - `--remove-eos` Removes EOS/EOB NALUs from both BL and EL, if present
    - `--discard` Discard the EL while muxing. This is equivalent to injecting the RPU, but without extracting first.

    **Examples**:
    ```console
    dovi_tool mux --bl BL.hevc --el EL.hevc
    ```
    ```console
    ffmpeg -i input.mkv -c:v copy -bsf:v hevc_mp4toannexb -f hevc - | dovi_tool mux --bl - --el EL.hevc
    ```

    **Example to convert RPU to profile 8.1 while muxing**:
    ```console
    dovi_tool -m 2 mux --bl BL.hevc --el EL.hevc --discard
    ```

&nbsp;
* ### **extract-rpu**
    Extracts Dolby Vision RPU from an HEVC file.  
    Input file:
    - HEVC bitstream: single track (BL + RPU), single track dual layer (BL+EL+RPU) or an enhancement layer (EL+RPU) video file.
    - Matroska =: MKV file containing a HEVC video track.
 
    **Supports profiles 4, 5, 7, and 8**.

    **Flags**:
    - `-l`, `--limit` Number of frames to process from the input. Processing stops after N frames.

    **Examples**:
    ```console
    dovi_tool extract-rpu video.hevc

    # Directly using MKV file
    dovi_tool extract-rpu video.mkv
    ```
    ```console
    ffmpeg -i input.mkv -c:v copy -bsf:v hevc_mp4toannexb -f hevc - | dovi_tool extract-rpu - -o RPU.bin
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
* ### **remove**
    Removes the enhancement layer and RPU data from the video.  
    Outputs to a `BL.hevc` file by default.

    **Examples**:
    ```console
    dovi_tool remove file.hevc
    ```
    ```console
    ffmpeg -i input.mkv -c:v copy -bsf:v hevc_mp4toannexb -f hevc - | dovi_tool remove -
    ```

&nbsp;

Build artifacts can be found in the Github Actions.  
More features may or may not be added in the future.
