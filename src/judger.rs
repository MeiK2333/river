use super::error::{Error, Result};
use super::process::{Process, ProcessStatus};
use crate::river::judge_request::Language;
use crate::river::judge_response::{JudgeResult, JudgeStatus};
use crate::river::{JudgeRequest, JudgeResponse};
use std::path::Path;
use tokio::fs;

impl JudgeResponse {
    fn new() -> JudgeResponse {
        JudgeResponse {
            time_used: 0,
            memory_used: 0,
            result: JudgeResult::Accepted as i32,
            errno: 0,
            exit_code: 0,
            stdout: "".into(),
            stderr: "".into(),
            errmsg: "".into(),
            status: JudgeStatus::Running as i32,
        }
    }

    fn set_process_status(self: &mut Self, status: &ProcessStatus) {
        self.time_used = status.time_used;
        self.memory_used = status.memory_used;
        self.exit_code = status.exit_code;
        self.errno = status.signal;
        self.stdout = status.stdout.clone();
        self.stderr = status.stderr.clone();
    }
}

pub async fn judger(request: &JudgeRequest) -> Result<JudgeResponse> {
    // TODO
    let mut resp = JudgeResponse::new();
    resp.time_used = request.time_limit.into();
    resp.status = JudgeStatus::Ended as i32;
    return Ok(resp);
}

pub async fn compile(request: &JudgeRequest, path: &Path) -> Result<JudgeResponse> {
    let mut resp = JudgeResponse::new();
    // 写入代码
    let filename = match Language::from_i32(request.language) {
        Some(Language::C) => "main.c",
        Some(Language::Cpp) => "main.cpp",
        Some(Language::Python) => "main.py",
        Some(Language::Rust) => "main.rs",
        Some(Language::Node) => "main.js",
        Some(Language::TypeScript) => "main.ts",
        Some(Language::Go) => "main.go",
        None => return Err(Error::LanguageNotFound(request.language)),
    };
    if let Err(e) = fs::write(path.join(filename), request.code.clone()).await {
        return Err(Error::FileWriteError(e));
    };

    let mut process = Process::new();
    let cmd = match Language::from_i32(request.language) {
        Some(Language::C) => "gcc main.c",
        Some(Language::Cpp) => "g++ main.cpp",
        Some(Language::Python) => "/usr/bin/python3 -m compileall main.py",
        Some(Language::Rust) => "rustc main.rs",
        // 无需编译的语言直接返回
        // TODO: eslint......
        Some(Language::Node) => return Ok(resp),
        Some(Language::TypeScript) => "tsc",
        Some(Language::Go) => "main.go",
        None => return Err(Error::LanguageNotFound(request.language)),
    };
    process.cmd = cmd.to_string();
    process.workdir = path.to_path_buf();
    process.time_limit = request.time_limit;
    process.memory_limit = request.memory_limit;
    process.stdout_file = Some("stdout.txt".to_string());
    process.stderr_file = Some("stderr.txt".to_string());

    let status = process.await?;
    if status.exit_code != 0 {
        resp.set_process_status(&status);
        resp.result = JudgeResult::CompileError as i32;
        resp.status = JudgeStatus::Ended as i32;
    }
    return Ok(resp);
}

pub fn pending() -> JudgeResponse {
    let mut resp = JudgeResponse::new();
    resp.status = JudgeStatus::Pending as i32;
    resp
}

pub fn running() -> JudgeResponse {
    let resp = JudgeResponse::new();
    resp
}

pub fn compiling() -> JudgeResponse {
    let mut resp = JudgeResponse::new();
    resp.status = JudgeStatus::Compiling as i32;
    resp
}
