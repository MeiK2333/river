use crate::config::{CONFIG, STDERR_FILENAME, STDIN_FILENAME, STDOUT_FILENAME, CPU_SEMAPHORE};
use crate::error::{Error, Result};
use crate::process::{Process, Runner};
use crate::result::{
    accepted, compile_error, compile_success, memory_limit_exceeded, runtime_error,
    standard_result, time_limit_exceeded, wrong_answer,
};
use crate::river::{JudgeResponse, JudgeResultEnum, JudgeType};
use std::path::Path;
use tokio::fs;
use tokio::fs::read_to_string;

pub async fn compile(language: &str, code: &str, path: &Path) -> Result<JudgeResponse> {
    let lang = match CONFIG.languages.get(language) {
        Some(val) => val,
        None => return Err(Error::LanguageNotFound(String::from(language))),
    };
    try_io!(fs::write(path.join(&lang.code_file), &code).await);

    let process = Process::new(
        String::from(&lang.compile_cmd),
        10000,
        1024 * 1024,
        path.to_path_buf(),
    );
    let semaphore = CPU_SEMAPHORE.clone();
    let permit = semaphore.acquire().await;
    let result = Runner::from(process)?;
    let status = result.await?;
    drop(permit);
    let mem_used = if status.memory_used < status.cgroup_memory_used {
        status.memory_used
    } else {
        status.cgroup_memory_used
    };
    if status.exit_code != 0 || status.signal != 0 {
        // 合并 stdout 与 stderr 为 errmsg
        // 因为不同的语言、不同的编译器，错误信息输出到了不同的地方
        let errmsg = format!(
            "{}\n{}",
            try_io!(read_to_string(path.join(STDOUT_FILENAME)).await),
            try_io!(read_to_string(path.join(STDERR_FILENAME)).await),
        );
        return Ok(compile_error(status.time_used, mem_used, &errmsg));
    } else {
        return Ok(compile_success(status.time_used, mem_used));
    }
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
    let data_dir = Path::new(&CONFIG.data_dir);
    // 复制输入文件
    try_io!(fs::copy(data_dir.join(&in_file), path.join(STDIN_FILENAME)).await);

    let lang = match CONFIG.languages.get(language) {
        Some(val) => val,
        None => return Err(Error::LanguageNotFound(String::from(language))),
    };
    let process = Process::new(
        String::from(&lang.run_cmd),
        time_limit,
        memory_limit,
        path.to_path_buf(),
    );
    // 信号量控制并发
    let semaphore = CPU_SEMAPHORE.clone();
    let permit = semaphore.acquire().await;
    let result = Runner::from(process)?;
    let status = result.await?;
    drop(permit);
    // 为了更好的体验，此处内存用量取 getrusage 报告与 cgroup 报告中较小的一个
    let mem_used = if status.memory_used < status.cgroup_memory_used {
        status.memory_used
    } else {
        status.cgroup_memory_used
    };
    if status.time_used > time_limit.into() || status.real_time_used as i64 > time_limit.into() {
        // TLE
        return Ok(time_limit_exceeded(status.time_used, mem_used));
    } else if mem_used > memory_limit.into() {
        // MLE
        return Ok(memory_limit_exceeded(status.time_used, mem_used));
    } else if status.signal != 0 {
        // RE
        return Ok(runtime_error(
            status.time_used,
            mem_used,
            &format!("Program was interrupted by signal: `{}`", status.signal),
        ));
    } else if status.exit_code != 0 {
        // RE
        // 就算是用户自己返回的非零，也算 RE
        return Ok(runtime_error(
            status.time_used,
            mem_used,
            &format!("Exceptional program return code: `{}`", status.exit_code),
        ));
    } else if judge_type == JudgeType::Standard as i32 {
        // 答案对比
        let out = try_io!(fs::read(path.join(STDOUT_FILENAME)).await);
        let ans = try_io!(fs::read(data_dir.join(&out_file)).await);
        let res = standard_result(&out, &ans)?;
        if res == JudgeResultEnum::Accepted {
            return Ok(accepted(status.time_used, mem_used));
        } else {
            return Ok(wrong_answer(status.time_used, mem_used));
        }
    }
    Err(Error::CustomError(String::from(format!(
        "Unknown JudgeType: {}",
        judge_type
    ))))
}
