extern crate libc;

use handlebars::Handlebars;
use std::collections::BTreeMap;
use std::env;

mod config;
mod runner;

fn main() {
    let mut handlebars = Handlebars::new();
    let source = "hello {{world}}";
    let _ = handlebars.register_template_string("t1", source);
    let mut data = BTreeMap::new();
    data.insert("hello".to_string(), "你好".to_string());
    data.insert("world".to_string(), "世界!".to_string());
    println!("{}", handlebars.render("t1", &data).unwrap());

    let args: Vec<String> = env::args().collect();
    let _config = if args.len() > 1 {
        match config::Config::load_from_file(&args[1]) {
            Ok(value) => value,
            Err(err) => {
                println!("{:?}", err);
                return
            }
        }
    } else {
        match config::Config::default() {
            Ok(value) => value,
            Err(err) => {
                println!("{:?}", err);
                return
            }
        }
    };

    process();
}

fn process() {
    let pid;
    unsafe {
        pid = libc::fork();
    }
    let mut run_configs = runner::JudgeConfigs {
        exec_file: "/usr/bin/python3".to_string(),
        exec_args: vec![
            "/usr/bin/python3".to_string(),
            "-c".to_string(),
            "import requests; print(requests.get('https://httpbin.org/get').json())".to_string(),
        ],
        test_cases: vec![],
    };
    if pid == 0 {
        unsafe {
            let exec_args = run_configs.exec_args().unwrap();
            libc::execve(exec_args.pathname, exec_args.argv, exec_args.envp);
        }
    } else if pid > 0 {
        println!("{:?}", pid);
        run_configs.test_cases.push(runner::TestCase {
            answer_file: "1.ans".to_string(),
            input_file: "1.in".to_string(),
            cpu_time_limit: 1000,
            real_time_limit: 1000,
            memory_limit: 65535,
            result: runner::TestCaseResult::Accepted,
        });
        run_configs.test_cases.push(runner::TestCase {
            answer_file: "2.ans".to_string(),
            input_file: "2.in".to_string(),
            cpu_time_limit: 1000,
            real_time_limit: 1000,
            memory_limit: 65535,
            result: runner::TestCaseResult::CompileError("compile error".to_string()),
        });
        run_configs.test_cases.push(runner::TestCase {
            answer_file: "3.ans".to_string(),
            input_file: "3.in".to_string(),
            cpu_time_limit: 1000,
            real_time_limit: 1000,
            memory_limit: 65535,
            result: runner::TestCaseResult::WrongAnswer,
        });
        run_configs.test_cases.push(runner::TestCase {
            answer_file: "4.ans".to_string(),
            input_file: "4.in".to_string(),
            cpu_time_limit: 1000,
            real_time_limit: 1000,
            memory_limit: 65535,
            result: runner::TestCaseResult::RuntimeError("runtime error".to_string()),
        });
        run_configs.test_cases.push(runner::TestCase {
            answer_file: "5.ans".to_string(),
            input_file: "5.in".to_string(),
            cpu_time_limit: 1000,
            real_time_limit: 1000,
            memory_limit: 65535,
            result: runner::TestCaseResult::SystemError("system error".to_string()),
        });
        println!("{:?}", run_configs.test_cases[0].result);
    } else {
        println!("fork failure!");
    }
}