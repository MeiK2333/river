use libc::strerror;
use std::ffi::CStr;
use std::fmt;
use std::result;
use yaml_rust::ScanError;

#[derive(Debug)]
pub enum Error {
    YamlScanError(ScanError),
    YamlParseError(String),
    LanguageNotFound(String),
    ReadFileError(String, Option<i32>),
}

pub type Result<T> = result::Result<T, Error>;

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
                let reason = match errno {
                    Some(no) => {
                        let stre = unsafe { strerror(no) };
                        let c_str: &CStr = unsafe { CStr::from_ptr(stre) };
                        c_str.to_str().unwrap()
                    }
                    _ => "Unknown Error!",
                };
                write!(f, "ReadFileError: `{}` {}", filename, reason)
            }
        }
    }
}
