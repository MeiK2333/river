use std::path::Path;

use tokio::fs;
use tokio::fs::{read_to_string, remove_file};

use crate::config::{CONFIG, CPU_SEMAPHORE, RESULT_FILENAME, STDERR_FILENAME, STDOUT_FILENAME};
use crate::error::{Error, Result};
use crate::result::{
    accepted, compile_error, compile_success, memory_limit_exceeded, runtime_error,
    standard_result, time_limit_exceeded, wrong_answer,
};
use crate::river::{JudgeResponse, JudgeResultEnum, JudgeType};
use crate::sandbox::Sandbox;

fn path_to_string(path: &Path) -> Result<String> {
    if let Some(s) = path.to_str() {
        return Ok(String::from(s));
    }
    Err(Error::PathToStringError())
}

pub async fn compile(language: &str, code: &str, path: &Path) -> Result<JudgeResponse> {
    info!("compile: language = `{}`", language);
    let lang = match CONFIG.languages.get(language) {
        Some(val) => val,
        None => return Err(Error::LanguageNotFound(String::from(language))),
    };
    try_io!(fs::write(path.join(&lang.code_file), &code).await);

    let semaphore = CPU_SEMAPHORE.clone();
    let permit = semaphore.acquire().await;

    let mut sandbox = Sandbox::new(
        &lang.compile_cmd,
        path_to_string(&path)?,
        String::from(&CONFIG.rootfs),
        path_to_string(&path.join(RESULT_FILENAME))?,
        String::from("/STDIN/"),
        path_to_string(&path.join(STDOUT_FILENAME))?,
        path_to_string(&path.join(STDERR_FILENAME))?,
        5000,
        655350,
        50 * 1024 * 1024,
        i32::from(CONFIG.cgroup),
        10,
    );
    let status = sandbox.spawn().await?;
    drop(permit);
    info!("status = {:?}", status);

    if status.exit_code != 0 || status.signal != 0 {
        // 合并 stdout 与 stderr 为 errmsg
        // 因为不同的语言、不同的编译器，错误信息输出到了不同的地方
        let output = try_io!(read_to_string(path.join(STDOUT_FILENAME)).await);
        let error = try_io!(read_to_string(path.join(STDERR_FILENAME)).await);
        let errmsg = format!(
            "{}\n{}",
            &output[..1024],
            &error[..1024],
        );
        return Ok(compile_error(status.time_used, status.memory_used, &errmsg));
    }
    Ok(compile_success(status.time_used, status.memory_used))
}

pub async fn judge(
    language: &str,
    in_file: &str,
    out_file: &str,
    time_limit: i32,
    memory_limit: i32,
    judge_type: i32,
    path: &Path,
) -> Result<JudgeResponse> {
    info!("judge: language = `{}`, in_file = `{}`, out_file = `{}`, time_limit = `{}`,  memory_limit = `{}`, judge_type = `{}`", language, in_file, out_file, time_limit, memory_limit, judge_type);
    let data_dir = Path::new(&CONFIG.data_dir);

    let lang = match CONFIG.languages.get(language) {
        Some(val) => val,
        None => return Err(Error::LanguageNotFound(String::from(language))),
    };
    // 信号量控制并发
    let semaphore = CPU_SEMAPHORE.clone();
    let permit = semaphore.acquire().await;

    try_io!(remove_file(&path.join(RESULT_FILENAME)).await);
    try_io!(remove_file(&path.join(STDOUT_FILENAME)).await);
    try_io!(remove_file(&path.join(STDERR_FILENAME)).await);
    let mut sandbox = Sandbox::new(
        &lang.run_cmd,
        path_to_string(&path)?,
        String::from(&CONFIG.rootfs),
        path_to_string(&path.join(RESULT_FILENAME))?,
        path_to_string(data_dir.join(&in_file).as_path())?,
        path_to_string(&path.join(STDOUT_FILENAME))?,
        path_to_string(&path.join(STDERR_FILENAME))?,
        time_limit,
        memory_limit,
        50 * 1024 * 1024,
        i32::from(CONFIG.cgroup),
        2,
    );
    let status = sandbox.spawn().await?;
    drop(permit);

    if status.time_used > time_limit.into() {
        // TLE
        return Ok(time_limit_exceeded(status.time_used, status.memory_used));
    } else if status.memory_used > memory_limit.into() {
        // MLE
        return Ok(memory_limit_exceeded(status.time_used, status.memory_used));
    } else if status.signal != 0 {
        // RE
        return Ok(runtime_error(
            status.time_used,
            status.memory_used,
            &format!("Program was interrupted by signal: `{}`", status.signal),
        ));
    } else if status.exit_code != 0 {
        // RE
        // 就算是用户自己返回的非零，也算 RE
        return Ok(runtime_error(
            status.time_used,
            status.memory_used,
            &format!("Exceptional program return code: `{}`", status.exit_code),
        ));
    } else if judge_type == JudgeType::Standard as i32 {
        // 答案对比
        let out = try_io!(fs::read(path.join(STDOUT_FILENAME)).await);
        let ans = try_io!(fs::read(data_dir.join(&out_file)).await);
        let res = standard_result(&out, &ans)?;
        return if res == JudgeResultEnum::Accepted {
            Ok(accepted(status.time_used, status.memory_used))
        } else {
            Ok(wrong_answer(status.time_used, status.memory_used))
        };
    }

    Err(Error::SystemError(String::from(format!("Unknown Error!"))))
}
