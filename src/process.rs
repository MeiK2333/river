use super::error::{errno_str, Error, Result};
use libc;
use std::env;
use std::ffi::CString;
use std::fs;
use std::future::Future;
use std::io;
use std::mem;
use std::path::PathBuf;
use std::pin::Pin;
use std::ptr;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::thread;
use std::time::SystemTime;

#[derive(Clone)]
pub struct Process {
    pub pid: i32,
    pub workdir: PathBuf,
    pub time_limit: i32,
    pub memory_limit: i32,
    pub stdin_file: Option<PathBuf>,
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
            stdin_file: None,
            cmd: "".to_string(),
            workdir: PathBuf::from(""),
            tx: Arc::new(Mutex::new(tx)),
            rx: Arc::new(Mutex::new(rx)),
        }
    }
    pub fn set_pid(&mut self, pid: i32) {
        self.pid = pid;
    }

    #[allow(dead_code)]
    #[allow(unused_variables)]
    // 为进程设置 stdin 的数据
    pub fn set_stdin(in_data: &Vec<u8>) {
        // TODO
    }

    #[allow(dead_code)]
    #[allow(unused_variables)]
    // 从 stdout 中读取指定长度的内容
    pub fn read_stdout(len: i32) {
        // TODO
    }

    #[allow(dead_code)]
    #[allow(unused_variables)]
    // 从 stderr 中读取指定长度的内容
    pub fn read_stderr(len: i32) {
        // TODO
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        if self.pid > 0 {
            let mut status = 0;
            let pid;
            unsafe {
                pid = libc::waitpid(self.pid, &mut status, libc::WNOHANG);
            }
            // > 0: 对应子进程退出
            // = 0: 对应子进程存在但未退出
            // 如果在运行过程中上层异常中断，则需要 kill 子进程并回收资源
            if pid >= 0 {
                unsafe {
                    libc::kill(self.pid, 9);
                    libc::waitpid(self.pid, &mut status, 0);
                }
            }
        }
    }
}

impl Future for Process {
    type Output = Result<ProcessStatus>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<ProcessStatus>> {
        let process = Pin::into_inner(self);
        // 如果 pid == -1，则说明子进程还没有运行，开始进程
        if process.pid == -1 {
            let (tx, rx) = mpsc::channel();
            let mut process_clone = process.clone();
            let waker = cx.waker().clone();
            thread::spawn(move || {
                let pid;
                unsafe {
                    pid = libc::fork();
                }
                if pid == 0 {
                    process_clone.run();
                } else if pid > 0 {
                    tx.send(pid).unwrap();
                    process_clone.set_pid(pid);
                    let status = process_clone.wait();
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

pub struct ExecArgs {
    pub pathname: *const libc::c_char,
    pub argv: *const *const libc::c_char,
    pub envp: *const *const libc::c_char,
}

impl ExecArgs {
    fn build(cmd: &String) -> Result<ExecArgs> {
        let cmds: Vec<&str> = cmd.split_whitespace().collect();
        let pathname = cmds[0].clone();
        let pathname_str = match CString::new(pathname) {
            Ok(value) => value,
            Err(err) => return Err(Error::StringToCStringError(err)),
        };
        let pathname = pathname_str.as_ptr();

        let mut argv_vec: Vec<*const libc::c_char> = vec![];
        for item in cmds.iter() {
            let cstr = match CString::new(item.clone()) {
                Ok(value) => value,
                Err(err) => return Err(Error::StringToCStringError(err)),
            };
            let cptr = cstr.as_ptr();
            // 需要使用 mem::forget 来标记
            // 否则在此次循环结束后，cstr 就会被回收，后续 exec 函数无法通过指针获取到字符串内容
            mem::forget(cstr);
            argv_vec.push(cptr);
        }
        // argv 与 envp 的参数需要使用 NULL 来标记结束
        argv_vec.push(ptr::null());
        let argv: *const *const libc::c_char = argv_vec.as_ptr() as *const *const libc::c_char;

        // env 环境变量传递当前进程环境变量
        let mut envp_vec: Vec<*const libc::c_char> = vec![];
        for (key, value) in env::vars_os() {
            let mut key = match key.to_str() {
                Some(val) => val.to_string(),
                None => return Err(Error::OsStringToStringError(key)),
            };
            let value = match value.to_str() {
                Some(val) => val.to_string(),
                None => return Err(Error::OsStringToStringError(value)),
            };
            key.push_str("=");
            key.push_str(&value);
            let cstr = match CString::new(key) {
                Ok(value) => value,
                Err(err) => return Err(Error::StringToCStringError(err)),
            };
            let cptr = cstr.as_ptr();
            // 需要使用 mem::forget 来标记
            // 否则在此次循环结束后，cstr 就会被回收，后续 exec 函数无法通过指针获取到字符串内容
            mem::forget(cstr);
            envp_vec.push(cptr);
        }
        envp_vec.push(ptr::null());
        let envp = envp_vec.as_ptr() as *const *const libc::c_char;

        mem::forget(pathname_str);
        mem::forget(argv_vec);
        mem::forget(envp_vec);
        Ok(ExecArgs {
            pathname,
            argv,
            envp,
        })
    }
}

impl Drop for ExecArgs {
    fn drop(&mut self) {
        // TODO: 将不安全的指针类型转换回内置类型，以便由 Rust 自动回收资源
        // TODO: 优先级较低，因为目前只在子进程里进行这个操作，且操作后会很快 exec，操作系统会回收这些内存
        println!("Dropping!");
    }
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

impl Process {
    fn run(&self) {
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
            if let Some(file) = &self.stdin_file {
                let filename = file.to_str().unwrap();
                dup(&filename, libc::STDIN_FILENO, libc::O_RDONLY, 0o644)
            }
            dup(
                "stdout.txt",
                libc::STDOUT_FILENO,
                libc::O_CREAT | libc::O_RDWR,
                0o644,
            );
            dup(
                "stderr.txt",
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

impl Process {
    fn wait(&self) -> ProcessStatus {
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
            let val = libc::wait4(pid, &mut status, 0, &mut rusage);
            if val < 0 {
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
        let stdout = match fs::read_to_string(&self.workdir.join("stdout.txt")) {
            Ok(val) => val,
            Err(_) => panic!("How dare you!"),
        };
        if let Err(_) = fs::remove_file(&self.workdir.join("stdout.txt")) {
            panic!("How dare you!");
        }
        let stderr = match fs::read_to_string(&self.workdir.join("stderr.txt")) {
            Ok(val) => val,
            Err(_) => panic!("How dare you!"),
        };
        if let Err(_) = fs::remove_file(&self.workdir.join("stderr.txt")) {
            panic!("How dare you!");
        }
        return ProcessStatus {
            rusage: rusage,
            exit_code: exit_code,
            status: status,
            signal: signal,
            time_used: time_used,
            memory_used: memory_used,
            real_time_used: real_time_used,
            stdout: stdout,
            stderr: stderr,
        };
    }
}

#[derive(Clone)]
pub struct ProcessStatus {
    pub rusage: libc::rusage,
    pub exit_code: i32,
    pub status: i32,
    pub signal: i32,
    pub time_used: i64,
    pub memory_used: i64,
    pub real_time_used: u128,
    pub stdout: String,
    pub stderr: String,
}
