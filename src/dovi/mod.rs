pub mod demuxer;
mod rpu;

#[derive(Debug, PartialEq)]
pub enum Format {
    Raw,
    RawStdin,
    Matroska,
}
