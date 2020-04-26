extern crate libc;

use super::error::{Error, Result};
use super::judger::{JudgeConfig, TestCase};
use handlebars::Handlebars;
use std::collections::BTreeMap;
use std::env;
use std::ffi::CString;
use std::fs;
use std::io;
use std::mem;
use std::path::Path;
use std::ptr;
use std::{thread, time};
use tempfile::tempdir;

pub struct ExecArgs {
    pub pathname: *const libc::c_char,
    pub argv: *const *const libc::c_char,
    pub envp: *const *const libc::c_char,
}

impl ExecArgs {
    fn build<'b>(cmd: &'b String, judge_config: &'b JudgeConfig) -> Result<ExecArgs> {
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
    for test_case in &judge_config.tests {
        let _ = run_one(judge_config, test_case, to_dir);
    }
    match pwd.close() {
        Ok(_) => {}
        Err(err) => return Err(Error::CloseTempDirError(err)),
    };
    Ok(())
}

pub fn run_one(judge_config: &JudgeConfig, test_case: &TestCase, workdir: &Path) -> Result<()> {
    println!("running test case: {}", test_case.index);
    let pid;

    unsafe {
        pid = libc::fork();
    }
    if pid == 0 {
        // 子进程
        // 此处如果出现 Error，则直接程序崩溃，父进程可以收集异常的信息

        // 修改工作目录
        env::set_current_dir(workdir).unwrap();

        let exec_args = ExecArgs::build(
            &judge_config.code.language.run_command.clone(),
            &judge_config,
        )
        .unwrap();
        unsafe {
            libc::execve(exec_args.pathname, exec_args.argv, exec_args.envp);
        }
        // 理论上，不会走到这里，在上一句 exec 后，程序就已经被替换为待执行程序了
        // 所以， How dare you!
        panic!("How dare you!");
    } else if pid > 0 {
        // 父进程
        println!("pid: {}", pid);
        println!("{:?}", workdir);
    } else {
        // 异常
        return Err(Error::ForkError(io::Error::last_os_error().raw_os_error()));
    }
    Ok(())
}
