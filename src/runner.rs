extern crate nix;

use super::allow::{gen_rules, trace_syscall};
use super::config::{STDERR_FILENAME, STDIN_FILENAME, STDOUT_FILENAME};
use super::error::{errno_str, Error, Result};
use super::exec_args::ExecArgs;
use super::seccomp::*;
use nix::unistd::close;
use std::convert::TryInto;
use std::env;
use std::ffi::CString;
use std::fs::{remove_file, File};
use std::future::Future;
use std::io;
use std::os::unix::io::IntoRawFd;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::ptr;
use std::sync::{mpsc, Arc, Mutex};
use std::task::{Context, Poll};
use std::thread;
use std::time::SystemTime;

extern "C" {
    fn MemoryUsage(fd: i32) -> i64;
}

#[derive(Clone)]
pub struct Runner {
    pub pid: i32,
    pub workdir: PathBuf,
    pub time_limit: i32,
    pub memory_limit: i32,
    pub traceme: bool,
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
    pub errmsg: String,
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
            // 处理评测进程的异常
            // 程序本身无法发出负数的 signal，因此此处使用负数作为异常标识
            if status.signal < 0 {
                return Poll::Ready(Err(Error::JudgeThreadError(status.errmsg)));
            }
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
            if Path::new(STDOUT_FILENAME).exists() {
                remove_file(STDOUT_FILENAME).unwrap();
            }
            dup(
                STDOUT_FILENAME,
                libc::STDOUT_FILENO,
                libc::O_CREAT | libc::O_RDWR,
                0o644,
            );
            if Path::new(STDERR_FILENAME).exists() {
                remove_file(STDERR_FILENAME).unwrap();
            }
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
            rl.rlim_cur = (self.time_limit as u64) / 1000 + 1;
            rl.rlim_max = rl.rlim_cur + 1;
            if libc::setrlimit(libc::RLIMIT_CPU, &rl) != 0 {
                eprintln!("setrlimit failure!");
                eprintln!("{:?}", io::Error::last_os_error().raw_os_error());
                panic!("How dare you!");
            }
            // 设置内存限制
            if self.memory_limit > 0 {
                rl.rlim_cur = (self.memory_limit as u64) * 1024;
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
            }
            if self.traceme {
                // 设置 trace 模式
                libc::ptrace(libc::PTRACE_TRACEME, 0, 0, 0);
                // 发送信号以确保父进程先执行
                libc::kill(libc::getpid(), libc::SIGSTOP);
            }
            let mut filter =
                SeccompFilter::new(gen_rules().into_iter().collect(), SeccompAction::Kill).unwrap();
            if self.traceme {
                let (syscall_number, rules) = trace_syscall(libc::SYS_brk);
                filter.add_rules(syscall_number, rules).unwrap();
                let (syscall_number, rules) = trace_syscall(libc::SYS_mmap);
                filter.add_rules(syscall_number, rules).unwrap();
                let (syscall_number, rules) = trace_syscall(libc::SYS_munmap);
                filter.add_rules(syscall_number, rules).unwrap();
                let (syscall_number, rules) = trace_syscall(libc::SYS_mremap);
                filter.add_rules(syscall_number, rules).unwrap();
            } else {
                let (syscall_number, rules) = allow_syscall(libc::SYS_brk);
                filter.add_rules(syscall_number, rules).unwrap();
                let (syscall_number, rules) = allow_syscall(libc::SYS_mmap);
                filter.add_rules(syscall_number, rules).unwrap();
                let (syscall_number, rules) = allow_syscall(libc::SYS_munmap);
                filter.add_rules(syscall_number, rules).unwrap();
                let (syscall_number, rules) = allow_syscall(libc::SYS_mremap);
                filter.add_rules(syscall_number, rules).unwrap();
            }
            SeccompFilter::apply(filter.try_into().unwrap()).unwrap();
            libc::execve(exec_args.pathname, exec_args.argv, exec_args.envp);
            libc::kill(libc::getpid(), libc::SIGKILL);
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
        // 程序占用内存定义为程序数据段 + 栈大小
        // from: https://www.hackerearth.com/practice/notes/vivekprakash/technical-diving-into-memory-used-by-a-program-in-online-judges/
        // VmRSS 为程序当前驻留在物理内存中的大小，对虚拟内存等无效
        let mut vm_mem = 0;
        let status_file = format!("/proc/{}/status", pid);
        let file = match File::open(status_file) {
            Ok(val) => val,
            Err(_) => {
                return judge_error(format!(
                    "open file `{}` failure!",
                    format!("/proc/{}/status", pid)
                ))
            }
        };
        let status_fd = file.into_raw_fd();
        let child_proc_str = format!("/proc/{}", pid);
        let child_proc = Path::new(&child_proc_str);

        unsafe {
            if self.traceme {
                // 设置 trace 模式与 seccomp 的互动
                libc::waitpid(pid, &mut status, 0);
                libc::ptrace(libc::PTRACE_SETOPTIONS, pid, 0, libc::PTRACE_O_TRACESECCOMP);
                // 控制子进程恢复执行
                libc::ptrace(libc::PTRACE_CONT, pid, 0, 0);
            }
            loop {
                if self.traceme {
                    // in call
                    // seccomp 会在系统调用之前触发 trace，因此此处空等一次，等待到系统调用返回时的 trace
                    libc::ptrace(libc::PTRACE_SYSCALL, pid, 0, 0);
                    libc::waitpid(pid, &mut status, 0);

                    let vmem = MemoryUsage(status_fd);
                    if vm_mem < vmem {
                        vm_mem = vmem;
                    }
                    // trace 模式下，如果检测到内存已经超出限制，则直接 kill & break
                    if vm_mem > self.memory_limit.into() {
                        debug!("MemoryLimitExceeded! break");
                        libc::kill(pid, libc::SIGKILL);
                        break;
                    }
                    // debug!("vm_mem: {}", vm_mem);

                    // 控制子进程恢复执行
                    libc::ptrace(libc::PTRACE_CONT, pid, 0, 0);
                }
                // out call
                // 等待子进程结束
                if !child_proc.exists() {
                    return judge_error("Process exited abnormally".to_string());
                }
                if libc::wait4(pid, &mut status, 0, &mut rusage) < 0 || libc::WIFEXITED(status) {
                    debug!("exited: {}", libc::WIFEXITED(status));
                    break;
                }
                // debug!("exited: {}", libc::WIFEXITED(status));
            }
        }

        match close(status_fd) {
            Ok(_) => {}
            Err(_) => return judge_error("close status file failure!".to_string()),
        };
        let mut exit_code = 0;
        let exited = libc::WIFEXITED(status);
        if exited {
            exit_code = libc::WEXITSTATUS(status);
        }
        let signal = if libc::WIFSIGNALED(status) {
            libc::WTERMSIG(status)
        } else if libc::WIFSTOPPED(status) {
            libc::WSTOPSIG(status)
        } else {
            0
        };
        // TODO: 添加 CGroup 的量度
        let time_used = rusage.ru_utime.tv_sec * 1000
            + i64::from(rusage.ru_utime.tv_usec) / 1000
            + rusage.ru_stime.tv_sec * 1000
            + i64::from(rusage.ru_stime.tv_usec) / 1000;

        let memory_used = if self.traceme {
            vm_mem
        } else {
            rusage.ru_maxrss
        };
        let real_time_used = match start.elapsed() {
            Ok(elapsed) => elapsed.as_millis(),
            Err(_) => return judge_error("real time elapsed failure!".to_string()),
        };
        return RunnerStatus {
            rusage: rusage,
            exit_code: exit_code,
            status: status,
            signal: signal,
            time_used: time_used,
            memory_used: memory_used,
            real_time_used: real_time_used,
            errmsg: "".to_string(),
        };
    }
}

fn judge_error(errmsg: String) -> RunnerStatus {
    RunnerStatus {
        rusage: libc::rusage {
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
        },
        exit_code: -1,
        status: -1,
        signal: -1,
        time_used: -1,
        memory_used: -1,
        real_time_used: 0,
        errmsg: errmsg,
    }
}
