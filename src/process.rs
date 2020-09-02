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
    pub stdin_file: Option<String>,
    pub stdout_file: Option<String>,
    pub stderr_file: Option<String>,
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
            stdout_file: None,
            stderr_file: None,
            cmd: "".to_string(),
            workdir: PathBuf::from(""),
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
                    let status = wait(pid, process_clone.workdir);
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
        println!("Dropping!");
    }
}

fn run(process: Process) {
    // 子进程里崩溃也无法返回，崩溃就直接崩溃了
    let exec_args = ExecArgs::build(&process.cmd).unwrap();
    // 修改工作目录
    env::set_current_dir(process.workdir).unwrap();
    unsafe {
        // TODO: 资源限制等
        // 重定向文件描述符
        if let Some(file) = process.stdin_file {
            dup(&file, libc::STDIN_FILENO, libc::O_RDONLY, 0o644)
        }
        if let Some(file) = process.stdout_file {
            dup(
                &file,
                libc::STDOUT_FILENO,
                libc::O_CREAT | libc::O_RDWR,
                0o644,
            )
        }
        if let Some(file) = process.stderr_file {
            dup(
                &file,
                libc::STDERR_FILENO,
                libc::O_CREAT | libc::O_RDWR,
                0o644,
            )
        }
        libc::execve(exec_args.pathname, exec_args.argv, exec_args.envp);
    }
    panic!("How dare you!");
}

unsafe fn dup(filename: &str, to: libc::c_int, flag: libc::c_int, mode: libc::c_int) {
    let filename_str = CString::new(filename).unwrap();
    let filename = filename_str.as_ptr();
    let fd = libc::open(filename, flag, mode);
    if fd < 0 {
        let err = io::Error::last_os_error().raw_os_error();
        eprintln!("open failure!");
        panic!(errno_str(err));
    }
    if libc::dup2(fd, to) < 0 {
        let err = io::Error::last_os_error().raw_os_error();
        eprintln!("dup2 failure!");
        panic!(errno_str(err));
    }
}

fn wait(pid: i32, workdir: PathBuf) -> ProcessStatus {
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
    let stdout = match fs::read_to_string(workdir.join("stdout.txt")) {
        Ok(val) => val,
        Err(_) => panic!("How dare you!"),
    };
    let stderr = match fs::read_to_string(workdir.join("stderr.txt")) {
        Ok(val) => val,
        Err(_) => panic!("How dare you!"),
    };
    return ProcessStatus {
        rusage: rusage,
        exit_code: exit_code,
        status: status,
        signal: signal,
        real_time_used: real_time_used,
        stdout: stdout,
        stderr: stderr,
    };
}

#[derive(Clone)]
pub struct ProcessStatus {
    pub rusage: libc::rusage,
    pub exit_code: i32,
    pub status: i32,
    pub signal: i32,
    pub real_time_used: u128,
    pub stdout: String,
    pub stderr: String,
}
