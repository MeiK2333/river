use std::env;

mod config;
mod error;
mod judger;
mod process;
mod result;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = if args.len() > 1 {
        config::Config::load_from_file(&args[1])
    } else {
        config::Config::default()
    };
    let config = match config {
        Ok(value) => value,
        Err(err) => {
            eprintln!("{}", err);
            return;
        }
    };
    let language = config.language_config_from_name("python");
    println!("{}", language.unwrap());

    let judge_config = match judger::JudgeConfig::load(&config, "example") {
        Ok(value) => value,
        Err(err) => {
            eprintln!("{}", err);
            return;
        }
    };
    println!("{}", judge_config.code.language);

    match process::run(&judge_config) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("{}", err);
            return;
        }
    };
}
