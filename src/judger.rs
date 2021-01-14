use crate::config::{CONFIG, STDERR_FILENAME, STDOUT_FILENAME};
use crate::error::{Error, Result};
use crate::process::{Process, Runner};
use crate::result::{compile_error, compile_success};
use crate::river;
use crate::river::JudgeResponse;
use std::fs;
use std::fs::read_to_string;
use std::path::Path;

#[cfg(test)]
use std::println as debug;

pub async fn compile(language: &str, code: &str, path: &Path) -> Result<JudgeResponse> {
    debug!("language: {}", language);
    debug!("code: {}", code);
    debug!("path: {:?}", path);
    let lang = match CONFIG.languages.get(language) {
        Some(val) => val,
        None => return Err(Error::LanguageNotFound(String::from(language))),
    };
    debug!("write file to {:?}", path.join(&lang.code_file));
    try_io!(fs::write(path.join(&lang.code_file), &code));

    debug!("build command: {}", lang.compile_cmd);
    let process = Process::new(
        String::from(&lang.compile_cmd),
        10000,
        1024 * 1024,
        path.to_path_buf(),
    );
    let result = Runner::from(process)?;
    let status = result.await?;
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
            try_io!(read_to_string(path.join(STDOUT_FILENAME))),
            try_io!(read_to_string(path.join(STDERR_FILENAME))),
        );
        debug!("{}", errmsg);
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
    debug!("language: {}", language);
    debug!("in_file: {}", in_file);
    debug!("out_file: {}", out_file);
    debug!("time_limit: {}", time_limit);
    debug!("memory_limit: {}", memory_limit);
    debug!("judge_type: {}", judge_type);
    debug!("path: {:?}", path);
    Ok(JudgeResponse {
        state: Some(river::judge_response::State::Status(
            river::JudgeStatus::Pending as i32,
        )),
    })
}
