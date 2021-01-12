use crate::error::Error;
use crate::river::judge_response::State;
use crate::river::{JudgeResponse, JudgeResult, JudgeResultEnum};

pub fn system_error(err: Error) -> JudgeResponse {
    warn!("{:?}", err);
    JudgeResponse {
        state: Some(State::Result(JudgeResult {
            time_used: 0,
            memory_used: 0,
            result: JudgeResultEnum::SystemError as i32,
            errmsg: format!("{}", err).into(),
        })),
    }
}
