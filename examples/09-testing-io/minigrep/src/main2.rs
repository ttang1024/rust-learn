// use std::env;
// use std::fs;

// fn main() {
//     let args: Vec<String> = env::args().collect();
//     let file_path = &args[1];
//     println!("In file {}", file_path);
//     let contents = fs::read_to_string(file_path).expect("Should have been able to read the file");
//     println!("With text:\n{contents}");
// }

// fn main() {
//     let args: Vec<String> = env::args().collect();

//     let config = parse_config(&args);

//     println!("Searching for {}", config.query);
//     println!("In file {}", config.file_path);

//     let contents =
//         fs::read_to_string(config.file_path).expect("Should have been able to read the file");
//     println!("With text:\n{contents}");
// }

// struct Config {
//     query: String,
//     file_path: String,
// }

// fn parse_config(args: &[String]) -> Config {
//     let query = args[1].clone();
//     let file_path = args[2].clone();

//     Config { query, file_path }
// }


use std::env;
use std::fs;
use std::process;

fn main() {
    // 读取传入的参数
    let args: Vec<String> = env::args().collect();

    // 对 build 返回的 `Result` 进行处理
    // unwrap_or_else 是定义在 Result<T,E> 上的常用方法
    // 如果 Result 是 Ok，那该方法就类似 unwrap：返回 Ok 内部的值；如果是 Err，就调用闭包中的自定义代码对错误进行进一步处理
    let config = Config::build(&args).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {err}");
        process::exit(1);
    });
    println!("Searching for {}", config.query);
    println!("In file {}", config.file_path);
    let contents = fs::read_to_string(config.file_path)
        .expect("Should have been able to read the file");
    println!("With text:\n{contents}");
}

struct Config {
    query: String,
    file_path: String,
}

impl Config {
    fn build(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 3 {
            return Err("not enough arguments");
        }

        let query = args[1].clone();
        let file_path = args[2].clone();

        Ok(Config { query, file_path })
    }
}

// 命令行 cargo run -- the a.txt