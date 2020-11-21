### Options
`--mode`, `-m` Sets the mode for RPU processing. `--help` for more info

### Commands

#### demux
Rust port of yusesope's python tool. Credits goes to them.  
Demuxes single track dual layer Dolby Vision into Base layer and Enhancement layer files.

* `dovi_tool demux file.hevc`
* `ffmpeg -i input.mkv -c:v copy -vbsf hevc_mp4toannexb -f hevc - | dovi_tool demux -`

#### extract-rpu
Extracts Dolby Vision RPU from an HEVC encoded file.
Supports profiles 5, 7, and 8.  
Input can be piped.

* `dovi_tool extract-rpu video.hevc`
* FEL to MEL example: `dovi_tool -m 1 extract-rpu video.hevc`

More features may or may not be added in the future.