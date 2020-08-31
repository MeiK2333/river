use super::error::{Error, Result};
use super::process::Process;
use crate::river::judge_request::Language;
use crate::river::judge_response::{JudgeResult, JudgeStatus};
use crate::river::{JudgeRequest, JudgeResponse};
use std::path::Path;
use tokio::fs;

pub async fn judger(request: &JudgeRequest) -> Result<JudgeResponse> {
    return Ok(JudgeResponse {
        time_used: request.time_limit,
        memory_used: 2,
        result: JudgeResult::Accepted as i32,
        errno: 0,
        exit_code: 0,
        stdout: "stdout".into(),
        stderr: "stderr".into(),
        errmsg: "".into(),
        status: JudgeStatus::Ended as i32,
    });
}

pub async fn compile(request: &JudgeRequest, path: &Path) -> Result<JudgeResponse> {
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

    let process = Process::new();
    let status = process.await?;
    println!("{}", status.exit_code);

    return Ok(JudgeResponse {
        time_used: request.time_limit,
        memory_used: 2,
        result: JudgeResult::Accepted as i32,
        errno: 0,
        exit_code: 0,
        stdout: "stdout".into(),
        stderr: "stderr".into(),
        errmsg: "".into(),
        status: JudgeStatus::Ended as i32,
    });
}

pub fn pending() -> JudgeResponse {
    JudgeResponse {
        time_used: 0,
        memory_used: 0,
        result: JudgeResult::Accepted as i32,
        errno: 0,
        exit_code: 0,
        stdout: "".into(),
        stderr: "".into(),
        errmsg: "".into(),
        status: JudgeStatus::Pending as i32,
    }
}

pub fn running() -> JudgeResponse {
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

pub fn compiling() -> JudgeResponse {
    JudgeResponse {
        time_used: 0,
        memory_used: 0,
        result: JudgeResult::Accepted as i32,
        errno: 0,
        exit_code: 0,
        stdout: "".into(),
        stderr: "".into(),
        errmsg: "".into(),
        status: JudgeStatus::Compiling as i32,
    }
}
