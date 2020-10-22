use super::config::{STDERR_FILENAME, STDIN_FILENAME, STDOUT_FILENAME};
use super::error::{errno_str, Error, Result};
use super::exec_args::ExecArgs;
use std::env;
use std::ffi::CString;
use std::future::Future;
use std::io;
use std::path::PathBuf;
use std::pin::Pin;
use std::ptr;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::thread;
use std::time::SystemTime;

#[derive(Clone)]
pub struct Runner {
    pub pid: i32,
    pub workdir: PathBuf,
    pub time_limit: i32,
    pub memory_limit: i32,
    pub cmd: String,
    pub tx: Arc<Mutex<mpsc::Sender<RunnerStatus>>>,
    pub rx: Arc<Mutex<mpsc::Receiver<RunnerStatus>>>,
}

impl Runner {
    fn set_pid(&mut self, pid: i32) {
        self.pid = pid;
    }
}

#[derive(Clone)]
pub struct RunnerStatus {
    pub rusage: libc::rusage,
    pub exit_code: i32,
    pub status: i32,
    pub signal: i32,
    pub time_used: i64,
    pub memory_used: i64,
    pub real_time_used: u128,
}

const ITIMER_REAL: libc::c_int = 0;
extern "C" {
    #[cfg_attr(
        all(target_os = "macos", target_arch = "x86"),
        link_name = "setitimer$UNIX2003"
    )]
    fn setitimer(
        which: libc::c_int,
        new_value: *const libc::itimerval,
        old_value: *mut libc::itimerval,
    ) -> libc::c_int;
}

impl Future for Runner {
    type Output = Result<RunnerStatus>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<RunnerStatus>> {
        let runner = Pin::into_inner(self);
        // 如果 pid == -1，则说明子进程还没有运行，开始进程
        if runner.pid == -1 {
            let (tx, rx) = mpsc::channel();
            // 因为 poll 和 spawn 都需要 process 的所有权，这是矛盾的
            // 因此此处进行 clone，需要处理因此产生的 double drop 的问题
            let mut runner_clone = runner.clone();
            let waker = cx.waker().clone();
            thread::spawn(move || {
                let pid;
                unsafe {
                    pid = libc::fork();
                }
                if pid == 0 {
                    runner_clone.run();
                } else if pid > 0 {
                    tx.send(pid).unwrap();
                    runner_clone.set_pid(pid);
                    let status = runner_clone.wait();
                    let status_tx = runner_clone.tx.lock().unwrap();
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
            runner.set_pid(pid);
            return Poll::Pending;
        } else {
            // 再次进入 poll，说明子进程已经结束，通知了 wake
            // 此时 channel 应该是有数据的
            let status = match runner.rx.lock() {
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

unsafe fn dup(filename: &str, to: libc::c_int, flag: libc::c_int, mode: libc::c_int) {
    let filename_str = CString::new(filename).unwrap();
    let filename = filename_str.as_ptr();
    let fd = libc::open(filename, flag, mode);
    if fd < 0 {
        let err = io::Error::last_os_error().raw_os_error();
        eprintln!("open failure!");
        eprintln!("{:?}", io::Error::last_os_error().raw_os_error());
        panic!(errno_str(err));
    }
    if libc::dup2(fd, to) < 0 {
        let err = io::Error::last_os_error().raw_os_error();
        eprintln!("dup2 failure!");
        eprintln!("{:?}", io::Error::last_os_error().raw_os_error());
        panic!(errno_str(err));
    }
}

impl Runner {
    pub fn run(&self) {
        // 子进程里崩溃也无法返回，崩溃就直接崩溃了
        let exec_args = ExecArgs::build(&self.cmd).unwrap();
        // 修改工作目录
        env::set_current_dir(&self.workdir).unwrap();
        let mut rl = libc::rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        // 实际运行时间限制设置为 CPU 时间 + 2 * 2，尽量在防止恶意代码占用评测资源的情况下给正常用户的代码最宽松的环境
        let rt = libc::itimerval {
            it_interval: libc::timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
            it_value: libc::timeval {
                tv_sec: i64::from(self.time_limit / 1000 + 2) * 2,
                tv_usec: 0,
            },
        };
        unsafe {
            // 重定向文件描述符
            dup(STDIN_FILENAME, libc::STDIN_FILENO, libc::O_RDONLY, 0o644);
            dup(
                STDOUT_FILENAME,
                libc::STDOUT_FILENO,
                libc::O_CREAT | libc::O_RDWR,
                0o644,
            );
            dup(
                STDERR_FILENAME,
                libc::STDERR_FILENO,
                libc::O_CREAT | libc::O_RDWR,
                0o644,
            );
            // 墙上时钟限制
            if setitimer(ITIMER_REAL, &rt, ptr::null_mut()) == -1 {
                eprintln!("setitimer failure!");
                eprintln!("{:?}", io::Error::last_os_error().raw_os_error());
                panic!("How dare you!");
            }
            // CPU 时间限制，粒度为 S
            rl.rlim_cur = (self.time_limit / 1000 + 1) as u64;
            rl.rlim_max = rl.rlim_cur + 1;
            if libc::setrlimit(libc::RLIMIT_CPU, &rl) != 0 {
                eprintln!("setrlimit failure!");
                eprintln!("{:?}", io::Error::last_os_error().raw_os_error());
                panic!("How dare you!");
            }
            // 设置内存限制
            rl.rlim_cur = (self.memory_limit * 1024) as u64;
            rl.rlim_max = rl.rlim_cur + 1024;
            if libc::setrlimit(libc::RLIMIT_DATA, &rl) != 0 {
                eprintln!("setrlimit failure!");
                eprintln!("{:?}", io::Error::last_os_error().raw_os_error());
                panic!("How dare you!");
            }
            if libc::setrlimit(libc::RLIMIT_AS, &rl) != 0 {
                eprintln!("setrlimit failure!");
                eprintln!("{:?}", io::Error::last_os_error().raw_os_error());
                panic!("How dare you!");
            }
            libc::execve(exec_args.pathname, exec_args.argv, exec_args.envp);
        }
        panic!("How dare you!");
    }
}

impl Runner {
    pub fn wait(&self) -> RunnerStatus {
        let pid = self.pid;
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
            // 等待子进程结束
            if libc::wait4(pid, &mut status, 0, &mut rusage) < 0 {
                eprintln!("{:?}", io::Error::last_os_error().raw_os_error());
                panic!("How dare you!");
            }
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
        // TODO: 添加 CGroup 的量度
        let time_used = rusage.ru_utime.tv_sec * 1000
            + i64::from(rusage.ru_utime.tv_usec) / 1000
            + rusage.ru_stime.tv_sec * 1000
            + i64::from(rusage.ru_stime.tv_usec) / 1000;
        let memory_used = rusage.ru_maxrss;
        let real_time_used = match start.elapsed() {
            Ok(elapsed) => elapsed.as_millis(),
            // 这种地方如果出错了，确实没有办法解决
            // 只能崩溃再见了
            // How dare you!
            Err(_) => panic!("How dare you!"),
        };
        return RunnerStatus {
            rusage: rusage,
            exit_code: exit_code,
            status: status,
            signal: signal,
            time_used: time_used,
            memory_used: memory_used,
            real_time_used: real_time_used,
        };
    }
}
