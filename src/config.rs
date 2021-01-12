use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

pub static STDIN_FILENAME: &str = "stdin.txt";
pub static STDOUT_FILENAME: &str = "stdout.txt";
pub static STDERR_FILENAME: &str = "stderr.txt";

lazy_static! {
  pub static ref CONFIG: Config = {
    let config = fs::read_to_string("config.yaml").unwrap();
    let cg: Config = serde_yaml::from_str(&config).unwrap();
    debug!("{:?}", cg);
    cg
  };
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct LanguageConf {
  pub compile_cmd: String,
  pub code_file: String,
  pub run_cmd: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
  pub data_dir: String,
  pub languages: HashMap<String, LanguageConf>,
}
