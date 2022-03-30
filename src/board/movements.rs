use std::convert::TryFrom;
use std::slice::Iter;

use crate::bitboard::{get_piece_attacks_bb, lsb};
use crate::board::{Board, Color, Coordinate, PieceType};

lazy_static! {
    pub static ref ADJACENCY_TABLE: AdjacencyTable = AdjacencyTable::new();
}

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum Direction {
    N = 0,
    NE = 1,
    E = 2,
    SE = 3,
    S = 4,
    SW = 5,
    W = 6,
    NW = 7,
}

impl Direction {
    pub fn iterator() -> Iter<'static, Direction> {
        static ALL_DIRECTIONS: [Direction; 8] = [
            Direction::N,
            Direction::S,
            Direction::E,
            Direction::W,
            Direction::NE,
            Direction::NW,
            Direction::SE,
            Direction::SW,
        ];
        ALL_DIRECTIONS.iter()
    }

    pub fn horizontal_vertical_iterator() -> Iter<'static, Direction> {
        static HV_DIRECTIONS: [Direction; 4] =
            [Direction::N, Direction::S, Direction::E, Direction::W];
        HV_DIRECTIONS.iter()
    }

    pub fn diagonal_iterator() -> Iter<'static, Direction> {
        static DIAG_DIRECTIONS: [Direction; 4] =
            [Direction::NE, Direction::NW, Direction::SE, Direction::SW];
        DIAG_DIRECTIONS.iter()
    }
}

// A table that knows which squares are adjacent to any
// square on the board in all possible directions
pub struct AdjacencyTable {
    table: [[Option<Coordinate>; 8]; 64],
}

impl AdjacencyTable {
    pub fn new() -> Self {
        let mut t = AdjacencyTable {
            table: [[None; 8]; 64],
        };

        for i in 0..64 {
            // This is safe as these are all legal coordinates
            let coord = Coordinate::try_from(i).unwrap();

            // All squares below the 8th rank should have an
            // adjacent square north of it
            if !coord.is_in_rank(8) {
                t.set(coord, coord.vertical_offset(1, true), Direction::N);
            }

            // All squares above the 1st rank should have an
            // adjacent square south of it
            if !coord.is_in_rank(1) {
                t.set(coord, coord.vertical_offset(1, false), Direction::S);
            }

            // All squares not in the A-file should have an adjacent square
            // on its left
            // TODO: Figure out a better way to represent files
            if !(i % 8 == 0) {
                t.set(coord, coord.horizontal_offset(1, true), Direction::W);
            }

            // All squares not in the H-file should have an adjacent square
            // on its left
            // TODO: Figure out a better way to represent files
            if !((i + 1) % 8 == 0) {
                t.set(coord, coord.horizontal_offset(1, false), Direction::E);
            }

            // All squares not in the A-file and not on the 8th rank should have
            // an adjacent square on its top left
            if !((i % 8 == 0) || coord.is_in_rank(8)) {
                t.set(coord, coord.diagonal_offset(true, true), Direction::NW);
            }

            // All squares not in the A-file and not on the 1st rank should have
            // an adjacent square on its bottom left
            if !((i % 8 == 0) || coord.is_in_rank(1)) {
                t.set(coord, coord.diagonal_offset(false, true), Direction::SW);
            }

            // All squares not in the H-file and not on the 8th rank should have
            // an adjacent square on its top right
            if !(((i + 1) % 8 == 0) || coord.is_in_rank(8)) {
                t.set(coord, coord.diagonal_offset(true, false), Direction::NE);
            }

            // All squares not in the H-file and not on the 1st rank should have
            // an adjacent square on its bottom right
            if !(((i + 1) % 8 == 0) || coord.is_in_rank(1)) {
                t.set(coord, coord.diagonal_offset(false, false), Direction::SE);
            }
        }
        t
    }

    fn set(&mut self, src: Coordinate, dest: Coordinate, dir: Direction) {
        self.table[src as usize][dir as usize] = Some(dest);
    }

    pub fn get(&self, src: Coordinate, dir: Direction) -> Option<Coordinate> {
        self.table[src as usize][dir as usize]
    }
}

pub fn is_square_controlled_by_player(board: &Board, color: Color, square: Coordinate) -> bool {
    // Explore all directions of attack

    is_square_controlled_by_pawn(board, color, square)
        || is_square_controlled_by_knight(board, color, square)
        || is_square_controlled_by_bishop_style(board, color, square)
        || is_square_controlled_by_rook_style(board, color, square)
        || is_square_controlled_by_king(board, color, square)
}

// Color is the color of the player doing the controlling
fn is_square_controlled_by_pawn(board: &Board, color: Color, square: Coordinate) -> bool {
    square_controlled_by_pawn_from(board, color, square).is_some()
}

pub fn square_controlled_by_pawn_from(
    board: &Board,
    color: Color,
    square: Coordinate,
) -> Option<Coordinate> {
    let directions = match color {
        Color::White => [Direction::SW, Direction::SE],
        Color::Black => [Direction::NW, Direction::NE],
    };

    for direction in &directions {
        if let Some(dest) = ADJACENCY_TABLE.get(square, *direction) {
            if let Some(p) = board.get_from_coordinate(dest) {
                if p.color == color && p.piece_type == PieceType::Pawn {
                    return Some(dest);
                }
            }
        }
    }

    None
}

pub fn is_square_controlled_by_knight(board: &Board, color: Color, square: Coordinate) -> bool {
    square_controlled_by_knight_from(board, color, square).is_some()
}

pub fn square_controlled_by_knight_from(
    board: &Board,
    color: Color,
    square: Coordinate,
) -> Option<Coordinate> {
    let attacking_knight_bb = get_piece_attacks_bb(PieceType::Knight, square)
        & board.get_color_bb(color)
        & board.get_piece_type_bb(PieceType::Knight);

    if attacking_knight_bb != 0 {
        Some(Coordinate::from_bb(lsb(attacking_knight_bb)))
    } else {
        None
    }
}

fn is_square_controlled_by_bishop_style(board: &Board, color: Color, square: Coordinate) -> bool {
    square_controlled_by_bishop_style_from(board, color, square).is_some()
}

pub fn square_controlled_by_bishop_style_from(
    board: &Board,
    color: Color,
    square: Coordinate,
) -> Option<(PieceType, Coordinate)> {
    for dir in Direction::diagonal_iterator() {
        let mut curr_square = square;

        loop {
            if let Some(dest_square) = ADJACENCY_TABLE.get(curr_square, *dir) {
                if let Some(p) = board.get_from_coordinate(dest_square) {
                    if p.color == color
                        && (p.piece_type == PieceType::Bishop || p.piece_type == PieceType::Queen)
                    {
                        return Some((p.piece_type, dest_square));
                    }
                    // Obstruction by non-bishop style piece or piece with the wrong color
                    break;
                } else {
                    curr_square = dest_square;
                }
            } else {
                break;
            }
        }
    }
    None
}

pub fn square_controlled_by_bishop_or_queen_from(
    board: &Board,
    color: Color,
    square: Coordinate,
    piece_type: PieceType,
) -> Option<(PieceType, Coordinate)> {
    for dir in Direction::diagonal_iterator() {
        let mut curr_square = square;

        loop {
            if let Some(dest_square) = ADJACENCY_TABLE.get(curr_square, *dir) {
                if let Some(p) = board.get_from_coordinate(dest_square) {
                    if p.color == color && (p.piece_type == piece_type) {
                        return Some((p.piece_type, dest_square));
                    }
                    // Obstruction by non-bishop style piece or piece with the wrong color
                    break;
                } else {
                    curr_square = dest_square;
                }
            } else {
                break;
            }
        }
    }
    None
}

fn is_square_controlled_by_rook_style(board: &Board, color: Color, square: Coordinate) -> bool {
    square_controlled_by_rook_style_from(board, color, square).is_some()
}

pub fn square_controlled_by_rook_style_from(
    board: &Board,
    color: Color,
    square: Coordinate,
) -> Option<(PieceType, Coordinate)> {
    for dir in Direction::horizontal_vertical_iterator() {
        let mut curr_square = square;

        loop {
            if let Some(dest_square) = ADJACENCY_TABLE.get(curr_square, *dir) {
                if let Some(p) = board.get_from_coordinate(dest_square) {
                    if p.color == color
                        && (p.piece_type == PieceType::Rook || p.piece_type == PieceType::Queen)
                    {
                        return Some((p.piece_type, dest_square));
                    }
                    // Obstruction by non-rook style piece or piece with the wrong color
                    break;
                } else {
                    curr_square = dest_square;
                }
            } else {
                break;
            }
        }
    }
    None
}

pub fn square_controlled_by_rook_or_queen_from(
    board: &Board,
    color: Color,
    square: Coordinate,
    piece_type: PieceType,
) -> Option<(PieceType, Coordinate)> {
    for dir in Direction::horizontal_vertical_iterator() {
        let mut curr_square = square;

        loop {
            if let Some(dest_square) = ADJACENCY_TABLE.get(curr_square, *dir) {
                if let Some(p) = board.get_from_coordinate(dest_square) {
                    if p.color == color && (p.piece_type == piece_type) {
                        return Some((p.piece_type, dest_square));
                    }
                    // Obstruction by non-rook style piece or piece with the wrong color
                    break;
                } else {
                    curr_square = dest_square;
                }
            } else {
                break;
            }
        }
    }
    None
}

fn is_square_controlled_by_king(board: &Board, color: Color, square: Coordinate) -> bool {
    square_controlled_by_king_from(board, color, square).is_some()
}

pub fn square_controlled_by_king_from(
    board: &Board,
    color: Color,
    square: Coordinate,
) -> Option<Coordinate> {
    for dir in Direction::iterator() {
        if let Some(dest_square) = ADJACENCY_TABLE.get(square, *dir) {
            if let Some(p) = board.get_from_coordinate(dest_square) {
                if p.color == color && p.piece_type == PieceType::King {
                    return Some(dest_square);
                }
            }
        }
    }
    None
}

pub fn are_squares_controlled_by_player(
    board: &Board,
    color: Color,
    squares: &[Coordinate],
) -> bool {
    if squares.len() == 0 {
        return false;
    }

    squares
        .iter()
        .map(|square| is_square_controlled_by_player(board, color, *square))
        .reduce(|a, b| a || b)
        .unwrap()
}

#[cfg(test)]
mod adjacency_table_tests {
    use super::*;

    #[test]
    fn center_square() {
        let t = AdjacencyTable::new();
        let src = Coordinate::E4;
        assert_eq!(t.get(src, Direction::N), Some(Coordinate::E5));
        assert_eq!(t.get(src, Direction::S), Some(Coordinate::E3));
        assert_eq!(t.get(src, Direction::E), Some(Coordinate::F4));
        assert_eq!(t.get(src, Direction::W), Some(Coordinate::D4));
        assert_eq!(t.get(src, Direction::NW), Some(Coordinate::D5));
        assert_eq!(t.get(src, Direction::NE), Some(Coordinate::F5));
        assert_eq!(t.get(src, Direction::SW), Some(Coordinate::D3));
        assert_eq!(t.get(src, Direction::SE), Some(Coordinate::F3));
    }

    #[test]
    fn a_file_center_square() {
        let t = AdjacencyTable::new();
        let src = Coordinate::A4;
        assert_eq!(t.get(src, Direction::N), Some(Coordinate::A5));
        assert_eq!(t.get(src, Direction::S), Some(Coordinate::A3));
        assert_eq!(t.get(src, Direction::E), Some(Coordinate::B4));
        assert_eq!(t.get(src, Direction::W), None);
        assert_eq!(t.get(src, Direction::NW), None);
        assert_eq!(t.get(src, Direction::NE), Some(Coordinate::B5));
        assert_eq!(t.get(src, Direction::SW), None);
        assert_eq!(t.get(src, Direction::SE), Some(Coordinate::B3));
    }

    #[test]
    fn h_file_center_square() {
        let t = AdjacencyTable::new();
        let src = Coordinate::H5;
        assert_eq!(t.get(src, Direction::N), Some(Coordinate::H6));
        assert_eq!(t.get(src, Direction::S), Some(Coordinate::H4));
        assert_eq!(t.get(src, Direction::W), Some(Coordinate::G5));
        assert_eq!(t.get(src, Direction::E), None);
        assert_eq!(t.get(src, Direction::NW), Some(Coordinate::G6));
        assert_eq!(t.get(src, Direction::NE), None);
        assert_eq!(t.get(src, Direction::SW), Some(Coordinate::G4));
        assert_eq!(t.get(src, Direction::SE), None);
    }

    #[test]
    fn first_rank_center_square() {
        let t = AdjacencyTable::new();
        let src = Coordinate::D1;
        assert_eq!(t.get(src, Direction::N), Some(Coordinate::D2));
        assert_eq!(t.get(src, Direction::S), None);
        assert_eq!(t.get(src, Direction::E), Some(Coordinate::E1));
        assert_eq!(t.get(src, Direction::W), Some(Coordinate::C1));
        assert_eq!(t.get(src, Direction::NW), Some(Coordinate::C2));
        assert_eq!(t.get(src, Direction::NE), Some(Coordinate::E2));
        assert_eq!(t.get(src, Direction::SW), None);
        assert_eq!(t.get(src, Direction::SE), None);
    }

    #[test]
    fn eighth_rank_center_square() {
        let t = AdjacencyTable::new();
        let src = Coordinate::C8;
        assert_eq!(t.get(src, Direction::N), None);
        assert_eq!(t.get(src, Direction::S), Some(Coordinate::C7));
        assert_eq!(t.get(src, Direction::E), Some(Coordinate::D8));
        assert_eq!(t.get(src, Direction::W), Some(Coordinate::B8));
        assert_eq!(t.get(src, Direction::NW), None);
        assert_eq!(t.get(src, Direction::NE), None);
        assert_eq!(t.get(src, Direction::SW), Some(Coordinate::B7));
        assert_eq!(t.get(src, Direction::SE), Some(Coordinate::D7));
    }
}

#[cfg(test)]
mod controlled_squares {
    use super::*;
    use crate::board::Piece;

    #[test]
    fn squares_controlled_by_white_pawn() {
        let mut board = Board::new_empty();
        let pawn = Piece {
            color: Color::White,
            piece_type: PieceType::Pawn,
        };
        board.place_piece(Coordinate::E4, pawn);

        assert!(is_square_controlled_by_pawn(
            &board,
            Color::White,
            Coordinate::D5
        ));
        assert!(is_square_controlled_by_pawn(
            &board,
            Color::White,
            Coordinate::F5
        ));

        assert!(!is_square_controlled_by_pawn(
            &board,
            Color::White,
            Coordinate::E5
        ));
        assert!(!is_square_controlled_by_pawn(
            &board,
            Color::White,
            Coordinate::D4
        ));
        assert!(!is_square_controlled_by_pawn(
            &board,
            Color::White,
            Coordinate::F4
        ));
    }

    #[test]
    fn squares_controlled_by_white_king() {
        let mut board = Board::new_empty();
        let pawn = Piece {
            color: Color::White,
            piece_type: PieceType::King,
        };
        board.place_piece(Coordinate::E4, pawn);

        assert!(is_square_controlled_by_king(
            &board,
            Color::White,
            Coordinate::D5
        ));
        assert!(is_square_controlled_by_king(
            &board,
            Color::White,
            Coordinate::E5
        ));
        assert!(is_square_controlled_by_king(
            &board,
            Color::White,
            Coordinate::F5
        ));

        assert!(is_square_controlled_by_king(
            &board,
            Color::White,
            Coordinate::D3
        ));
        assert!(is_square_controlled_by_king(
            &board,
            Color::White,
            Coordinate::E3
        ));
        assert!(is_square_controlled_by_king(
            &board,
            Color::White,
            Coordinate::F3
        ));

        assert!(is_square_controlled_by_king(
            &board,
            Color::White,
            Coordinate::D4
        ));
        assert!(is_square_controlled_by_king(
            &board,
            Color::White,
            Coordinate::F4
        ));
    }

    #[test]
    fn squares_controlled_by_bishop() {
        let mut board = Board::new_empty();
        let bishop = Piece {
            color: Color::White,
            piece_type: PieceType::Bishop,
        };
        board.place_piece(Coordinate::E4, bishop);

        assert!(is_square_controlled_by_bishop_style(
            &board,
            Color::White,
            Coordinate::D5
        ));
        assert!(is_square_controlled_by_bishop_style(
            &board,
            Color::White,
            Coordinate::C6
        ));
        assert!(is_square_controlled_by_bishop_style(
            &board,
            Color::White,
            Coordinate::D3
        ));
        assert!(is_square_controlled_by_bishop_style(
            &board,
            Color::White,
            Coordinate::C2
        ));
        assert!(is_square_controlled_by_bishop_style(
            &board,
            Color::White,
            Coordinate::F5
        ));
        assert!(is_square_controlled_by_bishop_style(
            &board,
            Color::White,
            Coordinate::G6
        ));
        assert!(is_square_controlled_by_bishop_style(
            &board,
            Color::White,
            Coordinate::F3
        ));
        assert!(is_square_controlled_by_bishop_style(
            &board,
            Color::White,
            Coordinate::B1
        ));

        let friendly_obstruction = Piece {
            color: Color::White,
            piece_type: PieceType::Pawn,
        };
        board.place_piece(Coordinate::F5, friendly_obstruction);

        assert!(!is_square_controlled_by_bishop_style(
            &board,
            Color::White,
            Coordinate::G6
        ));

        let enemy_obstruction = Piece {
            color: Color::Black,
            piece_type: PieceType::Pawn,
        };
        board.place_piece(Coordinate::C2, enemy_obstruction);

        assert!(!is_square_controlled_by_bishop_style(
            &board,
            Color::White,
            Coordinate::B1
        ));
    }

    #[test]
    fn squares_controlled_by_queen() {
        let mut board = Board::new_empty();
        let queen = Piece {
            color: Color::White,
            piece_type: PieceType::Queen,
        };
        board.place_piece(Coordinate::E4, queen);

        assert!(is_square_controlled_by_bishop_style(
            &board,
            Color::White,
            Coordinate::D5
        ));
        assert!(is_square_controlled_by_bishop_style(
            &board,
            Color::White,
            Coordinate::C6
        ));
        assert!(is_square_controlled_by_bishop_style(
            &board,
            Color::White,
            Coordinate::D3
        ));
        assert!(is_square_controlled_by_bishop_style(
            &board,
            Color::White,
            Coordinate::C2
        ));
        assert!(is_square_controlled_by_bishop_style(
            &board,
            Color::White,
            Coordinate::F5
        ));
        assert!(is_square_controlled_by_bishop_style(
            &board,
            Color::White,
            Coordinate::G6
        ));
        assert!(is_square_controlled_by_bishop_style(
            &board,
            Color::White,
            Coordinate::F3
        ));
        assert!(is_square_controlled_by_bishop_style(
            &board,
            Color::White,
            Coordinate::B1
        ));

        let friendly_obstruction = Piece {
            color: Color::White,
            piece_type: PieceType::Pawn,
        };
        board.place_piece(Coordinate::F5, friendly_obstruction);

        assert!(!is_square_controlled_by_bishop_style(
            &board,
            Color::White,
            Coordinate::G6
        ));

        let enemy_obstruction = Piece {
            color: Color::Black,
            piece_type: PieceType::Pawn,
        };
        board.place_piece(Coordinate::C2, enemy_obstruction);

        assert!(!is_square_controlled_by_bishop_style(
            &board,
            Color::White,
            Coordinate::B1
        ));
    }
}
