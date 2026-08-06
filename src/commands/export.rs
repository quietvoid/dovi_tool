use std::path::PathBuf;

use clap::{
    Args, ValueHint,
    builder::{EnumValueParser, PossibleValue, TypedValueParser},
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
        value_parser = ExportDataOptionParser,
        value_delimiter = ','
    )]
    pub data: Vec<(ExportData, Option<PathBuf>)>,

    #[arg(
        id = "levels-format",
        help = "Format to output levels exports",
        long,
        short = 'f',
        default_value = "csv"
    )]
    pub levels_format: LevelsOutputFormat,

    #[arg(
        id = "levels",
        help = "List of key-value export parameters formatted as `key=output`, where `output` is an output file path.\nSupports multiple occurences prefixed by --levels or delimited by ','",
        long,
        short = 'l',
        value_parser = ExportLevelsOptionParser,
        value_delimiter = ','
    )]
    pub levels: Vec<(ExportLevel, Option<PathBuf>)>,

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

#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LevelsOutputFormat {
    #[default]
    Csv,
    Json,
}

#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportLevel {
    #[value(alias = "l1")]
    Level1,
    #[value(alias = "l2")]
    Level2,
    #[value(alias = "l3")]
    Level3,
    #[value(alias = "l4")]
    Level4,
    #[value(alias = "l5")]
    Level5,
    #[value(alias = "l6")]
    Level6,
    #[value(alias = "l8")]
    Level8,
    #[value(alias = "l9")]
    Level9,
    #[value(alias = "l10")]
    Level10,
    #[value(alias = "l11")]
    Level11,
    #[value(alias = "l15")]
    Level15,
    #[value(alias = "l16")]
    Level16,
    #[value(alias = "l17")]
    Level17,
    #[value(alias = "l18")]
    Level18,
}

impl ExportData {
    pub const fn default_output_file(&self) -> &'static str {
        match self {
            Self::All => "RPU_export.json",
            Self::Scenes => "RPU_scenes.txt",
            Self::Level5 => "RPU_L5_edit_config.json",
        }
    }
}

impl ExportLevel {
    pub const fn level(&self) -> u8 {
        match self {
            Self::Level1 => 1,
            Self::Level2 => 2,
            Self::Level3 => 3,
            Self::Level4 => 4,
            Self::Level5 => 5,
            Self::Level6 => 6,
            Self::Level8 => 8,
            Self::Level9 => 9,
            Self::Level10 => 10,
            Self::Level11 => 11,
            Self::Level15 => 15,
            Self::Level16 => 16,
            Self::Level17 => 17,
            Self::Level18 => 18,
        }
    }
    pub fn default_output_file(&self, format: LevelsOutputFormat) -> String {
        let level = self.level();
        let ext = format.ext();

        format!("L{level}_export.{ext}")
    }
}

impl LevelsOutputFormat {
    pub const fn ext(&self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Csv => "csv",
        }
    }
}

#[derive(Clone)]
pub struct ExportDataOptionParser;

#[derive(Clone)]
pub struct ExportLevelsOptionParser;

impl TypedValueParser for ExportDataOptionParser {
    type Value = (ExportData, Option<PathBuf>);

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        export_option_parse_ref(cmd, arg, value)
    }

    fn possible_values(&self) -> Option<Box<dyn Iterator<Item = PossibleValue> + '_>> {
        export_option_possible_values::<ExportData>()
    }
}

impl TypedValueParser for ExportLevelsOptionParser {
    type Value = (ExportLevel, Option<PathBuf>);

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        export_option_parse_ref(cmd, arg, value)
    }

    fn possible_values(&self) -> Option<Box<dyn Iterator<Item = PossibleValue> + '_>> {
        export_option_possible_values::<ExportLevel>()
    }
}

fn export_option_parse_ref<E>(
    cmd: &clap::Command,
    arg: Option<&clap::Arg>,
    value: &std::ffi::OsStr,
) -> Result<(E, Option<PathBuf>), clap::Error>
where
    E: clap::ValueEnum + Clone + Send + Sync + 'static,
{
    let data_parser = EnumValueParser::<E>::new();

    if let Some((data_str, output_str)) = value.split_once("=") {
        Ok((
            data_parser.parse_ref(cmd, arg, data_str)?,
            output_str.to_str().map(str::parse).and_then(Result::ok),
        ))
    } else {
        Ok((data_parser.parse_ref(cmd, arg, value)?, None))
    }
}

fn export_option_possible_values<E>() -> Option<Box<dyn Iterator<Item = PossibleValue>>>
where
    E: clap::ValueEnum + Clone + Send + Sync + 'static,
{
    Some(Box::new(
        E::value_variants()
            .iter()
            .filter_map(|v| v.to_possible_value()),
    ))
}
