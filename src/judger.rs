use super::error::Result;
use crate::river::judge_response::{JudgeResult, JudgeStatus};
use crate::river::{JudgeRequest, JudgeResponse};

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

pub async fn compile(request: &JudgeRequest) -> Result<JudgeResponse> {
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
