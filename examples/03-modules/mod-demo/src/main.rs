use mod_demo::eat::eat_bamboo;
use mod_demo::sleep;

#[derive(Debug)]
fn main() {
    println!("{:?}", mod_demo::get_up()); // "get up!"
    println!("{:?}", eat_bamboo()); // "eat bamboo~"
    println!("{:?}", sleep::sleep_night()); // "good night~"
}
