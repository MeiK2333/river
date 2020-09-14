use super::error::{Error, Result};
use super::reader::Reader;
use super::runner::Runner;
use libc;
use std::ffi::c_void;
use std::ffi::CString;
use std::os::raw::c_char;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

pub struct Process {
    // 因为 await 会获取所有权，导致 await 执行完会直接 drop
    // 因此资源不能直接绑定要 await 的目标，否则会在出现在收集需要的信息之前资源已经被回收
    // 同时，直接将结构传递给子进程也有可能会出现 double drop 的情况
    // 因此，此处需要抽离一层
    pub runner: Runner,
}

fn path_buf_str(path_buf: &PathBuf) -> Result<String> {
    let file = match path_buf.file_stem() {
        Some(stem) => match stem.to_str() {
            Some(val) => val.to_string(),
            None => return Err(Error::PathBufToStringError(path_buf.clone())),
        },
        None => return Err(Error::PathBufToStringError(path_buf.clone())),
    };
    Ok(file)
}

impl Process {
    pub fn new(
        cmd: String,
        workdir: PathBuf,
        time_limit: i32,
        memory_limit: i32,
    ) -> Result<Process> {
        let (tx, rx) = mpsc::channel();

        let memfile = path_buf_str(&workdir)?;
        let outfile = match CString::new(format!("{}{}", "stdout", memfile)) {
            Ok(val) => val,
            Err(e) => return Err(Error::StringToCStringError(e)),
        };
        let errfile = match CString::new(format!("{}{}", "stderr", memfile)) {
            Ok(val) => val,
            Err(e) => return Err(Error::StringToCStringError(e)),
        };
        let stdout_fd = unsafe {
            libc::shm_open(
                outfile.as_ptr(),
                libc::O_RDWR | libc::O_CREAT,
                libc::S_IRUSR | libc::S_IWUSR,
            )
        };
        let stderr_fd = unsafe {
            libc::shm_open(
                errfile.as_ptr(),
                libc::O_RDWR | libc::O_CREAT,
                libc::S_IRUSR | libc::S_IWUSR,
            )
        };
        Ok(Process {
            runner: Runner {
                pid: -1,
                time_limit: time_limit,
                memory_limit: memory_limit,
                stdin_fd: None,
                stdout_fd: stdout_fd,
                stderr_fd: stderr_fd,
                cmd: cmd,
                workdir: workdir,
                tx: Arc::new(Mutex::new(tx)),
                rx: Arc::new(Mutex::new(rx)),
            },
        })
    }

    // 为进程设置 stdin 的数据
    pub fn set_stdin(&mut self, in_data: &Vec<u8>) -> Result<()> {
        let memfile = path_buf_str(&self.runner.workdir)?;
        let memfile = format!("{}{}", "stdin", memfile);
        // 打开内存文件
        let fd = unsafe {
            libc::shm_open(
                CString::new(memfile).unwrap().as_ptr(),
                libc::O_RDWR | libc::O_CREAT,
                libc::S_IRUSR | libc::S_IWUSR,
            )
        };
        if fd <= 0 {
            return Err(Error::SyscallError("shm_open".to_string()));
        }
        self.runner.stdin_fd = Some(fd);
        // 扩充内存到数据文件大小
        if unsafe { libc::ftruncate(fd, in_data.len() as i64) } < 0 {
            return Err(Error::SyscallError("ftruncate".to_string()));
        }

        // 复制数据到创建的内存中
        unsafe {
            let ptr = libc::mmap(
                std::ptr::null_mut() as *mut c_void,
                in_data.len(),
                libc::PROT_WRITE,
                libc::MAP_SHARED,
                fd,
                0,
            );
            if ptr == libc::MAP_FAILED {
                return Err(Error::SyscallError("mmap".to_string()));
            }
            libc::strcpy(ptr as *mut c_char, in_data.as_ptr() as *const i8);
            // 复制完要记得 munmap，否则会造成无法回收的内存泄露
            if libc::munmap(ptr, in_data.len()) < 0 {
                return Err(Error::SyscallError("munmap".to_string()));
            }
        }

        Ok(())
    }

    pub fn stdout_reader(&self) -> Result<Reader> {
        Reader::new(self.runner.stdout_fd)
    }
    pub fn stderr_reader(&self) -> Result<Reader> {
        Reader::new(self.runner.stderr_fd)
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        let mut status = 0;
        let pid;
        unsafe {
            pid = libc::waitpid(self.runner.pid, &mut status, libc::WNOHANG);
        }
        // > 0: 对应子进程退出但未回收资源
        // = 0: 对应子进程存在但未退出
        // 如果在运行过程中上层异常中断，则需要 kill 子进程并回收资源
        if pid >= 0 {
            unsafe {
                libc::kill(self.runner.pid, 9);
                libc::waitpid(self.runner.pid, &mut status, 0);
            }
        }
        // 如果设置了 stdin 数据，则需要释放对应的内存
        if let Some(_) = self.runner.stdin_fd {
            // 如果 stdin_fd 有值，则说明 pathbuf 的转换一定没问题，否则上面也不会转换成功
            let memfile = self
                .runner
                .workdir
                .clone()
                .into_os_string()
                .into_string()
                .unwrap();
            unsafe {
                libc::shm_unlink(CString::new(memfile).unwrap().as_ptr());
            }
        }
        // 清理 stdout 和 stderr 的空间
        let memfile = path_buf_str(&self.runner.workdir).unwrap();
        let outfile = CString::new(format!("{}{}", "stdout", memfile)).unwrap();
        let errfile = CString::new(format!("{}{}", "stderr", memfile)).unwrap();
        unsafe {
            libc::shm_unlink(CString::new(outfile).unwrap().as_ptr());
            libc::shm_unlink(CString::new(errfile).unwrap().as_ptr());
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::process::Process;
    use std::fs;
    use tempfile::tempdir_in;

    #[test]
    fn hello() {
        let s = String::from("hello");
        let bytes = s.into_bytes();
        assert_eq!(&[104, 101, 108, 108, 111][..], &bytes[..]);
    }

    #[tokio::test]
    async fn run() {
        let cmd = String::from("/bin/echo hello");
        let pwd = tempdir_in("./runner").unwrap().into_path();
        let path = pwd.to_str().unwrap();
        let process = Process::new(cmd, pwd.clone(), 1000, 65535).unwrap();
        let runner = process.runner.clone();
        let status = runner.await.unwrap();
        fs::remove_dir_all(path).unwrap();
        assert_eq!(status.exit_code, 0);

        let mut buf: [u8; 1024] = [0; 1024];
        let reader = process.stdout_reader().unwrap();
        reader.readline(&mut buf).unwrap();
        let s = String::from("hello\n");
        let bytes = s.into_bytes();
        assert_eq!(&buf[0..6], &bytes[..]);
        assert_eq!(buf[6], 0);
    }
}
