use handlebars::RenderError;
use libc::strerror;
use std::ffi::{CStr, NulError, OsString};
use std::fmt;
use std::io;
use std::result;
use std::time;
use yaml_rust::ScanError;

#[derive(Debug)]
pub enum Error {
    YamlScanError(ScanError),
    YamlParseError(String),
    LanguageNotFound(String),
    ReadFileError(String, Option<i32>),

    UnknownJudgeType(String),
    PathJoinError,

    StringToCStringError(NulError),
    TemplateRenderError(RenderError),
    LanguageConfigError(String),
    CreateTempDirError(io::Error),
    CloseTempDirError(io::Error),
    CopyFileError(io::Error),
    OsStringToStringError(OsString),
    ForkError(Option<i32>),
    WaitError(Option<i32>),
    SystemTimeError(time::SystemTimeError),
}

pub type Result<T> = result::Result<T, Error>;

pub fn errno_str(errno: Option<i32>) -> String {
    match errno {
        Some(no) => {
            let stre = unsafe { strerror(no) };
            let c_str: &CStr = unsafe { CStr::from_ptr(stre) };
            c_str.to_str().unwrap().to_string()
        }
        _ => "Unknown Error!".to_string(),
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::YamlScanError(ref err) => {
                let _ = write!(f, "YamlScanError: ");
                err.fmt(f)
            }
            Error::YamlParseError(ref err) => write!(f, "YamlParseError: {}", err),
            Error::LanguageNotFound(ref lang) => write!(f, "LanguageNotFound: {}", lang),
            Error::ReadFileError(ref filename, errno) => {
                let reason = errno_str(errno);
                write!(f, "ReadFileError: `{}` {}", filename, reason)
            }
            Error::UnknownJudgeType(ref judge_type) => {
                write!(f, "UnknownJudgeType: {}", judge_type)
            }
            Error::PathJoinError => write!(f, "PathJoinError"),
            Error::ForkError(errno) => {
                let reason = errno_str(errno);
                write!(f, "ForkError: {}", reason)
            }
            Error::WaitError(errno) => {
                let reason = errno_str(errno);
                write!(f, "WaitError: {}", reason)
            }
            _ => write!(f, "{:?}", self),
        }
    }
}
