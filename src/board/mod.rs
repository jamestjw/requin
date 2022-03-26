mod board;
pub mod movements;

pub use board::*;
pub use movements::{Direction, ADJACENCY_TABLE, KNIGHT_MOVES_TABLE};
