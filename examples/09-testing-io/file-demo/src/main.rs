// use std::fs::File;
// use std::io::prelude::*;
// use std::path::Path;

// fn main() {
//     // 创建指向所需的文件的路径
//     let path = Path::new("hello.txt");
//     let display = path.display();

//     // 以只读方式打开路径，返回 `io::Result<File>`
//     let mut file = match File::open(&path) {
//         // `io::Error` 的 `description` 方法返回一个描述错误的字符串。
//         Err(why) => panic!("couldn't open {}: {:?}", display, why),
//         Ok(file) => file,
//     };

//     // 读取文件内容到一个字符串，返回 `io::Result<usize>`
//     let mut s = String::new();
//     match file.read_to_string(&mut s) {
//         Err(why) => panic!("couldn't read {}: {:?}", display, why),
//         Ok(_) => print!("{} contains:\n{}", display, s),
//     }

//     // `file` 离开作用域，并且 `hello.txt` 文件将被关闭。
// }

static LOREM_IPSUM: &'static str = "hello world
hello rust
hello spiderMan
";

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

fn main() {
    let path = Path::new("out/lorem_ipsum.txt");
    let display = path.display();

    // 以只写模式打开文件，返回 `io::Result<File>`
    // create 静态方法以只写模式（write-only mode）打开一个文件。若文件已经存在，则旧内容将被销毁。否则，将创建一个新文件。
    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {:?}", display, why),
        Ok(file) => file,
    };

    // 将 `LOREM_IPSUM` 字符串写进 `file`，返回 `io::Result<()>`
    match file.write_all(LOREM_IPSUM.as_bytes()) {
        Err(why) => panic!("couldn't write to {}: {:?}", display, why),
        Ok(_) => println!("successfully wrote to {}", display),
    }
}
