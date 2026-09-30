// #[derive(Debug)]
// struct Rectangle {
//     width: u32,
//     height: u32,
// }

// impl Rectangle {
//     fn area(&self) -> u32 {
//         self.width * self.height
//     }
// }

// fn main() {
//     let rect1 = Rectangle {
//         width: 30,
//         height: 50,
//     };

//     println!("{:?}", rect1.area())
// }

// impl Rectangle {
//     fn width(&self) -> bool {
//         self.width > 0
//     }
// }

// fn main() {
//     let rect1 = Rectangle {
//         width: 0,
//         height: 50,
//     };

//     if rect1.width() {
//         println!("The rectangle has a nonzero width; it is {}", rect1.width);
//     } else {
//         println!("？？？？？")
//     }
// }

// impl Rectangle {
//     pub fn new(width: u32, height: u32) -> Self {
//         Rectangle { width, height }
//     }
//     pub fn width(&self) -> u32 {
//         return self.width;
//     }
// }

// fn main() {
//     let rect1 = Rectangle::new(30, 50);

//     println!("{}", rect1.width()); // 30
// }

// impl Rectangle {
//     fn area(&self) -> u32 {
//         self.width * self.height
//     }

//     fn can_hold(&self, other: &Rectangle) -> bool {
//         self.width > other.width && self.height > other.height
//     }
// }

// fn main() {
//     let rect1 = Rectangle {
//         width: 30,
//         height: 50,
//     };
//     let rect2 = Rectangle {
//         width: 10,
//         height: 40,
//     };
//     let rect3 = Rectangle {
//         width: 60,
//         height: 45,
//     };

//     println!("Can rect1 hold rect2? {}", rect1.can_hold(&rect2)); // true
//     println!("Can rect1 hold rect3? {}", rect1.can_hold(&rect3)); // false
// }

// impl Rectangle {
//     fn new(w: u32, h: u32) -> Rectangle {
//         Rectangle { width: w, height: h }
//     }
// }

// fn main() {
//     let sq = Rectangle::new(1, 2);

//     let rect1 = Rectangle {
//         width: 22,
//         height: 33
//     };

//     println!("{:?}", sq); // Rectangle { width: 1, height: 2 }

//     println!("{:?}", rect1); // Rectangle { width: 22, height: 33 }
// }

#[derive(Debug)]
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(i32, i32, i32),
}

impl Message {
    fn call(&self) {
        // 在这里定义方法体
        println!("{:?}", &self);
    }
}

fn main() {
    let m = Message::Write(String::from("hello"));
    m.call(); // Write("hello")

    let n = Message::ChangeColor(24, 44, 344);
    n.call(); // ChangeColor(24, 44, 344)

    let o = Message::Move { x: 1, y: 2 };
    o.call(); // Move { x: 1, y: 2 }
}
