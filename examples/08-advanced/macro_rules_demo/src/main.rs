// macro_rules! some_rules {
//     // 把“更具体”的规则放在前面
//     ($a:ident++) => {{
//         $a = $a + 1;
//         $a
//     }};
//     ($e:expr) => {
//         $e
//     };
// }

// fn main() {
//     let mut a = 0;
//     println!("{}", some_rules!(a++));
// }

// macro_rules! get_last {
//     ($($i:ident),*) => {
//         get_last!(@internal $($i),*)
//     };
//     (@internal $i0:ident) => {  // 注意把这个规则放在前面
//         $i0
//     };
//     (@internal $i0:ident, $($i:ident),*) => {
//         get_last!(@internal $($i),*)
//     };
// }

// fn main() {
//     let d = 1;
//     println!("{}", get_last!(a, b, c, d));
// }

// macro_rules! create_function {
//     // 此宏接受一个 `ident` 指示符表示的参数，并创建一个名为 `$func_name` 的函数。
//     // `ident` 指示符用于变量名或函数名
//     ($func_name:ident) => {
//         fn $func_name() {
//             // `stringify!` 宏把 `ident` 转换成字符串。
//             println!("You called {:?}()", stringify!($func_name))
//         }
//     };
// }

// // 借助上述宏来创建名为 `foo` 和 `bar` 的函数。
// create_function!(foo); // You called "foo"()
// create_function!(bar); // You called "bar"()

// macro_rules! print_result {
//     // 此宏接受一个 `expr` 类型的表达式，并将它作为字符串，连同其结果一起打印出来。
//     // `expr` 指示符表示表达式。
//     ($expression:expr) => {
//         // `stringify!` 把表达式*原样*转换成一个字符串。
//         println!("{:?} = {:?}", stringify!($expression), $expression)
//     };
// }

// fn main() {
//     foo();
//     bar();

//     print_result!(1u32 + 1); // "1u32 + 1" = 2

//     // 代码块也是表达式！
//     print_result!({
//         let x = 1u32;
//         x * x + 2 * x - 1
//     }); // "{ let x = 1u32; x * x + 2 * x - 1 }" = 2
// }

// // 根据你调用它的方式，`test!` 将以不同的方式来比较 `$left` 和 `$right`。
// macro_rules! test {
//     // 参数不需要使用逗号隔开。 参数可以任意组合！
//     ($left:expr; and $right:expr) => {
//         println!(
//             "{:?} and {:?} is {:?}",
//             stringify!($left),
//             stringify!($right),
//             $left && $right
//         )
//     };
//     // 每个分支都必须以分号结束。
//     ($left:expr; or $right:expr) => {
//         println!(
//             "{:?} or {:?} is {:?}",
//             stringify!($left),
//             stringify!($right),
//             $left || $right
//         )
//     };
// }

// fn main() {
//     test!(1i32 + 1 == 2i32; and 2i32 * 2 == 4i32); // "1i32 + 1 == 2i32" and "2i32 * 2 == 4i32" is true
//     test!(true; or false); // "true" or "false" is true
// }

// // `min!` 将求出任意数量的参数的最小值。
// macro_rules! find_min {
//     // 基本情形：
//     ($x:expr) => ($x);
//     // `$x` 后面跟着至少一个 `$y,`
//     ($x:expr, $($y:expr),+) => (
//         // 对 `$x` 后面的 `$y` 们调用 `find_min!`
//         std::cmp::min($x, find_min!($($y),+))
//     )
// }

// fn main() {
//     println!("{}", find_min!(1u32)); // 1
//     println!("{}", find_min!(1u32 + 2, 2u32)); // 2
//     println!("{}", find_min!(5u32, 2u32 * 3, 4u32)); // 4
// }

// use std::ops::{Add, Mul, Sub};

// macro_rules! assert_equal_len {
//     // `tt`（token tree，标记树）指示符表示运算符和标记。
//     ($a:ident, $b: ident, $func:ident, $op:tt) => {
//         assert!(
//             $a.len() == $b.len(),
//             "{:?}: dimension mismatch: {:?} {:?} {:?}",
//             stringify!($func),
//             ($a.len(),),
//             stringify!($op),
//             ($b.len(),)
//         );
//     };
// }

// macro_rules! op {
//     ($func:ident, $bound:ident, $op:tt, $method:ident) => {
//         fn $func<T: $bound<T, Output = T> + Copy>(xs: &mut Vec<T>, ys: &Vec<T>) {
//             assert_equal_len!(xs, ys, $func, $op);

//             for (x, y) in xs.iter_mut().zip(ys.iter()) {
//                 *x = $bound::$method(*x, *y);
//                 // *x = x.$method(*y); // 效果同上
//             }
//         }
//     };
// }

// // 实现 `add_assign`、`mul_assign` 和 `sub_assign` 等函数。
// op!(add_assign, Add, +=, add);
// op!(mul_assign, Mul, *=, mul);
// op!(sub_assign, Sub, -=, sub);

// mod test {
//     use std::iter;
//     macro_rules! test {
//         ($func: ident, $x:expr, $y:expr, $z:expr) => {
//             #[test]
//             fn $func() {
//                 for size in 0usize..10 {
//                     let mut x: Vec<_> = iter::repeat($x).take(size).collect();
//                     let y: Vec<_> = iter::repeat($y).take(size).collect();
//                     let z: Vec<_> = iter::repeat($z).take(size).collect();

//                     super::$func(&mut x, &y);

//                     assert_eq!(x, z);
//                 }
//             }
//         };
//     }

//     // 测试 `add_assign`、`mul_assign` 和 `sub_assign`
//     test!(add_assign, 1u32, 2u32, 3u32);
//     test!(mul_assign, 2u32, 3u32, 6u32);
//     test!(sub_assign, 3u32, 2u32, 1u32);
// }

// macro_rules! calculate {
//     (eval $e:expr) => {{
//         {
//             let val: usize = $e; // 强制类型为整型
//             println!("{} = {}", stringify!{$e}, val);
//         }
//     }};
// }

// fn main() {
//     calculate! {
//         eval 1 + 2 // `eval` 可并不是 Rust 的关键字！
//     } // 1 + 2 = 3

//     calculate! {
//         eval (1 + 2) * (3 / 4)
//     } // (1 + 2) * (3 / 4) = 0   ??
// }

// 可变参数接口
macro_rules! calculate {
    // 单个 `eval` 的模式
    (eval $e:expr) => {{
        {
            let val: usize = $e; // Force types to be integers
            println!("{} = {}", stringify!{$e}, val);
        }
    }};

    // 递归地拆解多重的 `eval`
    (eval $e:expr, $(eval $es:expr),+) => {{
        calculate! { eval $e }
        calculate! { $(eval $es),+ }
    }};
}

fn main() {
    calculate! { // 可变参数的 `calculate!`！
        eval 1 + 2,
        eval 3 + 4,
        eval (2 * 3) + 1,
        eval 6 + 4
    }
    // 1 + 2 = 3
    // 3 + 4 = 7
    // (2 * 3) + 1 = 7
    // 6 + 4 = 10
}
