use super::error::Result;
use std::fmt;
use std::path::Path;

#[derive(Debug, Copy, Clone)]
pub struct ResourceUsed {
    pub time_used: u32,
    pub memory_used: u32,
}

#[derive(Debug, Clone)]
pub enum TestCaseResult {
    Accepted(ResourceUsed),
    CompileError(ResourceUsed, String),
    WrongAnswer(ResourceUsed),
    RuntimeError(ResourceUsed, String),
    SystemError(ResourceUsed, String),
}

impl fmt::Display for TestCaseResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl TestCaseResult {
    pub fn standard<P: AsRef<Path>>(output_file: P, answer_file: P) -> Result<TestCaseResult> {
        let result = TestCaseResult::Accepted(ResourceUsed {
            time_used: 0,
            memory_used: 0,
        });
        // TODO: 对比答案文件，返回结果
        Ok(result)
    }
}
