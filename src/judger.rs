use std::path::Path;

#[cfg(test)]
use std::println as debug;

pub async fn compile(language: &str, code: &str, path: &Path) {
    debug!("language: {}", language);
    debug!("code: {}", code);
    debug!("path: {:?}", path);
}

pub async fn judge(
    language: &str,
    in_file: &str,
    out_file: &str,
    time_limit: i32,
    memory_limit: i32,
    judge_type: i32,
    path: &Path,
) {
    debug!("language: {}", language);
    debug!("in_file: {}", in_file);
    debug!("out_file: {}", out_file);
    debug!("time_limit: {}", time_limit);
    debug!("memory_limit: {}", memory_limit);
    debug!("judge_type: {}", judge_type);
    debug!("path: {:?}", path);
}
