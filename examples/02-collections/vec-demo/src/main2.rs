// #![allow(unused)]
// fn main() {
//     // // Vec::new 创建动态数组
//     // let mut v = Vec::new();
//     // v.push(1);
//     // println!("{:?}", v);
//     // // 使用宏 vec! 来创建数组，能在创建同时给予初始化值：
//     // let v2 = vec![1, 2, 3];
//     // println!("{:?}", v2);

//     // // 读取指定位置的元素有两种方式可选：
//     // // 通过下标索引访问。
//     // // 使用 get 方法。
//     // let v3 = vec![1, 2, 3, 4, 5];

//     // let third: &i32 = &v3[2];
//     // println!("第三个元素是 {}", third);

//     // match v3.get(2) {
//     //     Some(third) => println!("第三个元素是 {third}"),
//     //     None => println!("去你的第三个元素，根本没有！"),
//     // }

//     // // &v[100] 的访问方式会导致程序无情报错退出，因为发生了数组越界访问。
//     // // v.get 就不会，它在内部做了处理，有值的时候返回 Some(T)，无值的时候返回 None，因此 v.get 的使用方式非常安全。
//     // let v = vec![1, 2, 3, 4, 5];
//     // // let does_not_exist = &v[100];
//     // // println!("{:?}", does_not_exist); // index out of bounds: the len is 5 but the index is 100

//     // let does_not_exist = v.get(100);
//     // println!("{:?}", does_not_exist); // None

//     // let v = vec![1, 2, 3];
//     // for i in &v {
//     //     println!("{i}");
//     // }

//     // let mut v = vec![1, 2, 3];
//     // for i in &mut v {
//     //     *i += 10
//     // }

//     // println!("{:?}", v); // [11, 12, 13]

// }

// 数组
// #[derive(Debug)]
// enum IpAddr {
//     V4(String),
//     V6(String),
// }
// fn main() {
//     let v = vec![
//         IpAddr::V4("127.0.0.1".to_string()),
//         IpAddr::V6("::1".to_string()),
//     ];

//     for ip in v {
//         show_addr(ip)
//     }
// }

// fn show_addr(ip: IpAddr) {
//     println!("{:?}", ip);
// }

// // 特征对象的实现
// trait IpAddr {
//     fn display(&self);
// }

// struct V4(String);
// impl IpAddr for V4 {
//     fn display(&self) {
//         println!("ipv4: {:?}", self.0)
//     }
// }
// struct V6(String);
// impl IpAddr for V6 {
//     fn display(&self) {
//         println!("ipv6: {:?}", self.0)
//     }
// }

// fn main() {
//     let v: Vec<Box<dyn IpAddr>> = vec![
//         Box::new(V4("127.0.0.1".to_string())),
//         Box::new(V6("::1".to_string())),
//     ];

//     for ip in v {
//         ip.display();
//     }
// }

// fn main() {
//     let mut v = Vec::with_capacity(10);
//     v.extend([1, 2, 3]); // 附加数据到 v
//     println!("Vector 长度是: {}, 容量是: {}", v.len(), v.capacity()); // 3, 10

//     v.reserve(100); // 调整 v 的容量，至少要有 100 的容量
//     println!(
//         "Vector（reserve） 长度是: {}, 容量是: {}",
//         v.len(),
//         v.capacity()
//     ); // 3, 103

//     v.shrink_to_fit(); // 释放剩余的容量，一般情况下，不会主动去释放容量
//     println!(
//         "Vector（shrink_to_fit） 长度是: {}, 容量是: {}",
//         v.len(),
//         v.capacity()
//     ); // 3, 3

// }

// #![allow(unused)]
// fn main() {
//     let mut v = vec![1, 2];
//     assert!(!v.is_empty()); // 检查 v 是否为空

//     v.insert(2, 3); // 在指定索引插入数据，索引值不能大于 v 的长度， v: [1, 2, 3]
//     assert_eq!(v.remove(1), 2); // 移除指定位置的元素并返回, v: [1, 3]
//     assert_eq!(v.pop(), Some(3)); // 删除并返回 v 尾部的元素，v: [1]
//     assert_eq!(v.pop(), Some(1)); // v: []
//     assert_eq!(v.pop(), None); // 记得 pop 方法返回的是 Option 枚举值
//     v.clear(); // 清空 v, v: []

//     let mut v1 = [11, 22].to_vec(); // append 操作会导致 v1 清空数据，增加可变声明
//     v.append(&mut v1); // 将 v1 中的所有元素附加到 v 中, v1: []
//     v.truncate(1); // 截断到指定长度，多余的元素被删除, v: [11]
//     v.retain(|x| *x > 10); // 保留满足条件的元素，即删除不满足条件的元素

//     let mut v = vec![11, 22, 33, 44, 55];
//     // 删除指定范围的元素，同时获取被删除元素的迭代器, v: [11, 55], m: [22, 33, 44]
//     let mut m: Vec<_> = v.drain(1..=3).collect();

//     let v2 = m.split_off(1); // 指定索引处切分成两个 vec, m: [22], v2: [33, 44]
// }

// use std::fmt::Display;

// fn main() {
//     // let v = vec![1, 2, 3, 4, 5];

//     // let third: &i32 = &v[2];
//     // println!("第三个元素是 {}", third);

//     // match v.get(2) {
//     //     Some(third) => println!("第三个元素是 {third}"),
//     //     None => println!("去你的第三个元素，根本没有！"),
//     // }
//     // let v = vec![1, 2, 3, 4, 5];

//     // let does_not_exist = &v[100]; // index out of bounds: the len is 5 but the index is 100
//     // println!("does_not_exist {}", does_not_exist);
//     // let does_not_exist2 = v.get(100);

//     //  match does_not_exist2 {
//     //     Some(third) => println!("第这个元素是什么"),
//     //     None => println!("这个元素根本没有！"),   // 这个元素根本没有！
//     // }

//     let mut v = vec![1, 2, 3, 4, 5];

//     let first = &v[0];

//     v.push(6);

//     // println!("The first element is: {first}"); //  immutable borrow later used here

//     println!("v: {}", v);
// }

// use std::fmt; // 导入 `fmt` 模块。

// // 定义一个包含单个 `Vec` 的结构体 `List`。
// struct List(Vec<i32>);

// impl fmt::Display for List {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         // 使用元组的下标获取值，并创建一个 `vec` 的引用
//         // 创建一个指向结构体 List 中第一个元素的不可变引用
// 		// self 是一个 &List 类型的引用，它表示一个指向类型为 List 的结构体实例的引用
// 		// .0 表示取得 List 结构体中的第一个元素
// 		// 必须使用 &self.0 来获取 List 这个引用所指向的 Vec<i32> 实例
//         let vec = &self.0;

// 		// ? 运算符，表示在发生错误时返回错误信息
// 		// 这样在上层函数中可以进行进一步的处理
// 		// ? 运算符只有在当前函数返回 Result 类型时才能使用
//         write!(f, "[")?;

//         // 使用 `v` 对 `vec` 进行迭代，并用 `count` 记录迭代次数。
//         for (count, v) in vec.iter().enumerate() {
//             // 对每个元素（第一个元素除外）加上逗号。
//             // 使用 `?` 或 `try!` 来返回错误。
//             if count != 0 { write!(f, ", ")?; }
//             write!(f, "{}: {}", count, v)?;
//         }

//         // 加上配对中括号，并返回一个 fmt::Result 值
//         write!(f, "]")
//     }
// }

// fn main() {
//     let v = List(vec![1, 2, 3]);
//     println!("{}", v);
// }

// fn main() {
//     // let v = vec![1, 2, 3];
//     // for i in &v {
//     //     println!("{i}"); // 1 2 3
//     // }

//     let mut v = vec![1, 2, 3];
//     for i in &mut v {
//         *i += 10
//     }

//     for i in &v {
//         println!("{i}"); // 11 12 13
//     }
// }

// #[derive(Debug)]
// enum IpAddr {
//     V4(String),
//     V6(String),
// }
// fn main() {
//     let v = vec![
//         IpAddr::V4("127.0.0.1".to_string()),
//         IpAddr::V6("::1".to_string()),
//     ];

//     for ip in v {
//         show_addr(ip)
//     }
// }

// fn show_addr(ip: IpAddr) {
//     println!("{:?}", ip);
// }

// trait IpAddr {
//     fn display(&self);
// }

// struct V4(String);
// impl IpAddr for V4 {
//     fn display(&self) {
//         println!("ipv4: {:?}", self.0)
//     }
// }
// struct V6(String);
// impl IpAddr for V6 {
//     fn display(&self) {
//         println!("ipv6: {:?}", self.0)
//     }
// }

// fn main() {
//     let v: Vec<Box<dyn IpAddr>> = vec![
//         Box::new(V4("127.0.0.1".to_string())),
//         Box::new(V6("::1".to_string())),
//     ];

//     for ip in v {
//         ip.display();
//     }
// }

// fn main() {
//     let v = vec![0; 3]; // 默认值为 0，初始长度为 3
//     let v_from = Vec::from([0, 0, 0]);
//     assert_eq!(v, v_from);
// }

// fn main() {
//     let mut v = Vec::with_capacity(10);
//     v.extend([1, 2, 3]); // 附加数据到 v
//     println!("Vector 长度是: {}, 容量是: {}", v.len(), v.capacity()); // 3 10

//     v.reserve(100); // 调整 v 的容量，至少要有 100 的容量
//     println!(
//         "Vector（reserve） 长度是: {}, 容量是: {}",
//         v.len(),
//         v.capacity()
//     ); // 3 103

//     v.shrink_to_fit(); // 释放剩余的容量，一般情况下，不会主动去释放容量
//     println!(
//         "Vector（shrink_to_fit） 长度是: {}, 容量是: {}",
//         v.len(),
//         v.capacity()
//     ); // 3 3
// }

fn main() {
    let mut v = vec![1, 2];
    assert!(!v.is_empty()); // 检查 v 是否为空

    v.insert(2, 3); // 在指定索引插入数据，索引值不能大于 v 的长度， v: [1, 2, 3]
    assert_eq!(v.remove(1), 2); // 移除指定位置的元素并返回, v: [1, 3]
    assert_eq!(v.pop(), Some(3)); // 删除并返回 v 尾部的元素，v: [1]
    assert_eq!(v.pop(), Some(1)); // v: []
    assert_eq!(v.pop(), None); // 记得 pop 方法返回的是 Option 枚举值
    v.clear(); // 清空 v, v: []

    let mut v1 = [11, 22].to_vec(); // append 操作会导致 v1 清空数据，增加可变声明
    v.append(&mut v1); // 将 v1 中的所有元素附加到 v 中, v1: []
    v.truncate(1); // 截断到指定长度，多余的元素被删除, v: [11]
    v.retain(|x| *x > 10); // 保留满足条件的元素，即删除不满足条件的元素

    let mut v = vec![11, 22, 33, 44, 55];
    // 删除指定范围的元素，同时获取被删除元素的迭代器, v: [11, 55], m: [22, 33, 44]
    let mut m: Vec<_> = v.drain(1..=3).collect();

    let _v2 = m.split_off(1); // 指定索引处切分成两个 vec, m: [22], v2: [33, 44]
}
