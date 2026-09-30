// #[warn(dead_code)]
// #[derive(Debug)]
// struct Person {
//     name: String,
//     age: u8,
// }
// fn main() {
//     // print! 将格式化文本输出到标准输出，不带换行符
//     // println! 同上，但是在行的末尾添加换行符
//     // format! 将格式化文本输出到 String 字符串
//     let s = "hello";
//     println!("{}, world", s);
//     let s1 = format!("{}, world", s);
//     print!("{}", s1);
//     print!("{}\n", "!");

//     let i = 3.1415926;
//     let s = String::from("hello");
//     let v = vec![1, 2, 3];
//     let p = Person {
//         name: "sunface".to_string(),
//         age: 18,
//     };
//     println!("{:?}, {:?}, {:?}, {:?}", i, s, v, p);
//     // 3.1415926, "hello", [1, 2, 3], Person { name: "sunface", age: 18 }
// }

// // 为自定义类型实现 Display 特征
// // 实现 Display 特征中的 fmt 方法，即可为自定义结构体 Person 添加自定义输出
// struct Person {
//     name: String,
//     age: u8,
// }

// use std::fmt;
// impl fmt::Display for Person {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(
//             f,
//             "小弟姓名{}，年芳{}",
//             self.name, self.age
//         )
//     }
// }
// fn main() {
//     let p = Person {
//         name: "sunface".to_string(),
//         age: 18,
//     };
//     println!("{}", p); // 小弟姓名sunface，年芳18
// }

// // 在 Rust 中，无法直接为外部类型实现外部特征，但是可以使用newtype解决此问题：
// struct Array(Vec<i32>);

// use std::fmt;
// impl fmt::Display for Array {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "数组是：{:?}", self.0)
//     }
// }
// fn main() {
//     let arr = Array(vec![1, 2, 3]);
//     println!("{}", arr); // 数组是：[1, 2, 3]
// }

// // 位置参数
// // 指定位置的参数去替换某个占位符，例如 {1}，表示用第二个参数替换该占位符(索引从 0 开始)：
// fn main() {
//     println!("{}{}", 1, 2); // =>"12"
//     println!("{1}{0}", 1, 2); // =>"21"
//     // => Alice, this is Bob. Bob, this is Alice
//     println!("{0}, this is {1}. {1}, this is {0}", "Alice", "Bob");
//     println!("{1}{}{0}{}", 1, 2); // => 2112
// }

// // 具名参数
// // 除了像上面那样指定位置外，我们还可以为参数指定名称：
// fn main() {
//     println!("{argument}", argument = "test"); // => "test"
//     println!("{name} {}", 1, name = 2); // => "2 1"
//     println!("{a} {c} {b}", a = "a", b = 'b', c = 3); // => "a 3 b"

//     // 需要注意的是：带名称的参数必须放在不带名称参数的后面，例如下面代码将报错：
//     // println!("{abc} {1}", abc = "def", 2);
//     println!("{abc} {0}", 2, abc = "def"); // def 2
// }

// // 格式化输出，例如只输出浮点数的小数点后两位：
//  fn main() {
//     let v = 3.1415926;
//     // Display => 3.14
//     println!("{:.2}", v);
//     // Debug => 3.14
//     println!("{:.2?}", v);
// }

// // 宽度, 宽度用来指示输出目标的长度，如果长度不够，则进行填充和对齐
// // 字符串填充, 字符串格式化默认使用空格进行填充，并且进行左对齐
// fn main() {
//     // 以下全部输出 "Hello x    !"
//     // 为"x"后面填充空格，补齐宽度5
//     println!("Hello {:5}!", "x"); // Hello x    !
//     // 使用参数5来指定宽度
//     println!("Hello {:1$}!", "x", 5); // Hello x    !
//     // 使用x作为占位符输出内容，同时使用5作为宽度
//     println!("Hello {1:0$}!", 5, "x"); // Hello x    !
//     // 使用有名称的参数作为宽度
//     println!("Hello {:width$}!", "x", width = 5); // Hello x    !

//     // 使用参数5为参数x指定宽度，同时在结尾输出参数5 => Hello x    !5
//     println!("Hello {:1$}!{}", "x", 5); // Hello x    !5

//     // 数字填充:符号和 0
//     // 数字格式化默认也是使用空格进行填充，但与字符串左对齐不同的是，数字是右对齐。
//     // 宽度是5 => Hello     5!
//     println!("Hello {:5}!", 5);
//     // 显式的输出正号 => Hello +5!
//     println!("Hello {:+}!", 5);
//     // 宽度5，使用0进行填充 => Hello 00005!
//     println!("Hello {:05}!", 5);
//     // 负号也要占用一位宽度 => Hello -0005!
//     println!("Hello {:05}!", -5);
// }

// // 对齐
// fn main() {
//     // 以下全部都会补齐5个字符的长度
//     // 左对齐 => Hello x    !
//     println!("Hello {:<5}!", "x");
//     // 右对齐 => Hello     x!
//     println!("Hello {:>5}!", "x");
//     // 居中对齐 => Hello   x  !
//     println!("Hello {:^5}!", "x");

//     // 对齐并使用指定符号填充 => Hello x&&&&!
//     println!("Hello {:&<5}!", "x");
// }

fn main() {
    let v = 3.1415926;
    // 保留小数点后两位 => 3.14
    println!("{:.2}", v);
    // 带符号保留小数点后两位 => +3.14
    println!("{:+.2}", v);
    // 不带小数 => 3
    println!("{:.0}", v);
    // 通过参数来设定精度 => 3.1416，相当于{:.4}
    println!("{:.1$}", v, 4);

    let s = "hi我是Sunface孙飞";
    // 保留字符串前三个字符 => hi我
    println!("{:.3}", s);
    // {:.*}接收两个参数，第一个是精度，第二个是被格式化的值 => Hello abc!
    println!("Hello {:.*}!", 3, "abcdefg");
}
