use crate::river::judge_response::State;
use crate::river::JudgeResponse;
use crate::river::JudgeResult;
use libc::strerror;
use std::ffi::{CStr, NulError, OsString};
use std::fmt;
use std::io;
use std::path::PathBuf;
use std::result;

#[allow(dead_code)]
#[derive(Debug)]
pub enum Error {
    CreateTempDirError(io::Error),
    LanguageNotFound(i32),
    FileWriteError(io::Error),
    ChannelRecvError,
    StringToCStringError(NulError),
    OsStringToStringError(OsString),
    RemoveFileError(PathBuf),
    PathBufToStringError(PathBuf),
    UnknownRequestData,
    RequestDataNotFound,
    SyscallError(String),
    CustomError(String),
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
            Error::SyscallError(ref syscall) => {
                let errno = io::Error::last_os_error().raw_os_error();
                let reason = errno_str(errno);
                write!(f, "SyscallError: `{}` {}", syscall, reason)
            }
            Error::CustomError(ref reason) => write!(f, "{}", reason),
            _ => write!(f, "{:?}", self),
        }
    }
}

pub fn system_error(err: Error) -> JudgeResponse {
    JudgeResponse {
        time_used: 0,
        memory_used: 0,
        state: Some(State::Result(JudgeResult::SystemError as i32)),
        errmsg: format!("{}", err).into(),
    }
}
