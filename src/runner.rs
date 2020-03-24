use std::ffi::CString;
use std::ffi::NulError;
use std::mem;
use std::ptr;
use std::result;

#[derive(Debug)]
pub enum Error {
    StringToCStringError(NulError),
}

pub type Result<T> = result::Result<T, Error>;

pub struct RunArgs {
    pub exec_file: String,
    pub exec_args: Vec<String>,
    pub cpu_time_limit: u32,
    pub real_time_limit: u32,
    pub memory_limit: u32,
}

pub struct ExecArgs {
    pub pathname: *const libc::c_char,
    pub argv: *const *const libc::c_char,
    pub envp: *const *const libc::c_char,
}

impl RunArgs {
    /**
     * 为 exec 函数生成参数
     * 涉及到 Rust 到 C 的内存转换，此过程是内存不安全的
     * 请务必手动清理内存，或者仅在马上要执行 exec 的位置执行此函数，以便由操作系统自动回收内存
     */
    pub fn exec_args(&self) -> Result<ExecArgs> {
        let exec_file = match CString::new(self.exec_file.clone()) {
            Ok(value) => value,
            Err(err) => return Err(Error::StringToCStringError(err)),
        };
        let exec_file_ptr = exec_file.as_ptr();
        let mut exec_args: Vec<*const libc::c_char> = vec![];
        for item in self.exec_args.iter() {
            let cstr = match CString::new(item.clone()) {
                Ok(value) => value,
                Err(err) => return Err(Error::StringToCStringError(err)),
            };
            let cptr = cstr.as_ptr();
            // 需要使用 mem::forget 来标记
            // 否则在此次循环结束后，cstr 就会被回收，后续 exec 函数无法通过指针获取到字符串内容
            mem::forget(cstr);
            exec_args.push(cptr);
        }
        // argv 与 envp 的参数需要使用 NULL 来标记结束
        exec_args.push(ptr::null());
        let exec_args_ptr: *const *const libc::c_char =
            exec_args.as_ptr() as *const *const libc::c_char;
        let env: Vec<*const libc::c_char> = vec![ptr::null()];
        let env_ptr = env.as_ptr() as *const *const libc::c_char;
        mem::forget(env);
        mem::forget(exec_file);
        mem::forget(exec_args);
        return Ok(ExecArgs {
            pathname: exec_file_ptr,
            argv: exec_args_ptr,
            envp: env_ptr,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base() {
        let run_args = RunArgs {
            exec_file: "/bin/echo".to_string(),
            exec_args: vec![
                "/bin/echo".to_string(),
                "Hello".to_string(),
                "World".to_string(),
            ],
            cpu_time_limit: 1000,
            real_time_limit: 1000,
            memory_limit: 65535,
        };
        unsafe {
            let pid = libc::fork();
            if pid == 0 {
                let exec_args = run_args.exec_args().unwrap();
                libc::execve(exec_args.pathname, exec_args.argv, exec_args.envp);
            }
        }
    }
}
