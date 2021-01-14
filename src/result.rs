use crate::error::Error;
use crate::river::judge_response::State;
use crate::river::{JudgeResponse, JudgeResult, JudgeResultEnum, JudgeStatus};

pub fn system_error(err: Error) -> JudgeResponse {
    warn!("{}", err);
    JudgeResponse {
        state: Some(State::Result(JudgeResult {
            time_used: 0,
            memory_used: 0,
            result: JudgeResultEnum::SystemError as i32,
            errmsg: format!("{}", err).into(),
        })),
    }
}

pub fn pending() -> JudgeResponse {
    JudgeResponse {
        state: Some(State::Status(JudgeStatus::Pending as i32)),
    }
}

pub fn running() -> JudgeResponse {
    JudgeResponse {
        state: Some(State::Status(JudgeStatus::Running as i32)),
    }
}

pub fn compile_error(time_used: i64, memory_used: i64, errmsg: &str) -> JudgeResponse {
    JudgeResponse {
        state: Some(State::Result(JudgeResult {
            time_used: time_used,
            memory_used: memory_used,
            result: JudgeResultEnum::CompileError as i32,
            errmsg: String::from(errmsg),
        })),
    }
}

pub fn compile_success(time_used: i64, memory_used: i64) -> JudgeResponse {
    JudgeResponse {
        state: Some(State::Result(JudgeResult {
            time_used: time_used,
            memory_used: memory_used,
            result: JudgeResultEnum::CompileSuccess as i32,
            errmsg: String::from(""),
        })),
    }
}
