use crate::error::{Error, Result};
use std::fs;
use std::fs::{read_to_string, remove_dir};
use std::path::PathBuf;
use tempfile::tempdir_in;

pub struct CGroupOptions {
    path: PathBuf,
}

impl CGroupOptions {
    pub fn new(base_path: &str) -> Result<CGroupOptions> {
        let pwd = try_io!(tempdir_in(base_path));
        Ok(CGroupOptions {
            path: pwd.path().to_path_buf(),
        })
    }

    /// 将指定进程加入该 cgroup 组
    pub fn apply(&self, pid: i32) -> Result<()> {
        self.set("cgroup.procs", &format!("{}", pid))
    }

    /// e.g `set("memory.limit_in_bytes", "67108864")`
    pub fn set(&self, key: &str, value: &str) -> Result<()> {
        try_io!(fs::write(self.path.join(key), value));
        Ok(())
    }

    /// e.g `get("memory.max_usage_in_bytes")`
    pub fn get(&self, key: &str) -> Result<String> {
        Ok(try_io!(read_to_string(self.path.join(key))))
    }
}

impl Drop for CGroupOptions {
    fn drop(&mut self) {
        remove_dir(&self.path).unwrap();
    }
}

/// cgroup v1
pub struct CGroupSet {
    // pub cpuset: CGroupOptions,
    pub memory: CGroupOptions,
}

impl CGroupSet {
    pub fn new() -> Result<CGroupSet> {
        Ok(CGroupSet {
            // cpuset: CGroupOptions::new("/sys/fs/cgroup/cpuset")?,
            memory: CGroupOptions::new("/sys/fs/cgroup/memory")?,
        })
    }
    pub fn apply(&self, pid: i32) -> Result<()> {
        // self.cpuset.apply(pid)?;
        self.memory.apply(pid)?;
        Ok(())
    }
}
