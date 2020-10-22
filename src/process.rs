use super::config::STDIN_FILENAME;
use super::error::{Error, Result};
use super::runner::Runner;
use libc;
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

pub struct Process {
    // 因为 await 会获取所有权，导致 await 执行完会直接 drop
    // 因此资源不能直接绑定要 await 的目标，否则会在出现在收集需要的信息之前资源已经被回收
    // 同时，直接将结构传递给子进程也有可能会出现 double drop 的情况
    // 因此，此处需要抽离一层
    pub runner: Runner,
}

impl Process {
    pub fn new(
        cmd: String,
        workdir: PathBuf,
        in_data: &Vec<u8>,
        time_limit: i32,
        memory_limit: i32,
    ) -> Result<Process> {
        let (tx, rx) = mpsc::channel();

        debug!("writing input file");
        // TODO: 此处同步写入文件，后续可以修改为异步写入，防止阻塞整体流程
        if let Err(e) = fs::write(workdir.join(STDIN_FILENAME), &in_data) {
            return Err(Error::FileWriteError(e));
        };

        Ok(Process {
            runner: Runner {
                pid: -1,
                time_limit: time_limit,
                memory_limit: memory_limit,
                cmd: cmd,
                workdir: workdir,
                tx: Arc::new(Mutex::new(tx)),
                rx: Arc::new(Mutex::new(rx)),
            },
        })
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        let mut status = 0;
        let pid;
        unsafe {
            pid = libc::waitpid(self.runner.pid, &mut status, libc::WNOHANG);
        }
        // > 0: 对应子进程退出但未回收资源
        // = 0: 对应子进程存在但未退出
        // 如果在运行过程中上层异常中断，则需要 kill 子进程并回收资源
        if pid >= 0 {
            unsafe {
                libc::kill(self.runner.pid, 9);
                libc::waitpid(self.runner.pid, &mut status, 0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn hello() {
        let s = String::from("hello");
        let bytes = s.into_bytes();
        assert_eq!(&[104, 101, 108, 108, 111][..], &bytes[..]);
    }

    #[tokio::test]
    async fn run() {}
}
