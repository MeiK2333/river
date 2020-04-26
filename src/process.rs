extern crate libc;

use super::error::{Error, Result};
use super::judger::{JudgeConfig, TestCase};
use handlebars::Handlebars;
use std::collections::BTreeMap;
use std::ffi::CString;
use std::mem;
use std::ptr;

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

        // env 环境变量传 null，默认不传递任何环境变量，后续有需求可以修改此处
        let envp_vec: Vec<*const libc::c_char> = vec![ptr::null()];
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

pub fn run(judge_config: &JudgeConfig) {
    for test_case in &judge_config.tests {
        run_one(judge_config, test_case);
        println!("{}", test_case);
    }
}

pub fn run_one(judge_config: &JudgeConfig, test_case: &TestCase) {
    println!("running test case: {}", test_case.index);
    
    let pid;
    unsafe {
        pid = libc::fork();
    }
    
    if pid == 0 { // 子进程
        // 此处如果出现 Error，则直接程序崩溃，父进程可以收集异常的信息
        let exec_args = ExecArgs::build(
            &judge_config.code.language.compile_command.clone(),
            &judge_config,
        ).unwrap();
        unsafe {
            libc::execve(exec_args.pathname, exec_args.argv, exec_args.envp);
        }
    } else if pid > 0 { // 父进程
        println!("pid: {}", pid);
    } else { // 异常
    }
}
