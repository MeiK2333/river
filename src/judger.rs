use super::config::{STDERR_FILENAME, STDOUT_FILENAME};
use super::error::{Error, Result};
use super::process::Process;
use super::runner::RunnerStatus;
use crate::river::judge_response::State;
use crate::river::Language;
use crate::river::{CompileData, JudgeData, JudgeResult, JudgeStatus};
use crate::river::{JudgeRequest, JudgeResponse};
use std::path::Path;
use tokio::fs;

impl JudgeResponse {
    fn new() -> JudgeResponse {
        JudgeResponse {
            time_used: 0,
            memory_used: 0,
            errmsg: "".into(),
            stdout: "".into(),
            stderr: "".into(),
            state: Some(State::Status(JudgeStatus::Running as i32)),
        }
    }

    fn set_process_status(self: &mut Self, status: &RunnerStatus) {
        self.time_used = status.time_used;
        self.memory_used = status.memory_used;
    }
}

pub async fn judger(
    request: &JudgeRequest,
    data: &JudgeData,
    path: &Path,
) -> Result<JudgeResponse> {
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
    let process = Process::new(
        cmd.to_string(),
        path.to_path_buf(),
        &data.in_data,
        data.time_limit,
        data.memory_limit,
    )?;

    // 开始执行并等待返回结果
    let runner = process.runner.clone();
    let status = runner.await?;

    resp.set_process_status(&status);
    // 先判断 tle 和 mle，一是因为 tle 和 mle 也会导致信号中断，二是优先程序复杂度判断
    if status.time_used > data.time_limit.into() {
        // TLE
        resp.state = Some(State::Result(JudgeResult::TimeLimitExceeded as i32));
    } else if status.memory_used > data.memory_limit.into() {
        // MLE
        resp.state = Some(State::Result(JudgeResult::MemoryLimitExceeded as i32));
    } else if status.signal != 0 {
        // 被信号中断的程序，RE
        resp.stderr = match fs::read_to_string(path.join(STDERR_FILENAME)).await {
            Ok(val) => val,
            Err(e) => return Err(Error::FileReadError(e)),
        };
        resp.errmsg = format!("Program was interrupted by signal: {}", status.signal);
        resp.state = Some(State::Result(JudgeResult::RuntimeError as i32));
    } else if status.exit_code != 0 {
        // 返回值不为 0 的程序，RE（虽然有可能是用户自己返回的）
        resp.errmsg = format!("Exceptional program return code: {}", status.exit_code);
        resp.state = Some(State::Result(JudgeResult::RuntimeError as i32));
    } else {
        // TODO: AC\WA\PE
        // 没有 ole 这种操作，之前 ole 就是错的
        resp.state = Some(State::Result(JudgeResult::Accepted as i32));
    }
    Ok(resp)
}

pub async fn compile(
    request: &JudgeRequest,
    data: &CompileData,
    path: &Path,
) -> Result<JudgeResponse> {
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
    if let Err(e) = fs::write(path.join(filename), &data.code).await {
        return Err(Error::FileWriteError(e));
    };

    // TODO: 使用配置文件
    let cmd = match Language::from_i32(request.language) {
        Some(Language::C) => "/usr/bin/gcc main.c -o a.out -Wall -O2 -std=c99 --static",
        Some(Language::Cpp) => "/usr/bin/g++ main.cpp -O2 -Wall --static -o a.out --std=gnu++17",
        Some(Language::Python) => "/usr/bin/python3 -m compileall main.py",
        Some(Language::Rust) => "/usr/bin/rustc main.rs -o a.out -C opt-level=2",
        // TODO: eslint......
        Some(Language::Node) => "/bin/echo hello",
        Some(Language::TypeScript) => "/usr/bin/tsc",
        Some(Language::Go) => "/usr/bin/go build -ldflags \"-s -w\" main.go",
        None => return Err(Error::LanguageNotFound(request.language)),
    };
    debug!("build command: {}", cmd);
    let v = vec![];
    let process = Process::new(
        cmd.to_string(),
        path.to_path_buf(),
        &v,
        // 编译的资源限制为固定的
        10000,
        64 * 1024,
    )?;

    let runner = process.runner.clone();
    let status = runner.await?;
    resp.set_process_status(&status);
    if status.exit_code != 0 || status.signal != 0 {
        debug!("compile exit code: {}", status.exit_code);
        debug!("compile signal: {}", status.exit_code);
        // 从 stdout 和 stderr 中获取错误信息
        resp.stdout = match fs::read_to_string(path.join(STDOUT_FILENAME)).await {
            Ok(val) => val,
            Err(e) => return Err(Error::FileReadError(e)),
        };
        resp.stderr = match fs::read_to_string(path.join(STDERR_FILENAME)).await {
            Ok(val) => val,
            Err(e) => return Err(Error::FileReadError(e)),
        };
        resp.state = Some(State::Result(JudgeResult::CompileError as i32));
    } else {
        resp.state = Some(State::Result(JudgeResult::Accepted as i32));
    }
    Ok(resp)
}

pub fn pending() -> JudgeResponse {
    let mut resp = JudgeResponse::new();
    resp.state = Some(State::Status(JudgeStatus::Pending as i32));
    resp
}

pub fn running() -> JudgeResponse {
    let resp = JudgeResponse::new();
    resp
}
