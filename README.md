## Rust port of yusesope's python tool.
## Credits goes to them.

## Commands

### Demux
Demuxes single track dual layer Dolby Vision into Base layer and Enhancement layer files.

* `dovi_tool demux file.hevc`
* `ffmpeg -i input.mkv -c:v copy -vbsf hevc_mp4toannexb -f hevc - | dovi_tool demux -`

More features may or may not be added in the future.