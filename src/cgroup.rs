use std::result;

#[derive(Debug)]
pub enum Error {
    Error(String),
}

pub type Result<T> = result::Result<T, Error>;

pub struct Cgroup {
    pub pid: u32,
}

impl Cgroup {
    pub fn new(pid: u32) -> Result<Self> {
        return Ok(Cgroup { pid });
    }
    pub fn attach(&self) -> Result<()> {
        if self.pid == 0 {
            return Err(Error::Error("Hello World!".to_string()));
        }
        return Ok(());
    }
}
