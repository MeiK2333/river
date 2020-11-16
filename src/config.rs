use crate::river::Language;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use std::fs;

pub static STDIN_FILENAME: &str = "stdin.txt";
pub static STDOUT_FILENAME: &str = "stdout.txt";
pub static STDERR_FILENAME: &str = "stderr.txt";

lazy_static! {
    pub static ref LANGUAGES: HashMap<i32, LanguageConf> = {
        let config = fs::read_to_string("languages.yaml").unwrap();
        let ls: Languages = serde_yaml::from_str(&config).unwrap();
        let mut m = HashMap::new();
        // add language
        m.insert(Language::C as i32, ls.C);
        m.insert(Language::Cpp as i32, ls.Cpp);
        m.insert(Language::Python as i32, ls.Python);
        m.insert(Language::Rust as i32, ls.Rust);
        m.insert(Language::Node as i32, ls.Node);
        m.insert(Language::TypeScript as i32, ls.TypeScript);
        m.insert(Language::Go as i32, ls.Go);
        m.insert(Language::Java as i32, ls.Java);
        m
    };
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct LanguageConf {
    pub compile_cmd: String,
    pub code_file: String,
    pub run_cmd: String,
}

#[allow(non_snake_case)]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Languages {
    C: LanguageConf,
    Cpp: LanguageConf,
    Python: LanguageConf,
    Rust: LanguageConf,
    Node: LanguageConf,
    TypeScript: LanguageConf,
    Go: LanguageConf,
    Java: LanguageConf,
}
