use std::path::{Path, PathBuf};

use tokio::fs;
use tokio::fs::{remove_file, File};
use tokio::io::AsyncReadExt;

use crate::config::{
    CONFIG, CPU_SEMAPHORE, RESULT_FILENAME, SPJ_ANSWER_FILENAME, SPJ_FILENAME, SPJ_INPUT_FILENAME,
    SPJ_RESULT_FILENAME, SPJ_STDERR_FILENAME, SPJ_STDOUT_FILENAME, STDERR_FILENAME,
    STDOUT_FILENAME,
};
use crate::error::{Error, Result};
use crate::result::{
    accepted, compile_error, compile_success, memory_limit_exceeded, runtime_error, spj_result,
    standard_result, time_limit_exceeded, wrong_answer,
};
use crate::river::{JudgeResponse, JudgeResultEnum, JudgeType};
use crate::sandbox::{ProcessExitStatus, Sandbox};

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
        8000,
        1024 * 1024,
        50 * 1024 * 1024,
        i32::from(CONFIG.cgroup),
        64,
    );
    let status = sandbox.spawn().await?;
    drop(permit);
    info!("status = {:?}", status);

    if status.exit_code != 0 || status.signal != 0 {
        // 合并 stdout 与 stderr 为 errmsg
        // 因为不同的语言、不同的编译器，错误信息输出到了不同的地方
        let outmsg = read_file_2048(path.join(STDOUT_FILENAME)).await?;
        let errmsg = read_file_2048(path.join(STDERR_FILENAME)).await?;
        let errmsg = if outmsg == "" {
            errmsg
        } else if errmsg == "" {
            outmsg
        } else {
            format!("{}\n{}", outmsg, errmsg)
        };
        return Ok(compile_error(status.time_used, status.memory_used, &errmsg));
    }
    Ok(compile_success(status.time_used, status.memory_used))
}

pub async fn judge(
    language: &str,
    in_file: &str,
    out_file: &str,
    spj_file: &str,
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
        if language == "Java"
            || language == "Go"
            || language == "JavaScript"
            || language == "TypeScript"
            || language == "CSharp"
        {
            1024 * 1024
        } else {
            memory_limit
        },
        50 * 1024 * 1024,
        i32::from(CONFIG.cgroup),
        32,
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
    } else if judge_type == JudgeType::Special as i32 {
        // Special Judge
        return special_judge(in_file, out_file, spj_file, path, data_dir, status).await;
    }

    Err(Error::SystemError(String::from(format!("Unknown Error!"))))
}

async fn special_judge(
    in_file: &str,
    out_file: &str,
    spj_file: &str,
    path: &Path,
    data_dir: &Path,
    status: ProcessExitStatus,
) -> Result<JudgeResponse> {
    if spj_file == "" {
        return Err(Error::SystemError(format!("field spj_file is required!")));
    }
    let spj = data_dir.join(&spj_file);
    if !spj.exists() {
        return Err(Error::SystemError(format!(
            "Special Judge File `{}` Not Found!",
            spj_file
        )));
    }
    // 将 spj 程序复制到沙盒内部
    try_io!(fs::copy(spj, path.join(SPJ_FILENAME)).await);

    // TODO: 创建 input file 与 answer file 的 named pipe，穿透沙盒以文件形式传递给 spj 程序？
    // 此方案不稳定因素较多，比如两个阻塞写入的线程、异常处理等。在没有明显性能问题前先不实现此方案

    // 将 input file 与 answer file 复制到沙盒内部，以供 spj 使用
    try_io!(fs::copy(data_dir.join(&in_file), path.join(SPJ_INPUT_FILENAME)).await);
    try_io!(fs::copy(data_dir.join(&out_file), path.join(SPJ_ANSWER_FILENAME)).await);

    // Program must be run with the following arguments: <input-file> <output-file> <answer-file>
    let spj_cmd = format!(
        "{} {} {} {}",
        SPJ_FILENAME, SPJ_INPUT_FILENAME, STDOUT_FILENAME, SPJ_ANSWER_FILENAME
    );

    let semaphore = CPU_SEMAPHORE.clone();
    let permit = semaphore.acquire().await;

    let mut sandbox = Sandbox::new(
        &spj_cmd,
        path_to_string(&path)?,
        String::from(&CONFIG.rootfs),
        path_to_string(&path.join(SPJ_RESULT_FILENAME))?,
        String::from("/STDIN/"),
        path_to_string(&path.join(SPJ_STDOUT_FILENAME))?,
        path_to_string(&path.join(SPJ_STDERR_FILENAME))?,
        5000,
        1024 * 1024,
        50 * 1024 * 1024,
        i32::from(CONFIG.cgroup),
        8,
    );
    let spj_status = sandbox.spawn().await?;
    drop(permit);

    // 读取 spj 程序的输出，无论结果 ac 与否，都要将其返回
    let outmsg = read_file_2048(path.join(SPJ_STDOUT_FILENAME)).await?;
    let errmsg = read_file_2048(path.join(SPJ_STDERR_FILENAME)).await?;
    // spj 程序的返回值（code）代表了结果，0 ac，1 wa
    return if spj_status.exit_code == 0 {
        Ok(spj_result(
            status.time_used,
            status.memory_used,
            JudgeResultEnum::Accepted,
            &outmsg,
            &errmsg,
        ))
    } else {
        Ok(spj_result(
            status.time_used,
            status.memory_used,
            JudgeResultEnum::WrongAnswer,
            &outmsg,
            &errmsg,
        ))
    };
}

async fn read_file_2048(filename: PathBuf) -> Result<String> {
    let mut buffer = [0; 2048];
    let mut file = try_io!(File::open(filename).await);
    try_io!(file.read(&mut buffer).await);

    let mut offset = 0;
    for i in 0..2047 {
        offset = i;
        if buffer[i] == 0 {
            break;
        }
    }
    try_io!(file.read(&mut buffer[offset..]).await);
    Ok(String::from(String::from_utf8_lossy(&buffer[..offset])))
}
