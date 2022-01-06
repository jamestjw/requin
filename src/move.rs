use crate::board::{Coordinate, Piece};

#[derive(PartialEq, Debug)]
pub struct Move {
    src: Coordinate,
    dest: Coordinate,
    piece: Piece,
    is_capture: bool,
}

impl Move {
    pub fn new(src: Coordinate, dest: Coordinate, piece: Piece, is_capture: bool) -> Self {
        Move {
            src,
            dest,
            piece,
            is_capture,
        }
    }
}
