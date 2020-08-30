use super::error::{Error, Result};
use libc;
use std::future::Future;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::thread;

#[derive(Clone)]
pub struct Process {
    pub pid: i32,
    pub time_limit: i32,
    pub memory_limit: i32,
    pub stdin_fd: i32,
    pub stdout_fd: i32,
    pub stderr_fd: i32,
    pub cmd: String,
}

impl Process {
    pub fn new() -> Process {
        Process {
            pid: -1,
            time_limit: -1,
            memory_limit: -1,
            stdin_fd: -1,
            stdout_fd: -1,
            stderr_fd: -1,
            cmd: "".to_string(),
        }
    }
    pub fn set_pid(&mut self, pid: i32) {
        self.pid = pid;
    }
}

impl Future for Process {
    type Output = Result<ProcessStatus>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<ProcessStatus>> {
        // pid == -1 时为第一次触发，此时开始运行
        // 直到进程执行完毕， wake 通知进入下次检查
        if self.pid == -1 {
            let pid;
            unsafe {
                pid = libc::fork();
            }
            if pid == 0 {
                // 子进程
                // TODO: fork、修改用户、组、设置限制，exec 等等
                // TODO: seccomp 等保证安全
                let process = Pin::into_inner(self).clone();
                run(process);
            } else if pid > 0 {
                // 父进程
                self.as_mut().set_pid(pid);
                let waker = cx.waker().clone();
                let pid = self.pid;

                thread::spawn(move || {
                    // 等待子进程结束
                    wait(pid);
                    // 触发唤醒异步
                    waker.wake();
                });
            } else {
                // 出错
                return Poll::Ready(Err(Error::ForkError(
                    io::Error::last_os_error().raw_os_error(),
                )));
            }
            return Poll::Pending;
        } else {
            // 子进程结束后被唤醒
            // 收集运行信息，返回数据，结束异步
            wait(self.pid);
            // TODO: Poll::Ready
            return Poll::Pending;
        }
    }
}

fn run(process: Process) {}

fn wait(pid: i32) {}

pub struct ProcessStatus {
    pub rusage: libc::rusage,
    pub exit_code: i32,
    pub status: i32,
}
