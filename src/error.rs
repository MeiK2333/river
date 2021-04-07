#![macro_use]

use std::ffi::CStr;
use std::ffi::{NulError, OsString};
use std::fmt;
use std::io;
use std::result;

use libc::strerror;

#[allow(dead_code)]
#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    StringToCStringError(NulError),
    ParseIntError(std::num::ParseIntError),
    CreateTempDirError(io::Error),
    CustomError(String),
    LanguageNotFound(String),
    SystemError(String),
    OsStringToStringError(OsString),
    PathToStringError(),
    StringSplitError(),
    StringToIntError(String),
}

pub type Result<T> = result::Result<T, Error>;

// 创建一个简单的包装
#[macro_export]
macro_rules! try_io {
    ($expression:expr) => {
        match $expression {
            Ok(val) => val,
            Err(e) => return Err(crate::error::Error::IOError(e)),
        };
    };
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::IOError(ref e) => write!(f, "IOError: `{}`", errno_str(e.raw_os_error())),
            Error::CustomError(ref e) => write!(f, "Internal Server Error: `{}`", e),
            Error::LanguageNotFound(ref e) => write!(f, "Language Not Fount: `{}`", e),
            Error::SystemError(ref e) => write!(f, "System Error: `{}`", e),
            _ => write!(f, "{:?}", self),
        }
    }
}

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
