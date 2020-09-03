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
            errmsg: "".into(),
            status: JudgeStatus::Running as i32,
        }
    }

    fn set_process_status(self: &mut Self, status: &ProcessStatus) {
        self.time_used = status.time_used;
        self.memory_used = status.memory_used;
    }
}

pub async fn judger(request: &JudgeRequest, path: &Path) -> Result<JudgeResponse> {
    let mut resp = JudgeResponse::new();
    let cmd = match Language::from_i32(request.language) {
        Some(Language::C) => "./a.out",
        Some(Language::Cpp) => "./a.out",
        Some(Language::Python) => "/usr/bin/python3 main.py",
        Some(Language::Rust) => "./a.out",
        Some(Language::Node) => "node main.js",
        Some(Language::TypeScript) => "node main.js",
        Some(Language::Go) => "./a.out",
        None => return Err(Error::LanguageNotFound(request.language)),
    };
    let mut process = Process::new();
    process.cmd = cmd.to_string();
    process.workdir = path.to_path_buf();
    process.time_limit = request.time_limit;
    process.memory_limit = request.memory_limit;

    // 写入输入文件
    let in_file = path.join("stdin.txt");
    if let Err(e) = fs::write(in_file.clone(), request.in_data.clone()).await {
        return Err(Error::FileWriteError(e));
    };
    process.stdin_file = Some(in_file.clone());

    let status = process.await?;

    if let Err(_) = fs::remove_file(in_file.clone()).await {
        return Err(Error::RemoveFileError(in_file.clone()));
    }

    resp.status = JudgeStatus::Ended as i32;
    // TODO: 对比答案，检查结果
    if status.exit_code != 0 {
        resp.set_process_status(&status);
        resp.result = JudgeResult::CompileError as i32;
        resp.status = JudgeStatus::Ended as i32;
    }
    Ok(resp)
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
        Some(Language::C) => "/usr/bin/gcc main.c -o a.out -Wall -O2 -std=c99 --static",
        Some(Language::Cpp) => "/usr/bin/g++ main.cpp -O2 -Wall --static -o a.out --std=gnu++17",
        Some(Language::Python) => "/usr/bin/python3 -m compileall main.py",
        Some(Language::Rust) => "/usr/bin/rustc main.rs -o a.out -C opt-level=2",
        // 无需编译的语言直接返回
        // TODO: eslint......
        Some(Language::Node) => return Ok(resp),
        Some(Language::TypeScript) => "/usr/bin/tsc",
        Some(Language::Go) => "/usr/bin/go build -ldflags \"-s -w\" main.go",
        None => return Err(Error::LanguageNotFound(request.language)),
    };
    process.cmd = cmd.to_string();
    process.workdir = path.to_path_buf();
    process.time_limit = request.time_limit;
    process.memory_limit = request.memory_limit;

    let status = process.await?;
    if status.exit_code != 0 {
        resp.set_process_status(&status);
        resp.errmsg = status.stdout;
        resp.result = JudgeResult::CompileError as i32;
        resp.status = JudgeStatus::Ended as i32;
    }
    Ok(resp)
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
