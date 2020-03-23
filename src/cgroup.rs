use std::result;

#[derive(Debug)]
pub enum Error {
    Error(String),
}

pub type Result<T> = result::Result<T, Error>;

pub struct Cgroup {
    pid: u32
}

impl Cgroup {
    pub fn new(pid: u32) -> Result<Self> {
        return Ok(Cgroup { pid });
    }
    pub fn attach(&self) -> Result<()> {
        return Ok(());
    }
}
