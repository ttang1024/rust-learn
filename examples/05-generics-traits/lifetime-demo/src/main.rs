// struct Interface<'b, 'a: 'b> {
//     manager: &'b mut Manager<'a>,
// }

// impl<'b, 'a: 'b> Interface<'b, 'a> {
//     pub fn noop(self) {
//         println!("interface consumed");
//     }
// }

// struct Manager<'a> {
//     text: &'a str,
// }

// struct List<'a> {
//     manager: Manager<'a>,
// }

// impl<'a> List<'a> {
//     pub fn get_interface<'b>(&'b mut self) -> Interface<'b, 'a>
//     where
//         'a: 'b,
//     {
//         Interface {
//             manager: &mut self.manager,
//         }
//     }
// }

// fn main() {
//     let mut list = List {
//         manager: Manager { text: "hello" },
//     };

//     list.get_interface().noop();

//     println!("Interface should be dropped here and the borrow released");

//     // 下面的调用可以通过，因为Interface的生命周期不需要跟list一样长
//     use_list(&list);
// }

// fn use_list(list: &List) {
//     println!("{}", list.manager.text);
// }

// use std::{slice::from_raw_parts, str::from_utf8_unchecked};

// fn get_memory_location() -> (usize, usize) {
//     // “Hello World” 是字符串字面量，因此它的生命周期是 `'static`.
//     // 但持有它的变量 `string` 的生命周期就不一样了，它完全取决于变量作用域，对于该例子来说，也就是当前的函数范围
//     let string = "Hello World!";
//     let pointer = string.as_ptr() as usize;
//     let length = string.len();
//     (pointer, length)
//     // `string` 在这里被 drop 释放
//     // 虽然变量被释放，无法再被访问，但是数据依然还会继续存活
// }

// fn get_str_at_location(pointer: usize, length: usize) -> &'static str {
//     // 使用裸指针需要 `unsafe{}` 语句块
//     unsafe { from_utf8_unchecked(from_raw_parts(pointer as *const u8, length)) }
// }

// fn main() {
//     let (pointer, length) = get_memory_location();
//     let message = get_str_at_location(pointer, length);
//     println!(
//         "The {} bytes at 0x{:X} stored: {}",
//         length, pointer, message
//     );
//     // 为何处理裸指针需要 `unsafe`，可以试着反注释以下代码
//     // let message = get_str_at_location(1000, 10);
// }

// use std::fmt::Display;

// fn main() {
//   let r1;
//   let r2;
//   {
//     static STATIC_EXAMPLE: i32 = 42;
//     r1 = &STATIC_EXAMPLE;
//     let x = "&'static str";
//     r2 = x;
//     // r1 和 r2 持有的数据都是 'static 的，因此在花括号结束后，并不会被释放
//   }

//   println!("&'static i32: {}", r1); // -> 42
//   println!("&'static str: {}", r2); // -> &'static str

// //   let r3: &str;

//   {
//     let s1 = "String".to_string();

//     // s1 虽然没有 'static 生命周期，但是它依然可以满足 T: 'static 的约束
//     static_bound(&s1);

//     // s1 是 String 类型，没有 'static 的生命周期，因此下面代码会报错
//     // r3 = &s1; //  ^^^ borrowed value does not live long enough

//     // s1 在这里被 drop
//   }
// //   println!("{}", r3);
// }

// fn static_bound<T: Display + 'static>(t: &T) {
//   println!("{}", t);
// }

// fn main() {
//     {
//         let static_string = "I'm in read-only memory";
//         println!("static_string: {}", static_string);

//     }

//     // 当 `static_string` 超出作用域时，该引用不能再被使用，但是数据依然会存在于 binary 所占用的内存中
//     println!("static_string reference remains alive: {}", static_string); //  not found in this scope
// }

// fn main() {
//     {
//         let r;

//         {
//             let x = 5;
//             r = &x;
//         }

//         println!("r: {}", r);
//     }
// }

// fn main() {
//     let string1 = String::from("abcd");
//     let string2 = "xyz";

//     let result = longest(string1.as_str(), string2);
//     println!("The longest string is {}", result);

//     // fn longest(x: &str, y: &str) -> &str {
//     //     if x.len() > y.len() {
//     //         x
//     //     } else {
//     //         y
//     //     }
//     // }

//     // &i32        // 一个引用
//     // &'a i32     // 具有显式生命周期的引用
//     // &'a mut i32 // 具有显式生命周期的可变引用

    // fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    //     if x.len() > y.len() {
    //         x
    //     } else {
    //         y
    //     }
    // }
// }

fn main() {
    fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
        if x.len() > y.len() {
            x
        } else {
            y
        }
    }
    
    let string1 = String::from("long string is long");

    {
        let string2 = String::from("xyz");
        let result = longest(string1.as_str(), string2.as_str());
        println!("The longest string is {}", result);
    }
}
