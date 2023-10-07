use std::path::PathBuf;

use clap::{
    builder::{EnumValueParser, PossibleValue, TypedValueParser},
    Args, ValueEnum, ValueHint,
};
use clap_lex::OsStrExt as _;

#[derive(Args, Debug)]
pub struct ExportArgs {
    #[arg(
        id = "input",
        help = "Sets the input RPU file to use",
        long,
        short = 'i',
        conflicts_with = "input_pos",
        required_unless_present = "input_pos",
        value_hint = ValueHint::FilePath,
    )]
    pub input: Option<PathBuf>,

    #[arg(
        id = "input_pos",
        help = "Sets the input RPU file to use (positional)",
        conflicts_with = "input",
        required_unless_present = "input",
        value_hint = ValueHint::FilePath
    )]
    pub input_pos: Option<PathBuf>,

    #[arg(
        id = "data",
        help = "List of key-value export parameters formatted as `key=output`, where `output` is an output file path.\nSupports multiple occurences prefixed by --data or delimited by ','",
        long,
        short = 'd',
        conflicts_with = "output",
        value_parser = ExportOptionParser,
        value_delimiter = ','
    )]
    pub data: Vec<(ExportData, Option<PathBuf>)>,

    // FIXME: export single output deprecation
    #[arg(
        id = "output",
        help = "Output JSON file name. Deprecated, replaced by `--data all=output`",
        long,
        short = 'o',
        conflicts_with = "data",
        hide = true,
        value_hint = ValueHint::FilePath
    )]
    pub output: Option<PathBuf>,
}

#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportData {
    /// Exports the list of RPUs as a JSON file
    All,
    /// Exports the frame indices at which `scene_refresh_flag` is set to 1
    Scenes,
    /// Exports the video's L5 metadata in the form of an `editor` config JSON
    Level5,
}

impl ExportData {
    pub fn default_output_file(&self) -> &'static str {
        match self {
            ExportData::All => "RPU_export.json",
            ExportData::Scenes => "RPU_scenes.txt",
            ExportData::Level5 => "RPU_L5_edit_config.json",
        }
    }
}

#[derive(Clone)]
struct ExportOptionParser;
impl TypedValueParser for ExportOptionParser {
    type Value = (ExportData, Option<PathBuf>);

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let data_parser = EnumValueParser::<ExportData>::new();

        if let Some((data_str, output_str)) = value.split_once("=") {
            Ok((
                data_parser.parse_ref(cmd, arg, data_str)?,
                output_str.to_str().map(str::parse).and_then(Result::ok),
            ))
        } else {
            Ok((data_parser.parse_ref(cmd, arg, value)?, None))
        }
    }

    fn possible_values(&self) -> Option<Box<dyn Iterator<Item = PossibleValue> + '_>> {
        Some(Box::new(
            ExportData::value_variants()
                .iter()
                .filter_map(|v| v.to_possible_value()),
        ))
    }
}
