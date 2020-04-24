extern crate libc;

use std::ptr;
use super::judger::{JudgeConfig};
use super::result::{ResourceUsed, TestCaseResult};

pub struct ExecArgs {
    pub pathname: *const libc::c_char,
    pub argv: *const *const libc::c_char,
    pub envp: *const *const libc::c_char,
}

impl ExecArgs {
    // TODO: 从配置文件中生成
}

impl Drop for ExecArgs {
    fn drop(&mut self) {
        println!("Dropping!");
    }
}

pub fn run(judge_config: &mut JudgeConfig) {
    let resource = ResourceUsed {
        time_used: 1024,
        memory_used: 65535,
    };
    for test_case in &mut judge_config.tests {
        let _exec_args = ExecArgs {
            pathname: ptr::null(),
            argv: ptr::null(),
            envp: ptr::null()
        };
        run_one();
        test_case.result = Some(TestCaseResult::CompileError(
            resource,
            "Compile Error!".to_string(),
        ));
        test_case.result = Some(TestCaseResult::WrongAnswer(resource));
        test_case.result = Some(TestCaseResult::RuntimeError(
            resource,
            "Runtime Error!".to_string(),
        ));
        test_case.result = Some(TestCaseResult::SystemError(
            resource,
            "System Error!".to_string(),
        ));
        test_case.result = Some(TestCaseResult::Accepted(resource));
        println!("{}", test_case);
    }
}

pub fn run_one() {
    println!("Hello World!");
}
