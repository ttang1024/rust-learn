// #[derive(Debug)]
// struct Point {
//     x: i32,
//     y: i32,
// }

// fn main() {
//     // let mut a = Point { x: 5, y: 5 };
//     // let b = &mut a.x;

//     // *b += 10;
//     // println!("{:?}", b); // 15
//     // println!("{:?}", a); // Point { x: 15, y: 5 }

//     // let mut a = Point { x: 5, y: 5 };
//     // let b = &mut a.x;
//     // *b += 10;
//     // let c = &mut a.y; // at now b is still active. But you can borrow a.y.
//     // *c += 20;
//     // println!("{:?}", c); // 25
//     // println!("{:?}", b); // 15
//     // println!("{:?}", a); // Point { x: 15, y: 25 }

//     // let mut a = vec![1, 2, 3];
//     // let b = &mut a;

//     // // b[0] += 10;
//     // (*b)[0] += 10; // 同样效果

//     // println!("{:?}", b);  // [11, 2, 3]
//     // println!("{:?}", a);  // [11, 2, 3]
// }

// #[derive(Debug)]
// struct Point {
//     x: Box<i32>,
//     y: Box<i32>,
//     z: i32,
// }

// fn main() {
    // let a = Point {
    //     x: Box::new(3),
    //     y: Box::new(4),
    //     z: 5,
    // };
    // let b = a.z; // success
    // println!("{:?}", b); // 5
    // println!("{:?}", a); // Point { x: 3, y: 4, z: 5 }

    // let b = a.x;
    // println!("{:?}", b);
    // println!("{:?}", a); // error value borrowed here after partial move

    // let b = a.x;
    // let c = a.y;
    // println!("{:?}", b); // 3
    // println!("{:?}", c); // 4

    // let mut a = Point {
    //     x: Box::new(3),
    //     y: Box::new(4),
    //     z: 5,
    // };
    // let b = &mut a;
    // b.x = Box::new(10);
    // println!("{:?}", b); // Point { x: 10, y: 4, z: 5 }
    // println!("{:?}", a); // Point { x: 10, y: 4, z: 5 }

    // let b = &mut a.x;
    // *b = Box::new(10);
    // println!("{:?}", b); // 10
    // println!("{:?}", a); // Point { x: 10, y: 4, z: 5 }

    // let mut b = &mut a;
    // let c = &mut b;
    // *c.x += 10;
    // println!("{:?}", c); // Point { x: 13, y: 4, z: 5 }
    // println!("{:?}", b); // Point { x: 13, y: 4, z: 5 }
    // println!("{:?}", a); // Point { x: 13, y: 4, z: 5 }

    // let b = &a;
    // let c = a.x;
    // println!("{:?}", c);
    // println!("{:?}", b);
    // println!("{:?}", a); // Error!  value borrowed here after partial move

    // let b = &mut a;
    // let c = b;
    // println!("{:?}", c);
    // println!("{:?}", b); // Error! borrow of moved value: `b`!
    // println!("{:?}", a);

    // // rust对于不可变引用和可变引用的底层处理机制是完全不同的。也不允许对结构体和其成员分别同时可变引用，以下代码无法编译：
    // let b = &mut a;
    // let c = &mut a.x; // Error!
    // let c = &a.x; // Error!

    // // 如果是不可变引用，则支持连环引用，也支持同时引用结构体本身和其成员，如下代码可编译：
    // let b = &a;
    // let c = &b;
    // let c = &a.x;
    // let c = &b.x;

    // 直接可变借用结构体的不同成员，或者通过可变借用再进行不同成员的可变借用，都可以！可变借用，也可以改变成员的值
    // let b = &mut a.x;
    // let c = &mut a.y;
    // *b = Box::new(30);
    // println!("{:?}", c); // 4
    // println!("{:?}", b); // 30
    // println!("{:?}", a); // Point { x: 30, y: 4, z: 5 }


    // let b = &mut a;
    // let c = &mut b.x;
    // *c = Box::new(20);
    // println!("{:?}", c); // 20
    // println!("{:?}", b); // Point { x: 20, y: 4, z: 5 }
    // println!("{:?}", a); // Point { x: 20, y: 4, z: 5 }
// }

fn main() {
    let x = Box::new(5); // 创建一个Box指针，指向值为5的整数

    let y = *x; // 解引用指针，将值移动到y中
    let y = x.clone(); // 与上面同等效果

    println!("x = {}", x); // x = 5
    println!("y = {}", y); // y = 5

    // x和y现在都已经超出作用域，x的内存将被自动释放
    println!("x2 = {:?}", x); // x2 = 5
    println!("y2 = {:?}", y); // y2 = 5
}
