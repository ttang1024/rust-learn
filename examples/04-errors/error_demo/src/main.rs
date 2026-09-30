// fn give_princess(gift: &str) {
//     if gift == "snake" {
//         panic!("AAAaaaaa!!!!");
//     }

//     println!("I love {}s!!!!!", gift);
// }

// fn main() {
//     give_princess("teddy bear");
//     give_princess("snake");
// }

// // 显式地使用 `match` 来处理。
// fn give_commoner(gift: Option<&str>) {
//     // 指出每种情况下的做法。
//     match gift {
//         Some("snake") => println!("Yuck! I'm throwing that snake in a fire."),
//         Some(inner) => println!("{}? How nice.", inner),
//         None => println!("No gift? Oh well."),
//     }
// }

// // 使用 `unwrap` 隐式地处理。
// fn give_princess(gift: Option<&str>) {
//     // `unwrap` 在接收到 `None` 时将返回 `panic`。
//     let inside = gift.unwrap();
//     if inside == "snake" {
//         panic!("AAAaaaaa!!!!");
//     }

//     println!("I love {}s!!!!!", inside);
// }

// fn main() {
//     let food = Some("chicken");
//     let snake = Some("snake");
//     let void = None;

//     give_commoner(food); // chicken? How nice.
//     give_commoner(snake); // Yuck! I'm throwing that snake in a fire.
//     give_commoner(void); // No gift? Oh well.

//     let bird = Some("robin");
//     let nothing = None;

//     give_princess(bird); // I love robins!!!!!
//     give_princess(nothing); // thread 'main' panicked at src/main.rs:29:23:   called `Option::unwrap()` on a `None` value
// }

// fn next_birthday(current_age: Option<u8>) -> Option<String> {
//     // 如果 `current_age` 是 `None`，这将返回 `None`。
//     // 如果 `current_age` 是 `Some`，内部的 `u8` 将赋值给 `next_age`。
//     let next_age: u8 = current_age?;
//     Some(format!("Next year I will be {}", next_age))
// }

// fn main() {
//     println!("{}", next_birthday(Some(32)).unwrap()); // Next year I will be 32
//     println!("{}", next_birthday(None).unwrap()); // thread 'main' panicked at src/main.rs:60:40: called `Option::unwrap()` on a `None` value
// }

// struct Person {
//     job: Option<Job>,
// }

// #[derive(Clone, Copy)]
// struct Job {
//     phone_number: Option<PhoneNumber>,
// }

// #[derive(Clone, Copy)]
// struct PhoneNumber {
//     area_code: Option<u8>,
//     number: u32,
// }

// impl Person {
//     // 获取此人的工作电话号码的区号（如果存在的话）。
//     fn work_phone_area_code(&self) -> Option<u8> {
//         // 没有`？`运算符的话，这将需要很多的嵌套的 `match` 语句。
//         self.job?.phone_number?.area_code
//     }
// }

// fn main() {
//     let p = Person {
//         job: Some(Job {
//             phone_number: Some(PhoneNumber {
//                 area_code: Some(61),
//                 number: 439222222,
//             }),
//         }),
//     };

//     println!("{:?}", p.work_phone_area_code()); // Some(61)
// }

// #![allow(dead_code)]

// #[derive(Debug)]
// enum Food {
//     Apple,
//     Carrot,
//     Potato,
// }

// #[derive(Debug)]
// struct Peeled(Food);
// #[derive(Debug)]
// struct Chopped(Food);
// #[derive(Debug)]
// struct Cooked(Food);

// // 削皮。如果没有食物，就返回 `None`。否则返回削好皮的食物。
// fn peel(food: Option<Food>) -> Option<Peeled> {
//     match food {
//         Some(food) => Some(Peeled(food)),
//         None => None,
//     }
// }

// // 切食物。如果没有食物，就返回 `None`。否则返回切好的食物。
// fn chop(peeled: Option<Peeled>) -> Option<Chopped> {
//     match peeled {
//         Some(Peeled(food)) => Some(Chopped(food)),
//         None => None,
//     }
// }

// // 烹饪食物。这里，我们使用 `map()` 来替代 `match` 以处理各种情况。
// fn cook(chopped: Option<Chopped>) -> Option<Cooked> {
//     chopped.map(|Chopped(food)| Cooked(food))
// }

// // 这个函数会完成削皮切块烹饪一条龙。我们把 `map()` 串起来，以简化代码。
// fn process(food: Option<Food>) -> Option<Cooked> {
//     food.map(|f| Peeled(f))
//         .map(|Peeled(f)| Chopped(f))
//         .map(|Chopped(f)| Cooked(f))
// }

// // 在尝试吃食物之前确认食物是否存在是非常重要的！
// fn eat(food: Option<Cooked>) {
//     match food {
//         Some(food) => println!("Mmm. I love {:?}", food),
//         None => println!("Oh no! It wasn't edible."),
//     }
// }

// fn main() {
//     let apple = Some(Food::Apple);
//     let carrot = Some(Food::Carrot);
//     let potato = None;

//     let cooked_apple = cook(chop(peel(apple)));
//     // let cooked_carrot = cook(chop(peel(carrot)));
//     let cooked_carrot = process(carrot); // 结果同上

//     // 看起来更简单的 `process()`
//     let cooked_potato = process(potato);

//     eat(cooked_apple); // Mmm. I love Cooked(Apple)
//     eat(cooked_carrot); // Mmm. I love Cooked(carrot)
//     eat(cooked_potato); // Oh no! It wasn't edible.
// }

// #![allow(dead_code)]

// #[derive(Debug)]
// enum Food {
//     CordonBleu,
//     Steak,
//     Sushi,
// }
// #[derive(Debug)]
// enum Day {
//     Monday,
//     Tuesday,
//     Wednesday,
// }

// // 没有制作寿司所需的原材料（ingredient）（有其他的原材料）。
// fn have_ingredients(food: Food) -> Option<Food> {
//     match food {
//         Food::Sushi => None,
//         _ => Some(food),
//     }
// }

// // 拥有全部食物的食谱，除了法国蓝带猪排（Cordon Bleu）的。
// fn have_recipe(food: Food) -> Option<Food> {
//     match food {
//         Food::CordonBleu => None,
//         _ => Some(food),
//     }
// }

// // 一系列 `match` 来表达这个逻辑：
// fn cookable_v1(food: Food) -> Option<Food> {
//     match have_ingredients(food) {
//         None => None,
//         Some(food) => match have_recipe(food) {
//             None => None,
//             Some(food) => Some(food),
//         },
//     }
// }

// // 也可以使用 `and_then()` 把上面的逻辑改写得更紧凑：
// fn cookable_v2(food: Food) -> Option<Food> {
//     have_ingredients(food).and_then(have_recipe)
// }

// fn eat(food: Food, day: Day) {
//     match cookable_v2(food) {
//         Some(food) => println!("Yay! On {:?} we get to eat {:?}.", day, food),
//         None => println!("Oh no. We don't get to eat on {:?}?", day),
//     }
// }

// fn main() {
//     let (cordon_bleu, steak, sushi) = (Food::CordonBleu, Food::Steak, Food::Sushi);

//     eat(cordon_bleu, Day::Monday); // Oh no. We don't get to eat on Monday?
//     eat(steak, Day::Tuesday); // Yay! On Tuesday we get to eat Steak.
//     eat(sushi, Day::Wednesday); // Oh no. We don't get to eat on Wednesday?
// }

// fn multiply(first_number_str: &str, second_number_str: &str) -> i32 {
//     let first_number = first_number_str.parse::<i32>().unwrap();
//     let second_number = second_number_str.parse::<i32>().unwrap();
//     first_number * second_number
// }

// fn main() {
//     let twenty = multiply("10", "2");
//     println!("double is {}", twenty); // double is 20

//     let tt = multiply("t", "2");
//     println!("double is {}", tt); // called `Result::unwrap()` on an `Err` value: ParseIntError { kind: InvalidDigit }
// }

// use std::num::ParseIntError;

// // 就像 `Option` 那样，我们可以使用 `map()` 之类的组合算子。
// // 除去写法外，这个函数与上面那个完全一致，它的作用是：
// // 如果值是合法的，计算其乘积，否则返回错误。
// fn multiply(first_number_str: &str, second_number_str: &str) -> Result<i32, ParseIntError> {
//     first_number_str.parse::<i32>().and_then(|first_number| {
//         second_number_str
//             .parse::<i32>()
//             .map(|second_number| first_number * second_number)
//     })
// }

// fn print(result: Result<i32, ParseIntError>) {
//     match result {
//         Ok(n) => println!("n is {}", n),
//         Err(e) => println!("Error: {}", e),
//     }
// }

// fn main() {
//     // 这种情况下仍然会给出正确的答案。
//     let twenty = multiply("10", "2");
//     print(twenty); // n is 20

//     // 这种情况下就会提供一条更有用的错误信息。
//     let tt = multiply("t", "2");
//     print(tt); // Error: invalid digit found in string
// }

// use std::num::ParseIntError;

// // 为带有错误类型 `ParseIntError` 的 `Result` 定义一个泛型别名。
// type AliasedResult<T> = Result<T, ParseIntError>;

// // 使用上面定义过的别名来表示上一节中的 `Result<i32,ParseIntError>` 类型。
// fn multiply(first_number_str: &str, second_number_str: &str) -> AliasedResult<i32> {
//     first_number_str.parse::<i32>().and_then(|first_number| {
//         second_number_str
//             .parse::<i32>()
//             .map(|second_number| first_number * second_number)
//     })
// }

// // 在这里使用别名又让我们节省了一些代码量。
// fn print(result: AliasedResult<i32>) {
//     match result {
//         Ok(n) => println!("n is {}", n),
//         Err(e) => println!("Error: {}", e),
//     }
// }

// fn main() {
//     print(multiply("10", "2")); // n is 20
//     print(multiply("t", "2")); // Error: invalid digit found in string
// }

// use std::num::ParseIntError;

// fn multiply(first_number_str: &str, second_number_str: &str) -> Result<i32, ParseIntError> {
//     let first_number = match first_number_str.parse::<i32>() {
//         Ok(first_number) => first_number,
//         Err(e) => return Err(e),
//     };

//     let second_number = match second_number_str.parse::<i32>() {
//         Ok(second_number) => second_number,
//         Err(e) => return Err(e),
//     };

//     Ok(first_number * second_number)
// }

// fn print(result: Result<i32, ParseIntError>) {
//     match result {
//         Ok(n) => println!("n is {}", n),
//         Err(e) => println!("Error: {}", e),
//     }
// }

// fn main() {
//     print(multiply("10", "2")); // n is 20
//     print(multiply("t", "2")); // Error: invalid digit found in string
// }

// // ? 运算符
// use std::num::ParseIntError;

// fn multiply(first_number_str: &str, second_number_str: &str) -> Result<i32, ParseIntError> {
//     let first_number = first_number_str.parse::<i32>()?;
//     let second_number = second_number_str.parse::<i32>()?;

//     Ok(first_number * second_number)
// }

// fn print(result: Result<i32, ParseIntError>) {
//     match result {
//         Ok(n) => println!("n is {}", n),
//         Err(e) => println!("Error: {}", e),
//     }
// }

// fn main() {
//     print(multiply("10", "2")); // n is 20
//     print(multiply("t", "2")); // Error: invalid digit found in string
// }

// // try! 宏
// use std::num::ParseIntError;

// fn multiply(first_number_str: &str, second_number_str: &str) -> Result<i32, ParseIntError> {
//     let first_number = r#try!(first_number_str.parse::<i32>());
//     let second_number = r#try!(second_number_str.parse::<i32>());

//     Ok(first_number * second_number)
// }

// fn print(result: Result<i32, ParseIntError>) {
//     match result {
//         Ok(n)  => println!("n is {}", n),
//         Err(e) => println!("Error: {}", e),
//     }
// }

// fn main() {
//     print(multiply("10", "2")); // n is 20
//     print(multiply("t", "2")); // Error: invalid digit found in string
// }

// // 处理多种错误类型
// fn double_first(vec: Vec<&str>) -> i32 {
//     let first = vec.first().unwrap(); // 生成错误 1
//     2 * first.parse::<i32>().unwrap() // 生成错误 2
// }

// fn main() {
//     let numbers = vec!["42", "93", "18"];
//     let empty = vec![];
//     let strings = vec!["tofu", "93", "18"];

//     println!("The first doubled is {}", double_first(numbers)); // The first doubled is 84

//     println!("The first doubled is {}", double_first(empty)); // called `Option::unwrap()` on a `None` value
//     // 错误1：输入 vector 为空

//     println!("The first doubled is {}", double_first(strings)); // called `Result::unwrap()` on an `Err` value: ParseIntError { kind: InvalidDigit }
//     // 错误2：此元素不能解析成数字
// }

// use std::num::ParseIntError;

// fn double_first(vec: Vec<&str>) -> Option<Result<i32, ParseIntError>> {
//     vec.first().map(|first| first.parse::<i32>().map(|n| 2 * n))
// }

// fn main() {
//     let numbers = vec!["42", "93", "18"];
//     let empty = vec![];
//     let strings = vec!["tofu", "93", "18"];

//     println!("The first doubled is {:?}", double_first(numbers)); // The first doubled is Some(Ok(84))
//     println!("The first doubled is {:?}", double_first(empty)); // The first doubled is None
//     println!("The first doubled is {:?}", double_first(strings)); // The first doubled is Some(Err(ParseIntError { kind: InvalidDigit }))
// }

// use std::num::ParseIntError;

// fn double_first(vec: Vec<&str>) -> Result<Option<i32>, ParseIntError> {
//     let opt = vec.first().map(|first| first.parse::<i32>().map(|n| 2 * n));

//     opt.map_or(Ok(None), |r| r.map(Some))
// }

// fn main() {
//     let numbers = vec!["42", "93", "18"];
//     let empty = vec![];
//     let strings = vec!["tofu", "93", "18"];

//     println!("The first doubled is {:?}", double_first(numbers)); // The first doubled is Ok(Some(84))
//     println!("The first doubled is {:?}", double_first(empty)); // The first doubled is Ok(None)
//     println!("The first doubled is {:?}", double_first(strings)); // The first doubled is Err(ParseIntError { kind: InvalidDigit })
// }

// // 定义一个错误类型
// use std::error;
// use std::fmt;

// type Result<T> = std::result::Result<T, DoubleError>;

// #[derive(Debug, Clone)]
// // 定义我们的错误类型，这种类型可以根据错误处理的实际情况定制。
// // 可以完全自定义错误类型，也可以在类型中完全采用底层的错误实现，也可以介于二者之间。
// struct DoubleError;

// // 没有储存关于错误的任何额外信息，也就是说，如果不修改我们的错误类型定义的话，就无法指明是哪个字符串解析失败了。
// impl fmt::Display for DoubleError {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "invalid first item to double")
//     }
// }

// // 为 `DoubleError` 实现 `Error` trait，这样其他错误可以包裹这个错误类型。
// impl error::Error for DoubleError {
//     fn source(&self) -> Option<&(dyn error::Error + 'static)> {
//         None // 泛型错误，没有记录其内部原因。
//     }
// }

// fn double_first(vec: Vec<&str>) -> Result<i32> {
//     vec.first()
//         .ok_or(DoubleError) // 把错误换成我们的新类型。
//         .and_then(|s| {
//             s.parse::<i32>()
//                 .map_err(|_| DoubleError) // 这里也换成新类型。
//                 .map(|i| 2 * i)
//         })
// }

// fn print(result: Result<i32>) {
//     match result {
//         Ok(n) => println!("The first doubled is {}", n),
//         Err(e) => println!("Error: {}", e),
//     }
// }

// fn main() {
//     let numbers = vec!["42", "93", "18"];
//     let empty = vec![];
//     let strings = vec!["tofu", "93", "18"];

//     print(double_first(numbers)); // The first doubled is 84
//     print(double_first(empty)); // Error: invalid first item to double
//     print(double_first(strings)); // Error: invalid first item to double
// }

// // 把错误 “装箱”
// use std::error;
// use std::fmt;

// // 为 `Box<error::Error>` 取别名。
// type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

// #[derive(Debug, Clone)]
// struct EmptyVec;

// impl fmt::Display for EmptyVec {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "invalid first item to double")
//     }
// }

// impl error::Error for EmptyVec {
//     fn description(&self) -> &str {
//         "invalid first item to double"
//     }

//     fn cause(&self) -> Option<&dyn error::Error> {
//         // 泛型错误。没有记录其内部原因。
//         None
//     }
// }

// fn double_first(vec: Vec<&str>) -> Result<i32> {
//     vec.first()
//         .ok_or_else(|| EmptyVec.into()) // 装箱
//         .and_then(|s| {
//             s.parse::<i32>()
//                 .map_err(|e| e.into()) // 装箱
//                 .map(|i| 2 * i)
//         })
// }

// fn print(result: Result<i32>) {
//     match result {
//         Ok(n) => println!("The first doubled is {}", n),
//         Err(e) => println!("Error: {}", e),
//     }
// }

// fn main() {
//     let numbers = vec!["42", "93", "18"];
//     let empty = vec![];
//     let strings = vec!["tofu", "93", "18"];

//     print(double_first(numbers));
//     print(double_first(empty));
//     print(double_first(strings));
// }

// // ? 的用法
// use std::error;
// use std::fmt;

// // 为 `Box<error::Error>` 取别名。
// type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

// #[derive(Debug)]
// struct EmptyVec;

// impl fmt::Display for EmptyVec {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "invalid first item to double")
//     }
// }

// impl error::Error for EmptyVec {}

// // 使用 `?` 立即得到内部值。
// fn double_first(vec: Vec<&str>) -> Result<i32> {
//     let first = vec.first().ok_or(EmptyVec)?;
//     let parsed = first.parse::<i32>()?;
//     Ok(2 * parsed)
// }

// fn print(result: Result<i32>) {
//     match result {
//         Ok(n) => println!("The first doubled is {}", n),
//         Err(e) => println!("Error: {}", e),
//     }
// }

// fn main() {
//     let numbers = vec!["42", "93", "18"];
//     let empty = vec![];
//     let strings = vec!["tofu", "93", "18"];

//     print(double_first(numbers));
//     print(double_first(empty));
//     print(double_first(strings));
// }

// // 包裹错误
// use std::error;
// use std::fmt;
// use std::num::ParseIntError;

// type Result<T> = std::result::Result<T, DoubleError>;

// #[derive(Debug)]
// enum DoubleError {
//     EmptyVec,
//     // 在这个错误类型中，我们采用 `parse` 的错误类型中 `Err` 部分的实现。
//     // 若想提供更多信息，则该类型中还需要加入更多数据。
//     Parse(ParseIntError),
// }

// impl fmt::Display for DoubleError {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match *self {
//             DoubleError::EmptyVec => write!(f, "please use a vector with at least one element"),
//             // 这是一个封装（wrapper），它采用内部各类型对 `fmt` 的实现。
//             DoubleError::Parse(ref e) => e.fmt(f),
//         }
//     }
// }

// impl error::Error for DoubleError {
//     fn source(&self) -> Option<&(dyn error::Error + 'static)> {
//         match *self {
//             DoubleError::EmptyVec => None,
//             // 原因采取内部对错误类型的实现。它隐式地转换成了 trait 对象 `&error:Error`。
//             // 这可以工作，因为内部的类型已经实现了 `Error` trait。
//             DoubleError::Parse(ref e) => Some(e),
//         }
//     }
// }

// // 实现从 `ParseIntError` 到 `DoubleError` 的转换。
// // 在使用 `?` 时，或者一个 `ParseIntError` 需要转换成 `DoubleError` 时，它会被自动调用。
// impl From<ParseIntError> for DoubleError {
//     fn from(err: ParseIntError) -> DoubleError {
//         DoubleError::Parse(err)
//     }
// }

// fn double_first(vec: Vec<&str>) -> Result<i32> {
//     let first = vec.first().ok_or(DoubleError::EmptyVec)?;
//     let parsed = first.parse::<i32>()?;

//     Ok(2 * parsed)
// }

// fn print(result: Result<i32>) {
//     match result {
//         Ok(n) => println!("The first doubled is {}", n),
//         Err(e) => println!("Error: {}", e),
//     }
// }

// fn main() {
//     let numbers = vec!["42", "93", "18"];
//     let empty = vec![];
//     let strings = vec!["tofu", "93", "18"];

//     print(double_first(numbers)); // The first doubled is 84
//     print(double_first(empty)); // Error: please use a vector with at least one element
//     print(double_first(strings)); // Error: invalid digit found in string
// }

// // 遍历 Result
// fn main() {
//     let strings = vec!["tofu", "93", "18"];
//     let numbers: Vec<_> = strings
//         .into_iter()
//         .map(|s| s.parse::<i32>())
//         .collect();
//     println!("Results: {:?}", numbers);
//     // Results: [Err(ParseIntError { kind: InvalidDigit }), Ok(93), Ok(18)]
// }

// // filter_map 会调用一个函数，过滤掉为 None 的所有结果
// fn main() {
//     let strings = vec!["tofu", "93", "18"];
//     let numbers: Vec<_> = strings
//         .into_iter()
//         .filter_map(|s| s.parse::<i32>().ok())
//         .collect();
//     println!("Results: {:?}", numbers); // Results: [93, 18]
// }

// // Result 实现了 FromIter，因此结果的向量（Vec<Result<T, E>>）可以被转换成结果包裹着向量（Result<Vec<T>, E>）。一旦找到一个 Result::Err ，遍历就被终止。
// fn main() {
//     let strings = vec!["tofu", "93", "18"];
//     let numbers: Result<Vec<_>, _> = strings
//         .into_iter()
//         .map(|s| s.parse::<i32>())
//         .collect();
//     println!("Results: {:?}", numbers); // Results: Err(ParseIntError { kind: InvalidDigit })
// }

// // 使用 Partition() 收集所有合法的值与错误
// fn main() {
//     let strings = vec!["tofu", "93", "18"];
//     let (numbers, errors): (Vec<_>, Vec<_>) = strings
//         .into_iter()
//         .map(|s| s.parse::<i32>())
//         .partition(Result::is_ok);
//     println!("Numbers: {:?}", numbers); // Numbers: [Ok(93), Ok(18)]
//     println!("Errors: {:?}", errors); // Errors: [Err(ParseIntError { kind: InvalidDigit })]
// }

fn main() {
    let strings = vec!["tofu", "93", "18"];
    let (numbers, errors): (Vec<_>, Vec<_>) = strings
        .into_iter()
        .map(|s| s.parse::<i32>())
        .partition(Result::is_ok);
    let numbers: Vec<_> = numbers.into_iter().map(Result::unwrap).collect();
    let errors: Vec<_> = errors.into_iter().map(Result::unwrap_err).collect();
    println!("Numbers: {:?}", numbers); // Numbers: [93, 18]
    println!("Errors: {:?}", errors); // Errors: [ParseIntError { kind: InvalidDigit }]
}

