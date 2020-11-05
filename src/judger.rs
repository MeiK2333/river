use super::config::{STDERR_FILENAME, STDOUT_FILENAME};
use super::error::{Error, Result};
use super::process::Process;
use super::runner::RunnerStatus;
use crate::result::standard_result;
use crate::river::judge_response::State;
use crate::river::Language;
use crate::river::{CompileData, JudgeData, JudgeResult, JudgeStatus};
use crate::river::{JudgeRequest, JudgeResponse};
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncReadExt;

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
    // TODO: 使用配置文件
    let cmd = match Language::from_i32(request.language) {
        Some(Language::C) => "./a.out",
        Some(Language::Cpp) => "./a.out",
        Some(Language::Python) => "/usr/bin/python3.8 main.py",
        Some(Language::Rust) => "./a.out",
        Some(Language::Node) => "/usr/bin/node main.js",
        Some(Language::TypeScript) => "/usr/bin/node main.js",
        Some(Language::Go) => "./a.out",
        Some(Language::Java) => "/usr/bin/java -cp . Main",
        None => return Err(Error::LanguageNotFound(request.language)),
    };
    let process = Process::new(
        cmd.to_string(),
        path.to_path_buf(),
        &data.in_data,
        data.time_limit,
        data.memory_limit,
    )?;
    debug!("run command: {}", cmd);

    // 开始执行并等待返回结果
    let mut runner = process.runner.clone();
    // 为 Java 虚拟机取消内存限制和 trace（万恶的 JVM）
    // 看起来虚拟机语言都有同样的问题
    if request.language == Language::Java as i32
        || request.language == Language::Go as i32
        || request.language == Language::Node as i32
    {
        runner.memory_limit = -1;
        runner.traceme = false;
    }
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
        // 没有 ole 这种操作，之前 ole 就是错的
        let mut file = match fs::File::open(path.join(STDOUT_FILENAME)).await {
            Ok(val) => val,
            Err(e) => return Err(Error::OpenFileError(path.join(STDOUT_FILENAME), e)),
        };
        let mut out = Vec::new();
        if let Err(e) = file.read_to_end(&mut out).await {
            return Err(Error::ReadFileError(path.join(STDOUT_FILENAME), e));
        };
        let result = standard_result(&out, &data.out_data)?;
        resp.state = Some(State::Result(result as i32));
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
        Some(Language::Java) => "Main.java",
        None => return Err(Error::LanguageNotFound(request.language)),
    };
    if let Err(e) = fs::write(path.join(filename), &data.code).await {
        return Err(Error::FileWriteError(e));
    };

    // TODO: 使用配置文件
    let cmd = match Language::from_i32(request.language) {
        Some(Language::C) => "/usr/bin/gcc main.c -o a.out -Wall -O2 -std=c99 --static",
        Some(Language::Cpp) => "/usr/bin/g++ main.cpp -O2 -Wall --static -o a.out --std=gnu++17",
        Some(Language::Python) => "/usr/bin/python3.8 -m compileall main.py",
        Some(Language::Rust) => "/root/.cargo/bin/rustc main.rs -o a.out -C opt-level=2",
        Some(Language::Node) => "/usr/bin/node /plugins/js/validate.js main.js",
        Some(Language::TypeScript) => "/usr/bin/tsc main.ts",
        Some(Language::Go) => "/usr/bin/go build -o a.out main.go",
        Some(Language::Java) => "/usr/bin/javac Main.java",
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
        1024 * 1024,
    )?;

    let mut runner = process.runner.clone();
    runner.traceme = false;
    // 为 Java 虚拟机取消内存限制（万恶的 JVM）
    if request.language == Language::Java as i32 || request.language == Language::Go as i32 {
        runner.memory_limit = -1;
    }
    if request.language == Language::Rust as i32 {
        // https://github.com/rust-lang/rust/issues/46345
        runner.memory_limit = -1;
    }
    let status = runner.await?;
    resp.set_process_status(&status);
    if status.exit_code != 0 || status.signal != 0 {
        debug!("compile exit code: {}", status.exit_code);
        debug!("compile signal: {}", status.signal);
        // 从 stdout 和 stderr 中获取错误信息
        resp.stdout = match fs::read_to_string(path.join(STDOUT_FILENAME)).await {
            Ok(val) => val,
            Err(e) => return Err(Error::FileReadError(e)),
        };
        resp.stderr = match fs::read_to_string(path.join(STDERR_FILENAME)).await {
            Ok(val) => val,
            Err(e) => return Err(Error::FileReadError(e)),
        };
        debug!("stdout: {}", resp.stdout);
        debug!("stderr: {}", resp.stderr);
        debug!("errmsg: {}", status.errmsg);
        resp.state = Some(State::Result(JudgeResult::CompileError as i32));
    } else {
        resp.state = Some(State::Result(JudgeResult::CompileSuccess as i32));
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
