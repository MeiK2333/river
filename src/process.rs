extern crate libc;

use super::error::{errno_str, Error, Result};
use super::judger::{JudgeConfig, TestCase};
use handlebars::Handlebars;
use libc::{c_long, suseconds_t, time_t};
use std::collections::BTreeMap;
use std::env;
use std::ffi::CString;
use std::fs;
use std::io;
use std::mem;
use std::path::Path;
use std::ptr;
use std::time::SystemTime;
use tempfile::tempdir;

pub struct ExecArgs {
    pub pathname: *const libc::c_char,
    pub argv: *const *const libc::c_char,
    pub envp: *const *const libc::c_char,
}

impl ExecArgs {
    fn build(cmd: &String, judge_config: &JudgeConfig) -> Result<ExecArgs> {
        let mut handlebars = Handlebars::new();
        let _ = handlebars.register_template_string("build", cmd);
        let mut data = BTreeMap::new();

        data.insert("filename", judge_config.code.file.clone());

        let formatted = match handlebars.render("build", &data) {
            Ok(val) => val,
            Err(err) => return Err(Error::TemplateRenderError(err)),
        };
        let splited = formatted.split_whitespace();
        let splited: Vec<&str> = splited.collect();

        if splited.len() < 1 {
            return Err(Error::LanguageConfigError(formatted));
        }
        let pathname = splited[0].clone();
        let pathname_str = match CString::new(pathname) {
            Ok(value) => value,
            Err(err) => return Err(Error::StringToCStringError(err)),
        };
        let pathname = pathname_str.as_ptr();

        let mut argv_vec: Vec<*const libc::c_char> = vec![];
        for item in splited.iter() {
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
        // 如果能保证资源能够正确 drop，不会出现泄漏，则 ExecArgs 相关的函数不必标注 unsafe
        println!("Dropping!");
    }
}

pub fn run(judge_config: &JudgeConfig) -> Result<()> {
    let pwd = match tempdir() {
        Ok(val) => val,
        Err(err) => return Err(Error::CreateTempDirError(err)),
    };
    let from_dir = Path::new(&judge_config.config_dir);
    let to_dir = pwd.path();

    let mut copy_files: Vec<String> = vec![];
    copy_files.push(judge_config.code.file.clone());
    copy_files.extend_from_slice(&judge_config.extra_files);
    for test_case in &judge_config.tests {
        copy_files.push(test_case.input_file.clone());
    }
    // copy file
    for file in &copy_files {
        let from_file = from_dir.join(file);
        let to_file = to_dir.join(file);
        match fs::copy(from_file, to_file) {
            Ok(_) => {}
            Err(err) => return Err(Error::CopyFileError(err)),
        };
    }
    let _ = compile(judge_config, to_dir)?;
    for test_case in &judge_config.tests {
        println!("\n----------------------------------\n");
        let _ = run_one(judge_config, test_case, to_dir)?;
        let output_file_path = to_dir.join(&test_case.output_file);
        match fs::remove_file(output_file_path) {
            Ok(_) => {}
            Err(_) => {
                return Err(Error::RemoveFileError(
                    test_case.output_file.clone(),
                    io::Error::last_os_error().raw_os_error(),
                ))
            }
        };
    }
    match pwd.close() {
        Ok(_) => {}
        Err(err) => return Err(Error::CloseTempDirError(err)),
    };
    Ok(())
}

pub struct ExitStatus {
    pub rusage: libc::rusage,
    pub status: i32,
    pub time_used: i64,
    pub real_time_used: u128,
    pub memory_used: i64,
}

pub fn wait(pid: i32) -> Result<ExitStatus> {
    let start = SystemTime::now();
    let mut status = 0;
    let mut rusage = libc::rusage {
        ru_utime: libc::timeval {
            tv_sec: 0 as time_t,
            tv_usec: 0 as suseconds_t,
        },
        ru_stime: libc::timeval {
            tv_sec: 0 as time_t,
            tv_usec: 0 as suseconds_t,
        },
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
    unsafe {
        let val = libc::wait4(pid, &mut status, 0, &mut rusage);
        if val < 0 {
            return Err(Error::WaitError(io::Error::last_os_error().raw_os_error()));
        }
    }
    let time_used = rusage.ru_utime.tv_sec * 1000
        + rusage.ru_utime.tv_usec / 1000
        + rusage.ru_stime.tv_sec * 1000
        + rusage.ru_stime.tv_usec / 1000;
    let memory_used = rusage.ru_maxrss;
    let real_time_used = match start.elapsed() {
        Ok(elapsed) => elapsed.as_millis(),
        Err(err) => return Err(Error::SystemTimeError(err)),
    };
    println!("time used:\t{}", time_used);
    println!("real time used:\t{}", real_time_used);
    println!("memory used:\t{}", rusage.ru_maxrss);
    Ok(ExitStatus {
        rusage,
        status,
        time_used,
        real_time_used,
        memory_used,
    })
}

pub unsafe fn dup(filename: &str, to: libc::c_int, flag: libc::c_int, mode: libc::c_int) {
    let filename_str = CString::new(filename).unwrap();
    let filename = filename_str.as_ptr();
    let fd = libc::open(filename, flag, mode);
    if fd < 0 {
        let err = io::Error::last_os_error().raw_os_error();
        eprintln!("open filure!");
        panic!(errno_str(err));
    }
    if libc::dup2(fd, to) < 0 {
        let err = io::Error::last_os_error().raw_os_error();
        eprintln!("dup2 filure!");
        panic!(errno_str(err));
    }
}

pub fn run_one(judge_config: &JudgeConfig, test_case: &TestCase, workdir: &Path) -> Result<()> {
    println!("running test case: {}", test_case.index);
    let pid;

    // 这个操作就需要 1-2ms 左右，此处预先完成，不占用子进程运行时间
    // 因此，父进程需要 drop 这个结构，需要保证没有内存泄漏
    let exec_args = ExecArgs::build(
        &judge_config.code.language.run_command.clone(),
        &judge_config,
    )?;
    unsafe {
        pid = libc::fork();
    }
    if pid == 0 {
        // 子进程
        // 此处如果出现 Error，则直接程序崩溃，父进程可以收集异常的信息

        // 修改工作目录
        env::set_current_dir(workdir).unwrap();
        unsafe {
            dup(
                &test_case.input_file,
                libc::STDIN_FILENO,
                libc::O_RDONLY,
                0644,
            );
            dup(
                &test_case.output_file,
                libc::STDOUT_FILENO,
                libc::O_CREAT | libc::O_RDWR,
                0644,
            );
            libc::execve(exec_args.pathname, exec_args.argv, exec_args.envp);
        }
        // 理论上，不会走到这里，在上一句 exec 后，程序就已经被替换为待执行程序了
        // 所以， How dare you!
        panic!("How dare you!");
    } else if pid > 0 {
        // 父进程
        let _ = wait(pid)?;
    } else {
        // 异常
        return Err(Error::ForkError(io::Error::last_os_error().raw_os_error()));
    }
    Ok(())
}

pub fn compile(judge_config: &JudgeConfig, workdir: &Path) -> Result<()> {
    let pid;
    unsafe {
        pid = libc::fork();
    }
    if pid == 0 {
        env::set_current_dir(workdir).unwrap();
        let exec_args = ExecArgs::build(
            &judge_config.code.language.compile_command.clone(),
            &judge_config,
        )
        .unwrap();
        unsafe {
            libc::execve(exec_args.pathname, exec_args.argv, exec_args.envp);
        }
        panic!("How dare you!");
    } else if pid > 0 {
        let _ = wait(pid)?;
    } else {
        return Err(Error::ForkError(io::Error::last_os_error().raw_os_error()));
    }
    Ok(())
}
