use std::env;

mod error;
mod config;

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
            eprintln!("{:?}", err);
            return
        }
    };
    let language = config.language_config_from_name("c");
    println!("{:?}", language.unwrap());
}
