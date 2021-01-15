use crate::cgroup::CGroupSet;
use crate::config::{STDERR_FILENAME, STDIN_FILENAME, STDOUT_FILENAME};
use crate::error::{errno_str, Error, Result};
use crate::exec_args::ExecArgs;
use crate::seccomp;
use libc;
use std::convert::TryInto;
use std::ffi::CString;
use std::fs::remove_file;
use std::future::Future;
use std::io;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::ptr;
use std::sync::{mpsc, Arc, Mutex};
use std::task::{Context, Poll};
use std::thread;
use std::time::{Duration, SystemTime};

#[cfg(test)]
use std::println as debug;

const STACK_SIZE: usize = 1024 * 1024;

macro_rules! syscall_or_panic {
    ($expression:expr) => {
        if $expression < 0 {
            let err = io::Error::last_os_error().raw_os_error();
            panic!(errno_str(err));
        };
    };
}

macro_rules! c_str_ptr {
    ($expression:expr) => {
        CString::new($expression).unwrap().as_ptr()
    };
}

#[derive(Clone)]
pub struct Process {
    pub cmd: String,
    pub time_limit: i32,
    pub memory_limit: i32,
    pub workdir: PathBuf,
}

#[derive(Clone)]
pub struct RunnerStatus {
    pub rusage: libc::rusage,
    pub exit_code: i32,
    pub status: i32,
    pub signal: i32,
    pub time_used: i64,
    pub real_time_used: u128,
    pub memory_used: i64,
    pub cgroup_memory_used: i64,
    pub errmsg: String,
}

impl Process {
    pub fn new(cmd: String, time_limit: i32, memory_limit: i32, workdir: PathBuf) -> Process {
        let runner = Process {
            cmd: cmd,
            time_limit: time_limit,
            memory_limit: memory_limit,
            workdir: workdir,
        };
        runner
    }
}

pub struct Runner {
    process: Process,
    pid: i32,
    tx: Arc<Mutex<mpsc::Sender<RunnerStatus>>>,
    rx: Arc<Mutex<mpsc::Receiver<RunnerStatus>>>,
    cgroup_set: CGroupSet,
}

impl Runner {
    pub fn from(process: Process) -> Result<Runner> {
        let (tx, rx) = mpsc::channel();
        Ok(Runner {
            process: process,
            pid: -1,
            tx: Arc::new(Mutex::new(tx)),
            rx: Arc::new(Mutex::new(rx)),
            cgroup_set: CGroupSet::new()?,
        })
    }
}

unsafe fn killpid(pid: i32) {
    let mut status = 0;

    // > 0: 对应子进程退出但未回收资源
    // = 0: 对应子进程存在但未退出
    // 如果在运行过程中上层异常中断，则需要 kill 子进程并回收资源
    if libc::waitpid(pid, &mut status, libc::WNOHANG) >= 0 {
        libc::kill(pid, 9);
        libc::waitpid(pid, &mut status, 0);
    }
}

impl Drop for Runner {
    fn drop(&mut self) {
        debug!("dropping");
        unsafe {
            killpid(self.pid);
        }
    }
}

impl Future for Runner {
    type Output = Result<RunnerStatus>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<RunnerStatus>> {
        let runner = Pin::into_inner(self);
        let time_limit = (runner.process.time_limit / 1000 + 1) as u64 * 2;
        // 创建 clone 所需的栈空间
        let stack = unsafe {
            libc::mmap(
                ptr::null_mut(),
                STACK_SIZE,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS | libc::MAP_STACK,
                -1,
                0,
            )
        };
        // 如果 pid == -1，则说明子进程还没有运行
        if runner.pid == -1 {
            let waker = cx.waker().clone();
            let pid = unsafe {
                libc::clone(
                    runit,
                    (stack as usize + STACK_SIZE) as *mut libc::c_void,
                    libc::SIGCHLD
                    | libc::CLONE_NEWUTS  // 设置新的 UTS 名称空间（主机名、网络名等）
                    | libc::CLONE_NEWNET  // 设置新的网络空间，如果没有配置网络，则该沙盒内部将无法联网
                    | libc::CLONE_NEWNS  // 为沙盒内部设置新的 namespaces 空间
                    | libc::CLONE_NEWIPC  // IPC 隔离
                    | libc::CLONE_NEWCGROUP  // 在新的 CGROUP 中创建沙盒
                    | libc::CLONE_NEWPID, // 外部进程对沙盒不可见
                    &mut runner.process as *mut _ as *mut libc::c_void,
                )
            };
            debug!("pid = {}", pid);
            runner.pid = pid;
            // 设置 cgroup 限制
            // 此处为父进程做的策略，所以没有与子进程的安全策略放一块
            runner.cgroup_set.apply(pid).unwrap();
            runner.cgroup_set.memory.set(
                "memory.limit_in_bytes",
                &format!("{}", runner.process.memory_limit as i64 * 1536), // 本来应该乘以 1024，此处略微放宽限制，从而让用户体验更好
            )?;
            let tx = runner.tx.clone();

            // 因为 wait 会阻塞等待结果，因此此处使用 thread::spawn 来 wait，以防止主流程被阻塞
            thread::spawn(move || {
                // 监控程序超时的任务
                // 此限制相对宽松（sec * 2 + 2），尽可能在保证系统安全的前提下给评测最好的体验
                thread::spawn(move || {
                    thread::sleep(Duration::from_secs(time_limit));
                    unsafe {
                        killpid(pid);
                    }
                });
                let status = Runner::waitpid(pid);
                // 子流程运行结束后通知主流程
                let status_tx = tx.lock().unwrap();
                status_tx.send(status).unwrap();
                waker.wake();
            });
            return Poll::Pending;
        } else {
            unsafe {
                libc::munmap(stack, STACK_SIZE);
            }
            let mut status = runner.rx.lock().unwrap().recv().unwrap();
            let mem_used = match runner
                .cgroup_set
                .memory
                .get("memory.max_usage_in_bytes")?
                .trim()
                .parse::<i64>()
            {
                Ok(val) => val,
                Err(e) => return Poll::Ready(Err(Error::ParseIntError(e))),
            };
            status.cgroup_memory_used = mem_used / 1024;
            debug!("cpu time used: {} ms", status.time_used);
            debug!("real time used: {} ms", status.real_time_used);
            debug!("cgroup memory used: {} KiB", status.cgroup_memory_used);
            debug!("rusage memory used: {} KiB", status.memory_used);
            if status.signal < 0 {
                return Poll::Ready(Err(Error::SystemError(status.errmsg)));
            }
            return Poll::Ready(Ok(status));
        }
    }
}

impl Runner {
    fn waitpid(pid: i32) -> RunnerStatus {
        let mut status: i32 = 0;
        let mut rusage = new_rusage();
        let start = SystemTime::now();
        unsafe {
            if libc::wait4(pid, &mut status, 0, &mut rusage) < 0 {
                return judge_system_error(String::from("wait4 failure!"));
            }
        }
        let real_time_used = match start.elapsed() {
            Ok(elapsed) => elapsed.as_millis(),
            Err(_) => return judge_system_error(String::from("real time elapsed failure!")),
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
        let time_used = rusage.ru_utime.tv_sec * 1000
            + i64::from(rusage.ru_utime.tv_usec) / 1000
            + rusage.ru_stime.tv_sec * 1000
            + i64::from(rusage.ru_stime.tv_usec) / 1000;
        let memory_used = rusage.ru_maxrss;

        RunnerStatus {
            errmsg: String::from(""),
            memory_used: memory_used,
            cgroup_memory_used: -1,
            time_used: time_used,
            real_time_used: real_time_used,
            exit_code: exit_code,
            signal: signal,
            status: status,
            rusage: rusage,
        }
    }
}

extern "C" fn runit(process: *mut libc::c_void) -> i32 {
    let process = unsafe { &mut *(process as *mut Process) };
    debug!("cmd = {}", process.cmd);
    let exec_args = ExecArgs::build(&process.cmd).unwrap();
    unsafe {
        security(&process);
        fd_dup();
        syscall_or_panic!(libc::execve(
            exec_args.pathname,
            exec_args.argv,
            exec_args.envp
        ));
        // 理论上并不会到这里，因此如果到这里，直接 kill 掉
        syscall_or_panic!(libc::kill(libc::getpid(), libc::SIGKILL));
    }
    0
}

/// 为评测沙盒提供安全保障
///
/// 包括以下策略：
///
/// - `mount` 隔离运行目录，安全的将内部数据传递出去
/// - `chdir` && `chroot`，将评测沙盒与宿主机的文件系统隔离
/// - `sethostname` && `setdomainname`，不暴露真实机器名
/// - `setgid` && `setuid`，修改运行用户为低权限的 `nobody`，配合文件权限，防止代码对沙盒内部进行恶意修改
/// - `seccomp` 阻止执行危险的系统调用
/// - `CLONE_NEWNET` 禁止沙盒内部连接网络
/// - `CLONE_NEWPID` 隔离内外进程空间
unsafe fn security(process: &Process) {
    // 全局默认权限 755，为运行目录设置特权
    syscall_or_panic!(libc::chmod(
        c_str_ptr!(process.workdir.to_str().unwrap()),
        0o777,
    ));

    // 等同于 mount --make-rprivate /
    // 不将挂载传播到其他空间，以免造成挂载混淆
    syscall_or_panic!(libc::mount(
        c_str_ptr!(""),
        c_str_ptr!("/"),
        c_str_ptr!(""),
        libc::MS_PRIVATE | libc::MS_REC,
        ptr::null_mut()
    ));

    // 挂载运行文件夹，除此目录外程序没有其他目录的写权限
    syscall_or_panic!(libc::mount(
        c_str_ptr!(process.workdir.to_str().unwrap()),
        c_str_ptr!("runtime/rootfs/tmp"),
        c_str_ptr!("none"),
        libc::MS_BIND | libc::MS_PRIVATE,
        ptr::null_mut(),
    ));

    // chdir && chroot，隔离文件系统
    syscall_or_panic!(libc::chdir(c_str_ptr!("runtime/rootfs")));
    syscall_or_panic!(libc::chroot(c_str_ptr!(".")));
    syscall_or_panic!(libc::chdir(c_str_ptr!("/tmp")));

    // 设置主机名
    syscall_or_panic!(libc::sethostname(c_str_ptr!("river"), 5));
    syscall_or_panic!(libc::setdomainname(c_str_ptr!("river"), 5));

    // 修改用户为 nobody
    syscall_or_panic!(libc::setgid(65534));
    syscall_or_panic!(libc::setuid(65534));

    let filter = seccomp::SeccompFilter::new(
        deny_syscalls().into_iter().collect(),
        seccomp::SeccompAction::Allow,
    )
    .unwrap();
    seccomp::SeccompFilter::apply(filter.try_into().unwrap()).unwrap();
}

/// 重定向 `stdin`、`stdout`、`stderr`
unsafe fn fd_dup() {
    // 重定向文件描述符
    if Path::new(STDIN_FILENAME).exists() {
        dup(STDIN_FILENAME, libc::STDIN_FILENO, libc::O_RDONLY, 0o644);
    }
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
}

unsafe fn dup(filename: &str, to: libc::c_int, flag: libc::c_int, mode: libc::c_int) {
    let filename_str = CString::new(filename).unwrap();
    let filename = filename_str.as_ptr();
    let fd = libc::open(filename, flag, mode);
    if fd < 0 {
        let err = io::Error::last_os_error().raw_os_error();
        panic!(errno_str(err));
    }
    syscall_or_panic!(libc::dup2(fd, to));
}

/// 阻止危险的系统调用
///
/// 参照 Docker 文档 [significant-syscalls-blocked-by-the-default-profile](https://docs.docker.com/engine/security/seccomp/#significant-syscalls-blocked-by-the-default-profile) 一节
fn deny_syscalls() -> Vec<seccomp::SyscallRuleSet> {
    vec![
        deny_syscall(libc::SYS_acct),
        deny_syscall(libc::SYS_add_key),
        deny_syscall(libc::SYS_bpf),
        deny_syscall(libc::SYS_clock_adjtime),
        deny_syscall(libc::SYS_clock_settime),
        deny_syscall(libc::SYS_create_module),
        deny_syscall(libc::SYS_delete_module),
        deny_syscall(libc::SYS_finit_module),
        deny_syscall(libc::SYS_get_kernel_syms),
        deny_syscall(libc::SYS_get_mempolicy),
        deny_syscall(libc::SYS_init_module),
        deny_syscall(libc::SYS_ioperm),
        deny_syscall(libc::SYS_iopl),
        deny_syscall(libc::SYS_kcmp),
        deny_syscall(libc::SYS_kexec_file_load),
        deny_syscall(libc::SYS_kexec_load),
        deny_syscall(libc::SYS_keyctl),
        deny_syscall(libc::SYS_lookup_dcookie),
        deny_syscall(libc::SYS_mbind),
        deny_syscall(libc::SYS_mount),
        deny_syscall(libc::SYS_move_pages),
        deny_syscall(libc::SYS_name_to_handle_at),
        deny_syscall(libc::SYS_nfsservctl),
        deny_syscall(libc::SYS_open_by_handle_at),
        deny_syscall(libc::SYS_perf_event_open),
        deny_syscall(libc::SYS_personality),
        deny_syscall(libc::SYS_pivot_root),
        deny_syscall(libc::SYS_process_vm_readv),
        deny_syscall(libc::SYS_process_vm_writev),
        deny_syscall(libc::SYS_ptrace),
        deny_syscall(libc::SYS_query_module),
        deny_syscall(libc::SYS_quotactl),
        deny_syscall(libc::SYS_reboot),
        deny_syscall(libc::SYS_request_key),
        deny_syscall(libc::SYS_set_mempolicy),
        deny_syscall(libc::SYS_setns),
        deny_syscall(libc::SYS_settimeofday),
        deny_syscall(libc::SYS_swapon),
        deny_syscall(libc::SYS_swapoff),
        deny_syscall(libc::SYS_sysfs),
        deny_syscall(libc::SYS__sysctl),
        deny_syscall(libc::SYS_umount2),
        deny_syscall(libc::SYS_unshare),
        deny_syscall(libc::SYS_uselib),
        deny_syscall(libc::SYS_userfaultfd),
        deny_syscall(libc::SYS_ustat),
    ]
}

#[inline(always)]
fn deny_syscall(syscall_number: i64) -> seccomp::SyscallRuleSet {
    (
        syscall_number,
        vec![seccomp::SeccompRule::new(
            vec![],
            seccomp::SeccompAction::Kill,
        )],
    )
}

/// 一个全为 `0` 的 `rusage`
#[inline(always)]
fn new_rusage() -> libc::rusage {
    libc::rusage {
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
    }
}

/// 由于评测系统本身异常而产生的错误
///
/// 因为正常程序返回 `signal` 不能为负数，因此此处使用负数的 `signal` 标识系统错误
#[inline(always)]
fn judge_system_error(errmsg: String) -> RunnerStatus {
    RunnerStatus {
        rusage: new_rusage(),
        exit_code: -1,
        status: -1,
        signal: -1,
        time_used: -1,
        memory_used: -1,
        cgroup_memory_used: -1,
        real_time_used: 0,
        errmsg: errmsg,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir_in;
    #[tokio::test]
    async fn test_echo() {
        let pwd = tempdir_in("/tmp").unwrap();
        let process = Process::new(
            String::from("/bin/echo Hello World!"),
            1000,
            65535,
            pwd.path().to_path_buf(),
        );
        let result = Runner::from(process).unwrap().await.unwrap();
        assert_eq!(result.exit_code, 0);
    }

    #[tokio::test]
    async fn test_sleep() {
        let pwd = tempdir_in("/tmp").unwrap();
        let process = Process::new(
            String::from("/bin/sleep 1"),
            2000,
            65535,
            pwd.path().to_path_buf(),
        );
        let result = Runner::from(process).unwrap().await.unwrap();
        assert!(result.real_time_used > 1000);
        assert!(result.real_time_used < 1500);
    }

    #[tokio::test]
    async fn test_output() {
        let pwd = tempdir_in("/tmp").unwrap();
        let process = Process::new(
            String::from("/bin/echo Hello World!"),
            1000,
            65535,
            pwd.path().to_path_buf(),
        );
        let runner = Runner::from(process).unwrap();
        let _ = runner.await.unwrap();
        let out = std::fs::read_to_string(pwd.path().join(STDOUT_FILENAME)).unwrap();
        assert_eq!(out, "Hello World!\n");
    }

    #[tokio::test]
    async fn time_limit() {
        let pwd = tempdir_in("/tmp").unwrap();
        let process = Process::new(
            String::from("/bin/sleep 100"),
            1000,
            65535,
            pwd.path().to_path_buf(),
        );
        let result = Runner::from(process).unwrap().await.unwrap();
        assert!(result.time_used < 2000);
        assert!(result.real_time_used < 5000);
        assert!(result.real_time_used >= 1000);
        assert_ne!(result.signal, 0);
    }
}
