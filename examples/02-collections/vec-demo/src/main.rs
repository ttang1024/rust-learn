// fn main() {
//     let mut vec = vec![1, 5, 10, 2, 15];
//     vec.sort_unstable();
//     assert_eq!(vec, vec![1, 2, 5, 10, 15]);
// }

// fn main() {
//   let mut vec = vec![1.0, 5.6, 10.3, 2.0, 15f32];
//   vec.sort_unstable();
//   assert_eq!(vec, vec![1.0, 2.0, 5.6, 10.3, 15f32]);
//   // 报错
//   // T: Ord,
//   // |            ^^^ required by this bound in `core::slice::<impl [T]>::sort_unstable`
//   // 在浮点数当中，存在一个 NAN 的值，这个值无法与其他的浮点数进行对比，因此，浮点数类型并没有实现全数值可比较 Ord 的特性，而是实现了部分可比较的特性 PartialOrd。
// }

// fn main() {
//     let mut vec = vec![1.0, 5.6, 10.3, 2.0, 15f32];
//     // partial_cmp 这个方法返回一个被Option包裹的Ordering，在self和other两者之间进行比较。
//     vec.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
//     assert_eq!(vec, vec![1.0, 2.0, 5.6, 10.3, 15f32]);
// }

#[derive(Debug)]
struct Person {
    name: String,
    age: u32,
}

impl Person {
    fn new(name: String, age: u32) -> Person {
        Person { name, age }
    }
}

fn main() {
    let mut people = vec![
        Person::new("Zoe".to_string(), 25),
        Person::new("Al".to_string(), 60),
        Person::new("John".to_string(), 1),
    ];
    // 定义一个按照年龄倒序排序的对比函数
    // people.sort_unstable_by(|a, b| b.age.cmp(&a.age)); // 降序
    people.sort_unstable_by(|a, b| a.age.cmp(&b.age)); // 升序

    // 有些变量通过实现Debug这个trait打印，比如Slices、vectors、options，此时，占位符不再是“{}”，而是“{:?}”。字符串类型已经实现了Debug trait，也可以通过这种方式打印
    println!("{:?}", people); // [Person { name: "Al", age: 60 }, Person { name: "Zoe", age: 25 }, Person { name: "John", age: 1 }]
}
