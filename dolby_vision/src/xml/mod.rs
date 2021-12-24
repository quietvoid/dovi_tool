/// XML metadata parser
mod parser;

#[cfg(test)]
mod tests;

pub use parser::{CmXmlParser, XmlParserOpts};
