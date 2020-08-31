use super::error::{Error, Result};
use libc;
use std::future::Future;
use std::pin::Pin;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::thread;
use std::time::SystemTime;

#[derive(Clone)]
pub struct Process {
    pub pid: i32,
    pub time_limit: i32,
    pub memory_limit: i32,
    pub stdin_fd: i32,
    pub stdout_fd: i32,
    pub stderr_fd: i32,
    pub cmd: String,
    tx: Arc<Mutex<mpsc::Sender<ProcessStatus>>>,
    rx: Arc<Mutex<mpsc::Receiver<ProcessStatus>>>,
}

impl Process {
    pub fn new() -> Process {
        let (tx, rx) = mpsc::channel();
        Process {
            pid: -1,
            time_limit: -1,
            memory_limit: -1,
            stdin_fd: -1,
            stdout_fd: -1,
            stderr_fd: -1,
            cmd: "".to_string(),
            tx: Arc::new(Mutex::new(tx)),
            rx: Arc::new(Mutex::new(rx)),
        }
    }
    pub fn set_pid(&mut self, pid: i32) {
        self.pid = pid;
    }
}

impl Future for Process {
    type Output = Result<ProcessStatus>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<ProcessStatus>> {
        let process = Pin::into_inner(self);
        // 如果 pid == -1，则说明子进程还没有运行，开始进程
        if process.pid == -1 {
            let (tx, rx) = mpsc::channel();
            let process_clone = process.clone();
            let waker = cx.waker().clone();
            thread::spawn(move || {
                let pid;
                unsafe {
                    pid = libc::fork();
                }
                if pid == 0 {
                    run(process_clone);
                } else if pid > 0 {
                    tx.send(pid).unwrap();
                    let status = wait(pid);
                    let status_tx = process_clone.tx.lock().unwrap();
                    status_tx.send(status).unwrap();
                    waker.wake();
                } else {
                    panic!("How dare you!");
                }
            });

            // 等待子线程启动子进程并返回 pid
            let pid = match rx.recv() {
                Ok(val) => val,
                Err(_) => return Poll::Ready(Err(Error::ChannelRecvError)),
            };
            process.set_pid(pid);
            return Poll::Pending;
        } else {
            // 再次进入 poll，说明子进程已经结束，通知了 wake
            // 此时 channel 应该是有数据的
            let status = match process.rx.lock() {
                Ok(rx) => match rx.recv() {
                    Ok(val) => val,
                    Err(_) => return Poll::Ready(Err(Error::ChannelRecvError)),
                },
                Err(_) => return Poll::Ready(Err(Error::ChannelRecvError)),
            };
            return Poll::Ready(Ok(status));
        }
    }
}

fn run(_process: Process) {
    unsafe {
        // TODO
        println!("run");
        libc::exit(1);
    }
}

fn wait(pid: i32) -> ProcessStatus {
    let start = SystemTime::now();
    let mut status = 0;
    let mut rusage = libc::rusage {
        ru_utime: libc::timeval {
            tv_sec: 0 as libc::time_t,
            tv_usec: 0 as libc::suseconds_t,
        },
        ru_stime: libc::timeval {
            tv_sec: 0 as libc::time_t,
            tv_usec: 0 as libc::suseconds_t,
        },
        ru_maxrss: 0 as libc::c_long,
        ru_ixrss: 0 as libc::c_long,
        ru_idrss: 0 as libc::c_long,
        ru_isrss: 0 as libc::c_long,
        ru_minflt: 0 as libc::c_long,
        ru_majflt: 0 as libc::c_long,
        ru_nswap: 0 as libc::c_long,
        ru_inblock: 0 as libc::c_long,
        ru_oublock: 0 as libc::c_long,
        ru_msgsnd: 0 as libc::c_long,
        ru_msgrcv: 0 as libc::c_long,
        ru_nsignals: 0 as libc::c_long,
        ru_nvcsw: 0 as libc::c_long,
        ru_nivcsw: 0 as libc::c_long,
    };
    unsafe {
        libc::waitpid(pid, &mut status, 0);
        libc::getrusage(pid, &mut rusage);
    }

    let mut exit_code = 0;
    let exited = unsafe { libc::WIFEXITED(status) };
    if exited {
        exit_code = unsafe { libc::WEXITSTATUS(status) };
    }
    let signal = unsafe {
        if libc::WIFSIGNALED(status) {
            libc::WTERMSIG(status)
        } else if libc::WIFSTOPPED(status) {
            libc::WSTOPSIG(status)
        } else {
            0
        }
    };
    let real_time_used = match start.elapsed() {
        Ok(elapsed) => elapsed.as_millis(),
        // 这种地方如果出错了，确实没有办法解决
        // 只能崩溃再见了
        // How dare you!
        Err(_) => panic!("How dare you!"),
    };
    return ProcessStatus {
        rusage: rusage,
        exit_code: exit_code,
        status: status,
        signal: signal,
        real_time_used: real_time_used,
    };
}

#[derive(Clone)]
pub struct ProcessStatus {
    pub rusage: libc::rusage,
    pub exit_code: i32,
    pub status: i32,
    pub signal: i32,
    pub real_time_used: u128,
}
