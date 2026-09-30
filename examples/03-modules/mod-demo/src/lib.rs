mod zoo;

pub use crate::zoo::eat;
pub use crate::zoo::sleep;

pub fn get_up() -> String {
  String::from("get up!")
}