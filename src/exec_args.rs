use super::error::{Error, Result};
use std::env;
use std::ffi::CString;
use std::mem;
use std::ptr;

pub struct ExecArgs {
    pub pathname: *const libc::c_char,
    pub argv: *const *const libc::c_char,
    pub envp: *const *const libc::c_char,
}

impl ExecArgs {
    pub fn build(cmd: &String) -> Result<ExecArgs> {
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