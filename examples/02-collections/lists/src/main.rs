use lists::List;

fn main() {
  let mut list = List::<i32> { head: None };
  list.push(1);
  list.push(2);
  list.push(3);

  // println!("{:?}", list);
}