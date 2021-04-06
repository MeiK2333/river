use std::path::Path;

use tokio::fs;
use tokio::fs::read_to_string;

use crate::config::{CONFIG, CPU_SEMAPHORE, STDERR_FILENAME, STDIN_FILENAME, STDOUT_FILENAME};
use crate::error::{Error, Result};
use crate::result::{
    accepted, compile_error, compile_success, memory_limit_exceeded, runtime_error,
    standard_result, time_limit_exceeded, wrong_answer,
};
use crate::river::{JudgeResponse, JudgeResultEnum, JudgeType};

pub async fn compile(language: &str, code: &str, path: &Path) -> Result<JudgeResponse> {
    info!("compile: language = `{}`", language);
    let lang = match CONFIG.languages.get(language) {
        Some(val) => val,
        None => return Err(Error::LanguageNotFound(String::from(language))),
    };
    try_io!(fs::write(path.join(&lang.code_file), &code).await);

    let semaphore = CPU_SEMAPHORE.clone();
    let permit = semaphore.acquire().await;
    // TODO: run process
    drop(permit);

    Ok(compile_success(0, 0))
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
    // 复制输入文件
    try_io!(fs::copy(data_dir.join(&in_file), path.join(STDIN_FILENAME)).await);

    let lang = match CONFIG.languages.get(language) {
        Some(val) => val,
        None => return Err(Error::LanguageNotFound(String::from(language))),
    };
    // 信号量控制并发
    let semaphore = CPU_SEMAPHORE.clone();
    let permit = semaphore.acquire().await;
    // TODO: run process
    drop(permit);
    Ok(accepted(0, 0))
}
