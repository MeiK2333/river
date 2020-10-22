use super::error::{Error, Result};
use libc;
use std::ffi::CString;
use std::os::raw::c_char;
use std::str;

pub struct Reader {
    fd: i32,
    stream: *mut libc::FILE,
}

impl Reader {
    pub fn new(fd: i32) -> Result<Reader> {
        let mode = match CString::new("r") {
            Ok(val) => val,
            Err(e) => return Err(Error::StringToCStringError(e)),
        };
        let stream = unsafe { libc::fdopen(fd, mode.as_ptr()) };
        Ok(Reader { fd, stream: stream })
    }
    pub fn readline(&self, buf: &mut [u8]) -> Result<()> {
        let res = unsafe {
            libc::fgets(
                buf.as_mut_ptr() as *mut c_char,
                buf.len() as i32,
                self.stream,
            )
        };
        if res.is_null() {
            return Err(Error::SyscallError("fgets".to_string()));
        }
        Ok(())
    }
    pub fn read(&self) -> Result<String> {
        let mut result: String = "".to_owned();
        let mut buf: [u8; 1024] = [0; 1024];
        while unsafe { libc::read(self.fd, buf.as_mut_ptr() as *mut libc::c_void, 1024) } != 0 {
            let s = match str::from_utf8(&buf) {
                Ok(val) => (val),
                Err(_) => return Err(Error::SyscallError("read".to_string())),
            };
            debug!("{}", s);
            result.push_str(s);
        }
        Ok(result)
    }
}

impl Drop for Reader {
    fn drop(&mut self) {
        unsafe {
            libc::fclose(self.stream);
        }
    }
}
