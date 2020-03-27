use std::result;
use yaml_rust::{ScanError, Yaml, YamlLoader};

#[derive(Debug)]
pub enum Error {
    YamlScanError(ScanError),
    YamlParseError(String),
}

pub type Result<T> = result::Result<T, Error>;

pub struct LanguageConfig {
    pub language: String,
    pub version: String,
    pub compile_command: String,
    pub run_command: String,
}

pub struct Config {
    pub languages: Vec<LanguageConfig>,
}

impl Config {
    fn load_yaml(yaml: &str) -> Result<Config> {
        let docs = match YamlLoader::load_from_str(yaml) {
            Ok(value) => value,
            Err(err) => return Err(Error::YamlScanError(err)),
        };
        let doc = &docs[0];

        // 读取语言配置
        let mut languages: Vec<LanguageConfig> = vec![];
        let yaml_languages = match &doc["languages"] {
            Yaml::Array(value) => value,
            Yaml::BadValue => {
                return Err(Error::YamlParseError(
                    "解析错误，无法解析到 languages 字段".to_string(),
                ))
            }
            _ => {
                return Err(Error::YamlParseError(
                    "解析错误，languages 字段的类型应该为 Array".to_string(),
                ))
            }
        };
        for yaml_language in yaml_languages {
            let language = match &yaml_language["language"] {
                Yaml::String(value) => value.clone(),
                _ => {
                    return Err(Error::YamlParseError(
                        "解析错误，language 字段的类型应该为 String".to_string(),
                    ))
                }
            };
            let version = match &yaml_language["version"] {
                Yaml::String(value) => value.clone(),
                _ => {
                    return Err(Error::YamlParseError(
                        "version 字段的类型应该为 String".to_string(),
                    ))
                }
            };
            let compile_command = match &yaml_language["compile_command"] {
                Yaml::String(value) => value.clone(),
                _ => {
                    return Err(Error::YamlParseError(
                        "compile_command 字段的类型应该为 String".to_string(),
                    ))
                }
            };
            let run_command = match &yaml_language["run_command"] {
                Yaml::String(value) => value.clone(),
                _ => {
                    return Err(Error::YamlParseError(
                        "run_command 字段的类型应该为 String".to_string(),
                    ))
                }
            };
            languages.push(LanguageConfig {
                language,
                version,
                compile_command,
                run_command,
            });
        }

        Ok(Config { languages })
    }
    pub fn default() -> Result<Config> {
        let config_yaml = "
languages:
  - language: c
    version: 7.5.0
    compile_command: /usr/bin/gcc {{filename}} -o a.out
    run_command: ./a.out
  - language: python
    version: 3.6.9
    compile_command: /usr/bin/python3 -m compileall {{filename}}
    run_command: /use/bin/python3 {{filename}}
    ";
        let config = Config::load_yaml(config_yaml)?;
        Ok(config)
    }
}
