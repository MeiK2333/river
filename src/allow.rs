use super::seccomp::*;
use libc;

pub fn gen_rules() -> Vec<SyscallRuleSet> {
    vec![
        allow_syscall(libc::SYS_access),
        allow_syscall(libc::SYS_arch_prctl),
        // allow_syscall(libc::SYS_brk),
        allow_syscall(libc::SYS_chmod),
        allow_syscall(libc::SYS_clock_gettime),
        allow_syscall(libc::SYS_clone),
        allow_syscall(libc::SYS_close),
        allow_syscall(libc::SYS_dup),
        allow_syscall(libc::SYS_dup2),
        allow_syscall(libc::SYS_execve),
        allow_syscall(libc::SYS_exit),
        allow_syscall(libc::SYS_exit_group),
        allow_syscall(libc::SYS_fcntl),
        allow_syscall(libc::SYS_fork),
        allow_syscall(libc::SYS_fstat),
        allow_syscall(libc::SYS_futex),
        allow_syscall(libc::SYS_getcwd),
        allow_syscall(libc::SYS_getdents),
        allow_syscall(libc::SYS_getegid),
        allow_syscall(libc::SYS_geteuid),
        allow_syscall(libc::SYS_getgid),
        allow_syscall(libc::SYS_getpid),
        allow_syscall(libc::SYS_gettid),
        allow_syscall(libc::SYS_getrandom),
        allow_syscall(libc::SYS_getrlimit),
        allow_syscall(libc::SYS_getrusage),
        allow_syscall(libc::SYS_getuid),
        allow_syscall(libc::SYS_ioctl),
        allow_syscall(libc::SYS_lseek),
        allow_syscall(libc::SYS_lstat),
        // allow_syscall(libc::SYS_mmap),
        allow_syscall(libc::SYS_mkdir),
        allow_syscall(libc::SYS_mprotect),
        // allow_syscall(libc::SYS_munmap),
        allow_syscall(libc::SYS_open),
        allow_syscall(libc::SYS_openat),
        allow_syscall(libc::SYS_prlimit64),
        allow_syscall(libc::SYS_read),
        allow_syscall(libc::SYS_readlink),
        allow_syscall(libc::SYS_rename),
        allow_syscall(libc::SYS_rt_sigaction),
        allow_syscall(libc::SYS_rt_sigprocmask),
        allow_syscall(libc::SYS_rt_sigreturn),
        allow_syscall(libc::SYS_select),
        allow_syscall(libc::SYS_set_robust_list),
        allow_syscall(libc::SYS_set_tid_address),
        allow_syscall(libc::SYS_sigaltstack),
        allow_syscall(libc::SYS_stat),
        allow_syscall(libc::SYS_sysinfo),
        allow_syscall(libc::SYS_umask),
        allow_syscall(libc::SYS_uname),
        allow_syscall(libc::SYS_unlink),
        allow_syscall(libc::SYS_vfork),
        allow_syscall(libc::SYS_wait4),
        allow_syscall(libc::SYS_write),
        allow_syscall(libc::SYS_writev),
    ]
}

#[inline(always)]
pub fn trace_syscall(syscall_number: i64) -> SyscallRuleSet {
    (
        syscall_number,
        // 为什么是 42？因为 42 是宇宙终极问题的答案
        vec![SeccompRule::new(vec![], SeccompAction::Trace(42))],
    )
}
