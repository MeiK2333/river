use std::collections::HashMap;
use std::fs;
use std::sync::Arc;

use lazy_static::lazy_static;
use num_cpus;
use serde::{Deserialize, Serialize};
use tokio::sync::Semaphore;

pub static STDIN_FILENAME: &str = "stdin.txt";
pub static STDOUT_FILENAME: &str = "stdout.txt";
pub static STDERR_FILENAME: &str = "stderr.txt";

lazy_static! {
    pub static ref CONFIG: Config = {
        let config = fs::read_to_string("config.yaml").unwrap();
        let cfg: Config = serde_yaml::from_str(&config).unwrap();
        debug!("{:?}", cfg);
        cfg
    };
    pub static ref CPU_SEMAPHORE: Arc<Semaphore> = {
        let num = num_cpus::get();
        info!("cpus = {}", num);
        // 设置最大并发量与 CPU 核数相同，以防止因资源不足而产生系统错误
        Arc::new(Semaphore::new(num))
    };
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct LanguageConf {
    pub compile_cmd: String,
    pub code_file: String,
    pub run_cmd: String,
    pub version: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub data_dir: String,
    pub judge_dir: String,
    pub languages: HashMap<String, LanguageConf>,
}
