use super::error::{Error, Result};
use libc;
use std::ffi::CString;
use std::os::raw::c_char;

pub struct Reader {
    stream: *mut libc::FILE,
}

impl Reader {
    pub fn new(fd: i32) -> Result<Reader> {
        let mode = match CString::new("r") {
            Ok(val) => val,
            Err(e) => return Err(Error::StringToCStringError(e)),
        };
        let stream = unsafe { libc::fdopen(fd, mode.as_ptr()) };
        Ok(Reader { stream: stream })
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
}

impl Drop for Reader {
    fn drop(&mut self) {
        unsafe {
            libc::fclose(self.stream);
        }
    }
}
