use std::process::{Command, Stdio};
use std::io::Read;
use libc;
use libc::{time_t, suseconds_t, c_long};

fn main() {
    run();
    cgroup();
}

fn cgroup() {
    println!("cgroup");
}

fn run() {
    let cmd = Command::new("echo")
        .arg("-c")
        .arg("s = 'Hello World!' * 100000000")
        .stdout(Stdio::piped())
        .spawn()
        .expect("ls command failed to start");
    let mut usage = libc::rusage {
        ru_utime: libc::timeval { tv_sec: 0 as time_t, tv_usec: 0 as suseconds_t },
        ru_stime: libc::timeval { tv_sec: 0 as time_t, tv_usec: 0 as suseconds_t },
        ru_maxrss: 0 as c_long,
        ru_ixrss: 0 as c_long,
        ru_idrss: 0 as c_long,
        ru_isrss: 0 as c_long,
        ru_minflt: 0 as c_long,
        ru_majflt: 0 as c_long,
        ru_nswap: 0 as c_long,
        ru_inblock: 0 as c_long,
        ru_oublock: 0 as c_long,
        ru_msgsnd: 0 as c_long,
        ru_msgrcv: 0 as c_long,
        ru_nsignals: 0 as c_long,
        ru_nvcsw: 0 as c_long,
        ru_nivcsw: 0 as c_long,
    };
    let pid = cmd.id() as i32;
    let mut status = 0;
    if let Some(mut stdout) = cmd.stdout {
        let mut s: String = String::from("");
        let _ = stdout.read_to_string(&mut s);
        println!("{}", s);
        unsafe {
            let _ = libc::wait4(pid, &mut status, 0, &mut usage);
        }
        let result = usage.ru_utime.tv_sec * 1000 + usage.ru_utime.tv_usec / 1000 +
            usage.ru_stime.tv_sec * 1000 + usage.ru_stime.tv_usec / 1000;
        println!("{}", result);
        println!("{}", usage.ru_maxrss);
    }
}