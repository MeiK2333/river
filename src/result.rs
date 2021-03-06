use crate::error::Error;
use crate::error::Result;
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
            outmsg: String::from(""),
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
            time_used,
            memory_used,
            result: JudgeResultEnum::CompileError as i32,
            errmsg: String::from(errmsg),
            outmsg: String::from(""),
        })),
    }
}

pub fn compile_success(time_used: i64, memory_used: i64) -> JudgeResponse {
    judge_result(time_used, memory_used, JudgeResultEnum::CompileSuccess)
}

pub fn accepted(time_used: i64, memory_used: i64) -> JudgeResponse {
    judge_result(time_used, memory_used, JudgeResultEnum::Accepted)
}

pub fn wrong_answer(time_used: i64, memory_used: i64) -> JudgeResponse {
    judge_result(time_used, memory_used, JudgeResultEnum::WrongAnswer)
}

pub fn time_limit_exceeded(time_used: i64, memory_used: i64) -> JudgeResponse {
    judge_result(time_used, memory_used, JudgeResultEnum::TimeLimitExceeded)
}

pub fn memory_limit_exceeded(time_used: i64, memory_used: i64) -> JudgeResponse {
    judge_result(time_used, memory_used, JudgeResultEnum::MemoryLimitExceeded)
}

pub fn runtime_error(time_used: i64, memory_used: i64, errmsg: &str) -> JudgeResponse {
    JudgeResponse {
        state: Some(State::Result(JudgeResult {
            time_used,
            memory_used,
            result: JudgeResultEnum::RuntimeError as i32,
            errmsg: String::from(errmsg),
            outmsg: String::from(""),
        })),
    }
}

fn judge_result(time_used: i64, memory_used: i64, result: JudgeResultEnum) -> JudgeResponse {
    JudgeResponse {
        state: Some(State::Result(JudgeResult {
            time_used,
            memory_used,
            result: result as i32,
            errmsg: String::from(""),
            outmsg: String::from(""),
        })),
    }
}

pub fn spj_result(
    time_used: i64,
    memory_used: i64,
    result: JudgeResultEnum,
    outmsg: &str,
    errmsg: &str,
) -> JudgeResponse {
    JudgeResponse {
        state: Some(State::Result(JudgeResult {
            time_used,
            memory_used,
            result: result as i32,
            errmsg: String::from(errmsg),
            outmsg: String::from(outmsg),
        })),
    }
}

pub fn standard_result(out: &[u8], ans: &[u8]) -> Result<JudgeResultEnum> {
    let out_len = out.len();
    let ans_len = ans.len();
    let mut out_offset = 0;
    let mut ans_offset = 0;
    // 没有 PE，PE 直接 WA
    let mut r = JudgeResultEnum::Accepted;
    while out_offset <= out_len && ans_offset <= ans_len {
        let (out_start, out_end, out_exists) = next_line(&out, out_offset, out_len);
        let (ans_start, ans_end, ans_exists) = next_line(&ans, ans_offset, ans_len);
        if !out_exists || !ans_exists {
            // 如果一个已经读取完但另一个还有数据，则结果为 WA
            if out_exists != ans_exists {
                r = JudgeResultEnum::WrongAnswer;
            }
            break;
        }
        // 如果两个数据当前行长度不同，则结果为 WA（这个长度已经排除了末尾空白符号）
        if out_end - out_start != ans_end - ans_start {
            r = JudgeResultEnum::WrongAnswer;
            break;
        }
        let line_len = out_end - out_start;
        for i in 0..line_len {
            // 逐个对比
            if out[out_start + i] != ans[ans_start + i] {
                r = JudgeResultEnum::WrongAnswer;
                break;
            }
        }
        // 如果结果出来了，则退出循环
        if r != JudgeResultEnum::Accepted {
            break;
        }
        out_offset = out_end;
        ans_offset = ans_end;
    }
    Ok(r)
}

/**
 * 忽略空行与每行末尾的空格与制表符
 * 如果某行只有空白字符，则忽略此行
 * "Hello   ;      "
 * "                    "
 * "    World"
 * -----------------
 * "Hello   ;"
 * "    World"
 */
fn next_line(v: &[u8], offset: usize, len: usize) -> (usize, usize, bool) {
    let mut line_offset = offset;
    let mut left = 0;
    let mut right = len;
    let mut has_line = false;
    while line_offset < len {
        let ch = v[line_offset] as char;
        // 当读取到某行结束时
        if ch == '\n' {
            if has_line {
                // 如果已经有新行的数据，则在这个位置结束
                right = line_offset;
                break;
            } else {
                // 如果还没有数据，说明整行为空，忽略当前行，将下一行设为起点重复过程
                left = line_offset + 1;
            }
        }
        if ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r' {
            // 空白字符
        } else {
            // 非空白字符
            has_line = true;
        }
        line_offset += 1;
    }
    // 排除该行末尾的空白字符
    while left < right {
        let ch = v[right - 1] as char;
        if ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r' {
            // 空白字符
        } else {
            // 非空白字符
            break;
        }
        right -= 1;
    }
    (left, right, has_line)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        let v: &[u8] = "Hello World!".as_bytes();
        let (l, r, e) = next_line(v, 0, v.len());
        assert_eq!(l, 0);
        assert_eq!(r, 12);
        assert!(e);
        let (_l, _r, e) = next_line(v, r, v.len());
        assert!(!e);
    }

    #[test]
    fn test2() {
        let v: &[u8] = "Hello World!   ".as_bytes();
        let (l, r, e) = next_line(v, 0, v.len());
        assert_eq!(l, 0);
        assert_eq!(r, 12);
        assert!(e);
    }

    #[test]
    fn test3() {
        let v: &[u8] = "   Hello World!".as_bytes();
        let (l, r, e) = next_line(v, 0, v.len());
        assert_eq!(l, 0);
        assert_eq!(r, 15);
        assert!(e);
    }

    #[test]
    fn test4() {
        let v: &[u8] =
            "   Hello World!\n   Hello World!\n\n\n\n   \n\n\n\n\n\n    \t\t\t   \t\n\n\n\n"
                .as_bytes();
        let (l, r, e) = next_line(v, 0, v.len());
        assert_eq!(l, 0);
        assert_eq!(r, 15);
        assert!(e);
        let (l, r, e) = next_line(v, r, v.len());
        assert_eq!(l, 16);
        assert_eq!(r, 31);
        assert!(e);
        let (_l, _r, e) = next_line(v, r, v.len());
        assert!(!e);
    }

    #[test]
    fn test5() {
        let ans: &[u8] = "Hello World!".as_bytes();
        let out: &[u8] = "Hello World!".as_bytes();
        assert_eq!(
            standard_result(out, ans).unwrap(),
            JudgeResultEnum::Accepted
        );
    }

    #[test]
    fn test6() {
        let ans: &[u8] = "Hello World!".as_bytes();
        let out: &[u8] = "Hello World!   ".as_bytes();
        assert_eq!(
            standard_result(out, ans).unwrap(),
            JudgeResultEnum::Accepted
        );
    }

    #[test]
    fn test7() {
        let ans: &[u8] = "Hello World!  \n\n\n\n  \n\n\n\n".as_bytes();
        let out: &[u8] = "Hello World!\t\t\t\t\n\n\n\n    \n\n\n\n\t\t\t\t".as_bytes();
        assert_eq!(
            standard_result(out, ans).unwrap(),
            JudgeResultEnum::Accepted
        );
    }

    #[test]
    fn test8() {
        let ans: &[u8] = "Hello World!".as_bytes();
        let out: &[u8] = "".as_bytes();
        assert_eq!(
            standard_result(out, ans).unwrap(),
            JudgeResultEnum::WrongAnswer
        );
    }

    #[test]
    fn test9() {
        let ans: &[u8] = "".as_bytes();
        let out: &[u8] = "".as_bytes();
        assert_eq!(
            standard_result(out, ans).unwrap(),
            JudgeResultEnum::Accepted
        );
    }

    #[test]
    fn test10() {
        let ans: &[u8] = "Hello World!".as_bytes();
        let out: &[u8] = "Hello World!\n".as_bytes();
        assert_eq!(
            standard_result(out, ans).unwrap(),
            JudgeResultEnum::Accepted
        );
    }
}
