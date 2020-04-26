use super::config;
use super::error::{Error, Result};
use std::fmt;
use std::fs;
use std::io;
use std::path::Path;
use yaml_rust::{Yaml, YamlLoader};

pub struct TestCase {
    pub index: u32,
    pub input_file: String,
    pub answer_file: String,
    pub cpu_time_limit: u32,
    pub real_time_limit: u32,
    pub memory_limit: u32,
}

impl fmt::Display for TestCase {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "index: {}
input_file: {}
answer_file: {}
time_limit: {}
memory_limit: {}
",
            self.index, self.input_file, self.answer_file, self.cpu_time_limit, self.memory_limit
        )
    }
}

pub enum JudgeType {
    Standard,
    Special,
}

pub struct Code {
    pub file: String,
    pub language: config::LanguageConfig,
}

pub struct JudgeConfig {
    pub tests: Vec<TestCase>,
    pub judge_type: JudgeType,
    pub extra_files: Vec<String>,
    pub code: Code,
}

impl JudgeConfig {
    fn load_yaml(global_config: &config::Config, yaml: &str) -> Result<JudgeConfig> {
        let docs = match YamlLoader::load_from_str(yaml) {
            Ok(value) => value,
            Err(err) => return Err(Error::YamlScanError(err)),
        };
        let doc = &docs[0];

        let default_time_limit = match &doc["time_limit"] {
            Yaml::Integer(value) => value.clone() as u32,
            Yaml::BadValue => 1000,
            _ => {
                return Err(Error::YamlParseError(
                    "解析错误，time_limit 字段的类型应该为 Integer".to_string(),
                ))
            }
        };
        let default_memory_limit = match &doc["memory_limit"] {
            Yaml::Integer(value) => value.clone() as u32,
            Yaml::BadValue => 65535,
            _ => {
                return Err(Error::YamlParseError(
                    "解析错误，memory_limit 字段的类型应该为 Integer".to_string(),
                ))
            }
        };
        let test_cases = match &doc["test_cases"] {
            Yaml::Array(value) => value,
            Yaml::BadValue => {
                return Err(Error::YamlParseError(
                    "解析错误，无法解析到 test_cases 字段".to_string(),
                ))
            }
            _ => {
                return Err(Error::YamlParseError(
                    "test_cases 字段的类型应该为 Array".to_string(),
                ))
            }
        };
        let mut tests: Vec<TestCase> = vec![];
        // Yaml 解析库没有实现 enumerate 方法，因此此处使用 index 进行计数
        let mut index = 1;
        for case in test_cases {
            let cpu_time_limit = match &case["time_limit"] {
                Yaml::Integer(value) => value.clone() as u32,
                Yaml::BadValue => default_time_limit,
                _ => {
                    return Err(Error::YamlParseError(
                        "解析错误，time_limit 字段的类型应该为 Integer".to_string(),
                    ))
                }
            };
            let real_time_limit = match &case["real_time_limit"] {
                Yaml::Integer(value) => value.clone() as u32,
                Yaml::BadValue => cpu_time_limit * 2 + 5000,
                _ => {
                    return Err(Error::YamlParseError(
                        "解析错误，real_time_limit 字段的类型应该为 Integer".to_string(),
                    ))
                }
            };
            let memory_limit = match &case["memory_limit"] {
                Yaml::Integer(value) => value.clone() as u32,
                Yaml::BadValue => default_memory_limit,
                _ => {
                    return Err(Error::YamlParseError(
                        "解析错误，memory_limit 字段的类型应该为 Integer".to_string(),
                    ))
                }
            };
            let input_file = match &case["in"] {
                Yaml::String(value) => value.clone(),
                Yaml::BadValue => {
                    return Err(Error::YamlParseError(
                        "解析错误，无法解析到 test_cases::in 字段".to_string(),
                    ))
                }
                _ => {
                    return Err(Error::YamlParseError(
                        "test_cases::in 字段的类型应该为 String".to_string(),
                    ))
                }
            };
            let answer_file = match &case["ans"] {
                Yaml::String(value) => value.clone(),
                Yaml::BadValue => {
                    return Err(Error::YamlParseError(
                        "解析错误，无法解析到 test_cases::ans 字段".to_string(),
                    ))
                }
                _ => {
                    return Err(Error::YamlParseError(
                        "test_cases::ans 字段的类型应该为 String".to_string(),
                    ))
                }
            };
            tests.push(TestCase {
                index,
                cpu_time_limit,
                real_time_limit,
                memory_limit,
                input_file,
                answer_file,
            });
            index += 1;
        }
        let judge_type = match &doc["judge_type"] {
            Yaml::String(value) => {
                if value == "standard" {
                    JudgeType::Standard
                } else if value == "special" {
                    JudgeType::Special
                } else {
                    return Err(Error::UnknownJudgeType(value.clone()));
                }
            }
            Yaml::BadValue => JudgeType::Standard,
            _ => {
                return Err(Error::YamlParseError(
                    "judge_type 字段的类型应该为 String".to_string(),
                ))
            }
        };
        let default_extra_files: Vec<Yaml> = vec![];
        let extra_files_yaml = match &doc["extra_files"] {
            Yaml::Array(value) => value,
            Yaml::BadValue => &default_extra_files,
            _ => {
                return Err(Error::YamlParseError(
                    "extra_files 字段的类型应该为 Array".to_string(),
                ))
            }
        };
        let mut extra_files: Vec<String> = vec![];
        for file in extra_files_yaml {
            let name = match &file {
                Yaml::String(value) => value.clone(),
                Yaml::BadValue => {
                    return Err(Error::YamlParseError(
                        "解析错误，无法解析到 extra_files 字段".to_string(),
                    ))
                }
                _ => {
                    return Err(Error::YamlParseError(
                        "extra_files 字段的类型应该为 Array<String>".to_string(),
                    ))
                }
            };
            extra_files.push(name);
        }
        let code = match &doc["code"] {
            Yaml::Hash(value) => {
                let file = match value.get(&Yaml::from_str("file")) {
                    Some(Yaml::String(val)) => val.clone(),
                    Some(Yaml::BadValue) => {
                        return Err(Error::YamlParseError(
                            "解析错误，无法解析到 code::file 字段".to_string(),
                        ))
                    }
                    None => {
                        return Err(Error::YamlParseError(
                            "必须提供 code::file 字段".to_string(),
                        ))
                    }
                    _ => {
                        return Err(Error::YamlParseError(
                            "code::file 字段的类型应该为 String".to_string(),
                        ))
                    }
                };
                let language = match value.get(&Yaml::from_str("language")) {
                    Some(Yaml::String(val)) => match global_config.language_config_from_name(val) {
                        Ok(v) => v,
                        Err(_) => return Err(Error::LanguageNotFound(val.clone())),
                    },
                    Some(Yaml::BadValue) => {
                        return Err(Error::YamlParseError(
                            "解析错误，无法解析到 code::language 字段".to_string(),
                        ))
                    }
                    None => {
                        return Err(Error::YamlParseError(
                            "必须提供 code::language 字段".to_string(),
                        ))
                    }
                    _ => {
                        return Err(Error::YamlParseError(
                            "code::language 字段的类型应该为 String".to_string(),
                        ))
                    }
                };
                Code { file, language }
            }
            Yaml::BadValue => {
                return Err(Error::YamlParseError(
                    "解析错误，无法解析到 code 字段".to_string(),
                ))
            }
            _ => {
                return Err(Error::YamlParseError(
                    "code 字段的类型应该为 Hash".to_string(),
                ))
            }
        };

        Ok(JudgeConfig {
            tests,
            judge_type,
            extra_files,
            code,
        })
    }
    pub fn load(global_config: &config::Config, path: &str) -> Result<JudgeConfig> {
        let dir = Path::new(&path);

        let config = dir.join("config.yml");
        let config = match config.as_path().to_str() {
            Some(value) => value,
            None => return Err(Error::PathJoinError),
        };
        let config = match fs::read_to_string(&config) {
            Ok(value) => value,
            Err(_) => {
                return Err(Error::ReadFileError(
                    path.to_string(),
                    io::Error::last_os_error().raw_os_error(),
                ))
            }
        };
        JudgeConfig::load_yaml(global_config, &config)
    }
}
