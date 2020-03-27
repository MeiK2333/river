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

pub struct TestCase {
    pub input_file: String,
    pub answer_file: String,
    pub cpu_time_limit: u32,
    pub real_time_limit: u32,
    pub memory_limit: u32,
    pub result: TestCaseResult,
}

#[derive(Debug)]
pub enum TestCaseResult {
    Accepted,
    CompileError(String),
    WrongAnswer,
    RuntimeError(String),
    SystemError(String),
}

pub struct JudgeConfigs {
    pub exec_file: String,
    pub exec_args: Vec<String>,
    pub test_cases: Vec<TestCase>,
}

pub struct ExecArgs {
    pub pathname: *const libc::c_char,
    pub argv: *const *const libc::c_char,
    pub envp: *const *const libc::c_char,
}

impl JudgeConfigs {
    /**
     * 为 exec 函数生成参数
     * 涉及到 Rust 到 C 的内存转换，此过程是内存不安全的
     * 请务必手动清理内存，或者仅在马上要执行 exec 的位置执行此函数，以便由操作系统自动回收内存
     */
    pub unsafe fn exec_args(&self) -> Result<ExecArgs> {
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

        Ok(ExecArgs {
            pathname: exec_file_ptr,
            argv: exec_args_ptr,
            envp: env_ptr,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base() {
        let run_args = JudgeConfigs {
            exec_file: "/bin/echo".to_string(),
            exec_args: vec![
                "/bin/echo".to_string(),
                "Hello".to_string(),
                "World".to_string(),
            ],
            test_cases: vec![],
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
