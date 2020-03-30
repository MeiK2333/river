use std::ffi::CString;
use std::ffi::NulError;
use std::fs;
use std::mem;
use std::path::Path;
use std::ptr;
use std::result;
use yaml_rust::{ScanError, Yaml, YamlLoader};

use super::config;

#[derive(Debug)]
pub enum Error {
    StringToCStringError(NulError),
    ReadFileError,
    PathJoinError,
    YamlScanError(ScanError),
    YamlParseError(String),
    UnknownJudgeType(String),
    LanguageNotFound(String),
}

pub type Result<T> = result::Result<T, Error>;

pub struct TestCase {
    pub input_file: String,
    pub answer_file: String,
    pub cpu_time_limit: u32,
    pub real_time_limit: u32,
    pub memory_limit: u32,
    pub result: Option<TestCaseResult>,
}

#[derive(Debug)]
pub enum TestCaseResult {
    Accepted,
    CompileError(String),
    WrongAnswer,
    RuntimeError(String),
    SystemError(String),
}

pub struct JudgeCode {
    pub file: String,
    pub language: config::LanguageConfig,
}

pub enum JudgeType {
    Standard,
    Special,
}

pub struct TestConfig {
    pub default_time_limit: u32,
    pub default_real_time_limit: u32,
    pub default_memory_limit: u32,
    pub tests: Vec<TestCase>,
    pub judge_type: JudgeType,
    pub extra_files: Vec<String>,
    pub code: JudgeCode,
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
    fn load_yaml(global_config: &config::Config, yaml: &str) -> Result<TestConfig> {
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
        let default_real_time_limit = match &doc["real_time_limit"] {
            Yaml::Integer(value) => value.clone() as u32,
            Yaml::BadValue => default_time_limit * 2 + 5000,
            _ => {
                return Err(Error::YamlParseError(
                    "解析错误，real_time_limit 字段的类型应该为 Integer".to_string(),
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
                cpu_time_limit,
                real_time_limit,
                memory_limit,
                result: None,
                input_file,
                answer_file,
            });
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
                JudgeCode { file, language }
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

        Ok(TestConfig {
            default_time_limit,
            default_real_time_limit,
            default_memory_limit,
            tests,
            judge_type,
            extra_files,
            code,
        })
    }
    /**
     * 读取评测文件夹
     */
    pub fn load(global_config: &config::Config, path: &str) -> Result<JudgeConfigs> {
        let dir = Path::new(&path);

        let config = dir.join("config.yml");
        let config = match config.as_path().to_str() {
            Some(value) => value,
            None => return Err(Error::PathJoinError),
        };
        let config = match fs::read_to_string(&config) {
            Ok(value) => value,
            Err(_) => return Err(Error::ReadFileError),
        };
        let _config = JudgeConfigs::load_yaml(global_config, &config)?;

        Ok(JudgeConfigs {
            exec_file: "".to_string(),
            exec_args: vec![],
            test_cases: vec![],
        })
    }
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
