use std::result;
use yaml_rust::{ScanError};

#[derive(Debug)]
pub enum Error {
    YamlScanError(ScanError),
    YamlParseError(String),
    LanguageNotFound(String),
    ReadFileError(String, Option<i32>),
}

pub type Result<T> = result::Result<T, Error>;
