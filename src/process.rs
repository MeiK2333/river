use libc;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::thread;
use std::time::Duration;

pub struct Process {
    pub pid: i32,
    pub time_limit: i32,
    pub read_time_limit: i32,
    pub memory_limit: i32,
    pub stdin_fd: i32,
    pub stdout_fd: i32,
    pub stderr_fd: i32,
    pub cmd: str,
}

impl Future for Process {
    type Output = &'static str;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<&'static str> {
        let mut status = 0;
        unsafe {
            // TODO: fork、修改用户、组、设置限制，exec 等等
            // TODO: seccomp 等保证安全
            let _pid = libc::waitpid(self.pid, &mut status, libc::WNOHANG);
        }
        if status != 0 {
            return Poll::Ready("");
        }
        let waker = cx.waker().clone();

        thread::spawn(move || {
            thread::sleep(Duration::from_millis(3000));
            waker.wake();
        });

        Poll::Pending
    }
}
