pub mod board;
pub mod game;
pub mod generator;
pub mod r#move;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate bitfield;

pub fn clear_screen() {
    print!("{}[2J", 27 as char);
}
