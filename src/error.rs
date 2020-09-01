use crate::river::judge_response::{JudgeResult, JudgeStatus};
use crate::river::JudgeResponse;
use libc::strerror;
use std::ffi::{CStr, NulError, OsString};
use std::fmt;
use std::io;
use std::result;

#[derive(Debug)]
pub enum Error {
    CreateTempDirError(io::Error),
    LanguageNotFound(i32),
    FileWriteError(io::Error),
    ChannelRecvError,
    StringToCStringError(NulError),
    OsStringToStringError(OsString),
}

pub type Result<T> = result::Result<T, Error>;

pub fn _errno_str(errno: Option<i32>) -> String {
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
            _ => write!(f, "{:?}", self),
        }
    }
}

pub fn system_error(err: Error) -> JudgeResponse {
    JudgeResponse {
        time_used: 0,
        memory_used: 0,
        result: JudgeResult::SystemError as i32,
        errno: 0,
        exit_code: 0,
        stdout: "".into(),
        stderr: "".into(),
        errmsg: format!("{}", err).into(),
        status: JudgeStatus::Ended as i32,
    }
}
