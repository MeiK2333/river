use std::fmt;

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
