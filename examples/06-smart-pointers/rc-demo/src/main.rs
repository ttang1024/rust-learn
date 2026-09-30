// use std::rc::Rc;
// 每当克隆一个 Rc 时，Rc 的引用计数就会增加 1，而每当克隆得到的 Rc 退出作用域时，引用计数就会减少 1。当 Rc 的引用计数变为 0 时，这意味着已经没有所有者，Rc 和值两者都将被删除。

// fn main() {
//     let rc_examples = "Rc examples".to_string();
//     {
//         println!("--- rc_a is created ---");

//         let rc_a: Rc<String> = Rc::new(rc_examples);
//         println!("Reference Count of rc_a: {}", Rc::strong_count(&rc_a)); // 1

//         {
//             println!("--- rc_a is cloned to rc_b ---");

//             let rc_b: Rc<String> = Rc::clone(&rc_a);
//             println!("Reference Count of rc_b: {}", Rc::strong_count(&rc_b)); // 2
//             println!("Reference Count of rc_a: {}", Rc::strong_count(&rc_a)); // 2

//             // 如果两者内部的值相等的话，则两个 `Rc` 相等。
//             println!("rc_a and rc_b are equal: {}", rc_a.eq(&rc_b)); // true

//             // 我们可以直接使用值的方法
//             println!("Length of the value inside rc_a: {}", rc_a.len()); // 11
//             println!("Value of rc_b: {}", rc_b); // Rc examples

//             println!("--- rc_b is dropped out of scope ---");
//         }

//         println!("Reference Count of rc_a: {}", Rc::strong_count(&rc_a)); // 1

//         println!("--- rc_a is dropped out of scope ---");
//     }

//     // 报错！`rc_examples` 已经移入 `rc_a`。
//     // 而且当 `rc_a` 被删时，`rc_examples` 也被一起删除。
//     // println!("rc_examples: {}", rc_examples);
//     // 试一试 ^ 注释掉此行代码
// }

use std::sync::Arc;
use std::thread;

fn main() {
    // 这个变量声明用来指定其值的地方。
    let apple = Arc::new("the same apple");

    for _ in 0..10 {
        // 这里没有数值说明，因为它是一个指向内存堆中引用的指针。
        let apple = Arc::clone(&apple);

        thread::spawn(move || {
            // 由于使用了Arc，线程可以使用分配在 `Arc` 变量指针位置的值来生成。
            println!("{:?}", apple); // ?? "the same apple" 个数是随机的
        });
    }
}


