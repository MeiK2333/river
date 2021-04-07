use tokio::fs::read_to_string;
use tokio::process::Command;

use crate::error::{Error, Result};

#[derive(Debug)]
pub struct ProcessExitStatus {
    pub time_used: i64,
    pub memory_used: i64,
    pub exit_code: i64,
    pub status: i64,
    pub signal: i64,
}

pub struct Sandbox {
    inner_args: Vec<String>,
    workdir: String,
    rootfs: String,
    result: String,
    stdin: String,
    stdout: String,
    stderr: String,
    time_limit: i32,
    memory_limit: i32,
    file_size_limit: i32,
    cgroup: i32,
    pids: i32,
}

impl Sandbox {
    pub fn new(
        cmd: &String,
        workdir: String,
        rootfs: String,
        result: String,
        stdin: String,
        stdout: String,
        stderr: String,
        time_limit: i32,
        memory_limit: i32,
        file_size_limit: i32,
        cgroup: i32,
        pids: i32,
    ) -> Self {
        let inner_args = String::from(cmd)
            .split(" ")
            .map(|s| s.to_string())
            .collect();
        Sandbox {
            inner_args,
            workdir,
            rootfs,
            result,
            stdin,
            stdout,
            stderr,
            time_limit,
            memory_limit,
            file_size_limit,
            cgroup,
            pids,
        }
    }

    pub async fn spawn(&mut self) -> Result<ProcessExitStatus> {
        let mut args = vec![
            String::from("./newbie-sandbox/target/x86_64-unknown-linux-gnu/release/newbie-sandbox"),
            String::from("-w"),
            String::from(&self.workdir),
            String::from("--rootfs"),
            String::from(&self.rootfs),
            String::from("-r"),
            String::from(&self.result),
            String::from("-i"),
            String::from(&self.stdin),
            String::from("-o"),
            String::from(&self.stdout),
            String::from("-e"),
            String::from(&self.stderr),
            String::from("-t"),
            self.time_limit.to_string(),
            String::from("-m"),
            self.memory_limit.to_string(),
            String::from("-f"),
            self.file_size_limit.to_string(),
            String::from("-c"),
            self.cgroup.to_string(),
            String::from("-p"),
            self.pids.to_string(),
            String::from("--"),
        ];
        args.extend_from_slice(&mut self.inner_args);
        info!("args = {:?}", args.join(" "));
        let mut child = try_io!(Command::new(&args[0]).args(&args[1..]).spawn());
        let exit_status = try_io!(child.wait().await);
        if !exit_status.success() {
            return Err(Error::SystemError(String::from("run sandbox error!")));
        }

        let mut time_used = 0;
        let mut memory_used = 0;
        let mut exit_code = 0;
        let mut status = 0;
        let mut signal = 0;

        let text = try_io!(read_to_string(&self.result).await);
        for line in text.split("\n") {
            if !line.contains("=") {
                continue;
            }
            let mut splitter = line.splitn(2, " = ");
            let key = if let Some(s) = splitter.next() {
                s
            } else {
                return Err(Error::StringSplitError());
            };
            let value = if let Some(s) = splitter.next() {
                s
            } else {
                return Err(Error::StringSplitError());
            };
            match key {
                "time_used" => time_used = string_to_i64(value)?,
                "memory_used" => memory_used = string_to_i64(value)?,
                "exit_code" => exit_code = string_to_i64(value)?,
                "status" => status = string_to_i64(value)?,
                "signal" => signal = string_to_i64(value)?,
                _ => continue,
            }
            debug!("{}: {}", key, value);
        }

        Ok(ProcessExitStatus {
            time_used,
            memory_used,
            exit_code,
            status,
            signal,
        })
    }
}

fn string_to_i64(value: &str) -> Result<i64> {
    if let Ok(res) = value.parse() {
        return Ok(res);
    }
    Err(Error::StringToIntError(String::from(value)))
}
