extern crate libc;

use super::error::{Error, Result};
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

pub struct ExitStatus {
    pub rusage: libc::rusage,
    pub status: i32,
    pub signal: i32,
    pub exited: bool,
    pub coredump: bool,
    pub time_used: i64,
    pub real_time_used: u128,
    pub memory_used: i64,
}

#[derive(Debug, Clone)]
pub enum TestCaseResult {
    Accepted,
    CompileError(String),
    WrongAnswer,
    RuntimeError(String),
    SystemError(String),
}

impl fmt::Display for TestCaseResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl TestCaseResult {
    pub fn standard<P: AsRef<Path>>(
        output_file: P,
        answer_file: P,
        exit_status: ExitStatus,
    ) -> Result<TestCaseResult> {
        println!("\n----------------------------------\n");
        println!("time used:\t{}", exit_status.time_used);
        println!("real time used:\t{}", exit_status.real_time_used);
        println!("memory used:\t{}", exit_status.rusage.ru_maxrss);
        println!("status:\t{}", exit_status.status);
        println!("signal:\t{}", exit_status.signal);
        println!("exited:\t{}", exit_status.exited);

        let output_filename = match output_file.as_ref().to_str() {
            Some(val) => val,
            None => "",
        };
        let answer_filename = match answer_file.as_ref().to_str() {
            Some(val) => val,
            None => "",
        };

        let output_file = match File::open(output_filename) {
            Ok(val) => val,
            Err(_) => {
                return Err(Error::OpenFileError(
                    output_filename.to_string(),
                    io::Error::last_os_error().raw_os_error(),
                ))
            }
        };
        let mut output_reader = BufReader::new(output_file);
        let mut output_buffer = String::new();
        let mut output_bytes;

        let answer_file = match File::open(answer_filename) {
            Ok(val) => val,
            Err(_) => {
                return Err(Error::OpenFileError(
                    answer_filename.to_string(),
                    io::Error::last_os_error().raw_os_error(),
                ))
            }
        };
        let mut answer_reader = BufReader::new(answer_file);
        let mut answer_buffer = String::new();
        let mut answer_bytes;

        loop {
            output_bytes = match output_reader.read_line(&mut output_buffer) {
                Ok(val) => val,
                Err(_) => {
                    return Err(Error::ReadFileError(
                        output_filename.to_string(),
                        io::Error::last_os_error().raw_os_error(),
                    ))
                }
            };
            answer_bytes = match answer_reader.read_line(&mut answer_buffer) {
                Ok(val) => val,
                Err(_) => {
                    return Err(Error::ReadFileError(
                        answer_filename.to_string(),
                        io::Error::last_os_error().raw_os_error(),
                    ))
                }
            };
            if output_bytes == 0 || answer_bytes == 0 {
                break;
            }
            if output_buffer.trim_end() != answer_buffer.trim_end() {
                return Ok(TestCaseResult::WrongAnswer);
            }
        }
        // 末尾的空白字符不影响结果
        while output_bytes != 0 {
            if output_buffer.trim() != "" {
                return Ok(TestCaseResult::WrongAnswer);
            }
            output_bytes = match output_reader.read_line(&mut output_buffer) {
                Ok(val) => val,
                Err(_) => {
                    return Err(Error::ReadFileError(
                        output_filename.to_string(),
                        io::Error::last_os_error().raw_os_error(),
                    ))
                }
            };
        }
        while answer_bytes != 0 {
            if answer_buffer.trim() != "" {
                return Ok(TestCaseResult::WrongAnswer);
            }
            answer_bytes = match answer_reader.read_line(&mut answer_buffer) {
                Ok(val) => val,
                Err(_) => {
                    return Err(Error::ReadFileError(
                        answer_filename.to_string(),
                        io::Error::last_os_error().raw_os_error(),
                    ))
                }
            };
        }

        Ok(TestCaseResult::Accepted)
    }
}
