use crate::r#move::{CastlingSide, Move};
use colored::Colorize;
use num_enum::TryFromPrimitive;
use std::convert::TryFrom;
use std::fmt;
use std::slice::Iter;

pub static FILE_LIST: [&str; 8] = ["a", "b", "c", "d", "e", "f", "g", "h"];

#[repr(u8)]
#[allow(dead_code)]
#[derive(TryFromPrimitive, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Coordinate {
    A1 = 0,
    B1,
    C1,
    D1,
    E1,
    F1,
    G1,
    H1,
    A2,
    B2,
    C2,
    D2,
    E2,
    F2,
    G2,
    H2,
    A3,
    B3,
    C3,
    D3,
    E3,
    F3,
    G3,
    H3,
    A4,
    B4,
    C4,
    D4,
    E4,
    F4,
    G4,
    H4,
    A5,
    B5,
    C5,
    D5,
    E5,
    F5,
    G5,
    H5,
    A6,
    B6,
    C6,
    D6,
    E6,
    F6,
    G6,
    H6,
    A7,
    B7,
    C7,
    D7,
    E7,
    F7,
    G7,
    H7,
    A8,
    B8,
    C8,
    D8,
    E8,
    F8,
    G8,
    H8,
}

impl Coordinate {
    // Rank is between 1-8, file is between 1-8
    pub fn new_from_rank_file(rank: u8, file: u8) -> Coordinate {
        Coordinate::try_from((rank - 1) * 8 + (file - 1)).unwrap()
    }

    // Returns the coordinate of the square some vertical offset
    // from this one. The offset can either be a front or
    // backward offset.
    pub fn vertical_offset(&self, offset: u8, front: bool) -> Coordinate {
        let offset_val = if front {
            *self as u8 + 8 * offset
        } else {
            *self as u8 - 8 * offset
        };
        return Coordinate::try_from(offset_val).expect("Invalid coordinate");
    }

    // Returns the coordinate of the square some horizontal offset
    // from this one. The offset can either be a left or
    // right offset.
    pub fn horizontal_offset(&self, offset: u8, left: bool) -> Coordinate {
        let offset_val = if left {
            *self as u8 - offset
        } else {
            *self as u8 + offset
        };
        return Coordinate::try_from(offset_val).expect("Invalid coordinate");
    }

    // Returns the coordinate of the square that is at a diagonal
    // offset from this one. The combination of the front and left
    // booleans determine which one of four directions the offset
    // should be.
    pub fn diagonal_offset(&self, front: bool, left: bool) -> Coordinate {
        let mut offset_val = *self as u8;

        if !left {
            offset_val += 2
        }

        if front {
            offset_val += 7
        } else {
            offset_val -= 9
        }

        return Coordinate::try_from(offset_val).expect("Invalid coordinate");
    }

    // Returns value 1..=8 corresponding to the rank of the square
    pub fn get_rank(&self) -> u8 {
        (*self as u8 / 8) + 1
    }

    // Returns value 1..=8 corresponding to the file of the square
    pub fn get_file(&self) -> u8 {
        (*self as u8 % 8) + 1
    }

    // Returns true if this coordinate is in a certain rank
    // Rank should be between 1 to 8
    pub fn is_in_rank(&self, rank: u8) -> bool {
        if rank < 1 || rank > 8 {
            panic!("Invalid parameter passed to `is_in_rank`.");
        }

        return self.get_rank() == rank;
    }

    // Returns true if this coordinate is in a certain file
    // File should be between 1 to 8
    pub fn is_in_file(&self, file: u8) -> bool {
        if file < 1 || file > 8 {
            panic!("Invalid parameter passed to `is_in_file`.");
        }
        return self.get_file() == file;
    }

    pub fn side_squares(&self) -> Vec<Coordinate> {
        let mut res = vec![];
        // i8 because the idx could be negative after applying the offset
        let coord_idx = *self as u8;
        let row_idx = coord_idx % 8;

        if row_idx != 0 {
            res.push(Coordinate::try_from(((coord_idx / 8) * 8) + row_idx - 1).unwrap());
        }

        if row_idx != 7 {
            res.push(Coordinate::try_from(((coord_idx / 8) * 8) + row_idx + 1).unwrap());
        }

        res
    }

    pub fn to_algebraic_notation(&self) -> String {
        format!("{:?}", *self).to_lowercase()
    }
}

bitfield! {
    #[derive(Clone, Copy)]
    struct CastlingRights(u8);
    get_white_kingside, set_white_kingside: 1;
    get_white_queenside, set_white_queenside: 2;
    get_black_kingside, set_black_kingside: 3;
    get_black_queenside, set_black_queenside: 4;
}

impl CastlingRights {
    pub fn new_with_all_disabled() -> Self {
        CastlingRights(0)
    }

    pub fn new_with_all_enabled() -> Self {
        CastlingRights(0b11111111)
    }
}

#[derive(Clone, Copy)]
pub struct Board {
    pieces: [Option<Piece>; 64],
    is_game_over: bool,
    player_turn: Color,
    castling_rights: CastlingRights,
    pub last_move: Option<Move>,
}

impl Board {
    pub fn new_empty() -> Board {
        Board {
            pieces: [None; 64],
            is_game_over: false,
            player_turn: Color::White,
            castling_rights: CastlingRights::new_with_all_disabled(),
            last_move: None,
        }
    }

    pub fn new_starting_pos() -> Board {
        let mut board = Board::new_empty();

        // At the starting position, both players have their castling rights
        board.castling_rights = CastlingRights::new_with_all_enabled();

        board.place_piece(
            Coordinate::A1,
            Piece {
                color: Color::White,
                piece_type: PieceType::Rook,
            },
        );
        board.place_piece(
            Coordinate::B1,
            Piece {
                color: Color::White,
                piece_type: PieceType::Knight,
            },
        );
        board.place_piece(
            Coordinate::C1,
            Piece {
                color: Color::White,
                piece_type: PieceType::Bishop,
            },
        );
        board.place_piece(
            Coordinate::D1,
            Piece {
                color: Color::White,
                piece_type: PieceType::Queen,
            },
        );
        board.place_piece(
            Coordinate::E1,
            Piece {
                color: Color::White,
                piece_type: PieceType::King,
            },
        );
        board.place_piece(
            Coordinate::F1,
            Piece {
                color: Color::White,
                piece_type: PieceType::Bishop,
            },
        );
        board.place_piece(
            Coordinate::G1,
            Piece {
                color: Color::White,
                piece_type: PieceType::Knight,
            },
        );
        board.place_piece(
            Coordinate::H1,
            Piece {
                color: Color::White,
                piece_type: PieceType::Rook,
            },
        );
        board.place_piece(
            Coordinate::A2,
            Piece {
                color: Color::White,
                piece_type: PieceType::Pawn,
            },
        );
        board.place_piece(
            Coordinate::B2,
            Piece {
                color: Color::White,
                piece_type: PieceType::Pawn,
            },
        );
        board.place_piece(
            Coordinate::C2,
            Piece {
                color: Color::White,
                piece_type: PieceType::Pawn,
            },
        );
        board.place_piece(
            Coordinate::D2,
            Piece {
                color: Color::White,
                piece_type: PieceType::Pawn,
            },
        );
        board.place_piece(
            Coordinate::E2,
            Piece {
                color: Color::White,
                piece_type: PieceType::Pawn,
            },
        );
        board.place_piece(
            Coordinate::F2,
            Piece {
                color: Color::White,
                piece_type: PieceType::Pawn,
            },
        );
        board.place_piece(
            Coordinate::G2,
            Piece {
                color: Color::White,
                piece_type: PieceType::Pawn,
            },
        );
        board.place_piece(
            Coordinate::H2,
            Piece {
                color: Color::White,
                piece_type: PieceType::Pawn,
            },
        );
        board.place_piece(
            Coordinate::A8,
            Piece {
                color: Color::Black,
                piece_type: PieceType::Rook,
            },
        );
        board.place_piece(
            Coordinate::B8,
            Piece {
                color: Color::Black,
                piece_type: PieceType::Knight,
            },
        );
        board.place_piece(
            Coordinate::C8,
            Piece {
                color: Color::Black,
                piece_type: PieceType::Bishop,
            },
        );
        board.place_piece(
            Coordinate::D8,
            Piece {
                color: Color::Black,
                piece_type: PieceType::Queen,
            },
        );
        board.place_piece(
            Coordinate::E8,
            Piece {
                color: Color::Black,
                piece_type: PieceType::King,
            },
        );
        board.place_piece(
            Coordinate::F8,
            Piece {
                color: Color::Black,
                piece_type: PieceType::Bishop,
            },
        );
        board.place_piece(
            Coordinate::G8,
            Piece {
                color: Color::Black,
                piece_type: PieceType::Knight,
            },
        );
        board.place_piece(
            Coordinate::H8,
            Piece {
                color: Color::Black,
                piece_type: PieceType::Rook,
            },
        );
        board.place_piece(
            Coordinate::A7,
            Piece {
                color: Color::Black,
                piece_type: PieceType::Pawn,
            },
        );
        board.place_piece(
            Coordinate::B7,
            Piece {
                color: Color::Black,
                piece_type: PieceType::Pawn,
            },
        );
        board.place_piece(
            Coordinate::C7,
            Piece {
                color: Color::Black,
                piece_type: PieceType::Pawn,
            },
        );
        board.place_piece(
            Coordinate::D7,
            Piece {
                color: Color::Black,
                piece_type: PieceType::Pawn,
            },
        );
        board.place_piece(
            Coordinate::E7,
            Piece {
                color: Color::Black,
                piece_type: PieceType::Pawn,
            },
        );
        board.place_piece(
            Coordinate::F7,
            Piece {
                color: Color::Black,
                piece_type: PieceType::Pawn,
            },
        );
        board.place_piece(
            Coordinate::G7,
            Piece {
                color: Color::Black,
                piece_type: PieceType::Pawn,
            },
        );
        board.place_piece(
            Coordinate::H7,
            Piece {
                color: Color::Black,
                piece_type: PieceType::Pawn,
            },
        );

        board
    }

    pub fn place_piece(&mut self, coord: Coordinate, piece: Piece) {
        self.pieces[coord as usize] = Some(piece);
    }

    pub fn print(&self) {
        for rank in (0..8).rev() {
            for file in 0..8 {
                // We add spaces before and after the piece
                match self.pieces[rank * 8 + file] {
                    Some(p) => print!(" {} ", p),
                    None => {
                        // Use 'x' to represent an empty square
                        print!(" x ");
                    }
                }
            }

            println!("");
        }
    }

    pub fn is_white_turn(&self) -> bool {
        return self.player_turn == Color::White;
    }

    pub fn get_from_coordinate(&self, coordinate: Coordinate) -> Option<Piece> {
        return self.pieces[coordinate as usize];
    }

    pub fn get_player_pieces(&self, color: Color) -> Vec<(Coordinate, &Piece)> {
        let mut res = vec![];

        for (coord, piece) in self.pieces.iter().enumerate() {
            match piece {
                Some(piece) => {
                    if color == piece.color {
                        res.push((Coordinate::try_from(coord as u8).unwrap(), piece));
                    }
                }
                None => {}
            }
        }

        return res;
    }

    pub fn get_player_color(&self) -> Color {
        self.player_turn
    }

    pub fn set_player_color(&mut self, color: Color) {
        self.player_turn = color;
    }

    pub fn get_opposing_player_color(&self) -> Color {
        self.player_turn.other_color()
    }

    pub fn is_square_occupied(&self, coord: Coordinate) -> bool {
        return self.pieces[coord as usize].is_some();
    }

    pub fn may_castle(&self, color: Color, kingside: bool) -> bool {
        match color {
            Color::White => {
                if kingside {
                    self.castling_rights.get_white_kingside()
                } else {
                    self.castling_rights.get_white_queenside()
                }
            }
            Color::Black => {
                if kingside {
                    self.castling_rights.get_black_kingside()
                } else {
                    self.castling_rights.get_black_queenside()
                }
            }
        }
    }

    pub fn enable_castling(&mut self, color: Color, kingside: bool) {
        match color {
            Color::White => {
                if kingside {
                    self.castling_rights.set_white_kingside(true)
                } else {
                    self.castling_rights.set_white_queenside(true)
                }
            }
            Color::Black => {
                if kingside {
                    self.castling_rights.set_black_kingside(true)
                } else {
                    self.castling_rights.set_black_queenside(true)
                }
            }
        }
    }

    pub fn disable_castling(&mut self, color: Color, kingside: bool) {
        match color {
            Color::White => {
                if kingside {
                    self.castling_rights.set_white_kingside(false)
                } else {
                    self.castling_rights.set_white_queenside(false)
                }
            }
            Color::Black => {
                if kingside {
                    self.castling_rights.set_black_kingside(false)
                } else {
                    self.castling_rights.set_black_queenside(false)
                }
            }
        }
    }

    // Applies a move to the board
    // This does not check that the move is legal
    pub fn apply_move(&mut self, m: &Move) {
        let player_color = self.get_from_coordinate(m.src).unwrap().color;
        // Handle a normal move
        if m.castling_side == CastlingSide::Unknown {
            let original_piece = self.pieces[m.src as usize].take();
            self.pieces[m.dest as usize] = original_piece;

            // Remove captured piece during en passant capture
            if m.is_capture && m.is_en_passant {
                let to_remove_square = m.dest.vertical_offset(1, !player_color.is_white());
                self.pieces[to_remove_square as usize] = None;
            }

            match m.promotes_to {
                Some(ppt) => {
                    let mut promoted_piece = original_piece.unwrap();
                    promoted_piece.piece_type = ppt;
                    self.pieces[m.dest as usize] = Some(promoted_piece);
                }
                None => {}
            }
        } else {
            // Handle castling
            let (king_src, king_dest, rook_src, rook_dest) = match player_color {
                Color::White => {
                    if m.castling_side == CastlingSide::Kingside {
                        (
                            Coordinate::E1,
                            Coordinate::G1,
                            Coordinate::H1,
                            Coordinate::F1,
                        )
                    } else {
                        (
                            Coordinate::E1,
                            Coordinate::C1,
                            Coordinate::A1,
                            Coordinate::D1,
                        )
                    }
                }
                Color::Black => {
                    if m.castling_side == CastlingSide::Kingside {
                        (
                            Coordinate::E8,
                            Coordinate::G8,
                            Coordinate::H8,
                            Coordinate::F8,
                        )
                    } else {
                        (
                            Coordinate::E8,
                            Coordinate::C8,
                            Coordinate::A8,
                            Coordinate::D8,
                        )
                    }
                }
            };

            let king = self.pieces[king_src as usize].take();
            let rook = self.pieces[rook_src as usize].take();
            self.pieces[king_dest as usize] = king;
            self.pieces[rook_dest as usize] = rook;

            self.disable_castling(player_color, true);
            self.disable_castling(player_color, false);
        }

        self.player_turn = self.get_opposing_player_color();
        self.last_move = Some(*m);
    }

    pub fn get_king_coordinate(&self, color: Color) -> Coordinate {
        for (i, piece) in self.pieces.iter().enumerate() {
            match piece {
                Some(piece) => {
                    if piece.color == color && piece.piece_type == PieceType::King {
                        return Coordinate::try_from(i as u8).unwrap();
                    }
                }
                None => {}
            }
        }

        panic!("Missing {:?} king on the board", color);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn other_color(&self) -> Color {
        if self.is_white() {
            Color::Black
        } else {
            Color::White
        }
    }

    pub fn is_white(&self) -> bool {
        *self == Color::White
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PieceType {
    Pawn,
    Bishop,
    Knight,
    Rook,
    King,
    Queen,
}

impl PieceType {
    pub fn new_from_string(s: &str) -> Result<Self, &'static str> {
        let res = match s {
            "B" => PieceType::Bishop,
            "N" => PieceType::Knight,
            "R" => PieceType::Rook,
            "K" => PieceType::King,
            "Q" => PieceType::Queen,
            "" => PieceType::Pawn,
            _ => return Err("Invalid piece type."),
        };

        Ok(res)
    }

    pub fn to_algebraic_notation(&self) -> String {
        match *self {
            PieceType::Pawn => "".to_string(),
            PieceType::Bishop => "B".to_string(),
            PieceType::Knight => "N".to_string(),
            PieceType::Rook => "R".to_string(),
            PieceType::King => "K".to_string(),
            PieceType::Queen => "Q".to_string(),
        }
    }

    pub fn promotable_piece_types() -> Vec<PieceType> {
        vec![
            PieceType::Bishop,
            PieceType::Knight,
            PieceType::Rook,
            PieceType::Queen,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Piece {
    pub color: Color,
    pub piece_type: PieceType,
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let to_print = match self.piece_type {
            PieceType::Pawn => "♙",
            PieceType::Bishop => "♗",
            PieceType::Knight => "♘",
            PieceType::Rook => "♖",
            PieceType::King => "♔",
            PieceType::Queen => "♕",
        };

        match self.color {
            Color::White => write!(f, "{}", to_print.white()),
            Color::Black => write!(f, "{}", to_print.cyan()),
        }
    }
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

// Converts "a" to 1, "b" to 2 and so on, panics if it gets an invalid string
pub fn file_to_index(s: &str) -> u8 {
    (1 + FILE_LIST.iter().position(|f| s.eq(*f)).unwrap()) as u8
}

#[cfg(test)]
mod coord_tests {
    use super::*;

    #[test]
    fn is_in_file() {
        assert!(Coordinate::A3.is_in_file(1));
        assert!(Coordinate::B4.is_in_file(2));
        assert!(Coordinate::C5.is_in_file(3));
        assert!(Coordinate::D2.is_in_file(4));
        assert!(Coordinate::E3.is_in_file(5));
        assert!(Coordinate::F4.is_in_file(6));
        assert!(Coordinate::G7.is_in_file(7));
        assert!(Coordinate::H8.is_in_file(8));
    }

    #[test]
    fn is_not_in_file() {
        assert!(!Coordinate::A3.is_in_file(2));
        assert!(!Coordinate::B4.is_in_file(3));
        assert!(!Coordinate::C5.is_in_file(6));
        assert!(!Coordinate::D2.is_in_file(1));
        assert!(!Coordinate::E3.is_in_file(3));
        assert!(!Coordinate::F4.is_in_file(3));
        assert!(!Coordinate::G7.is_in_file(2));
        assert!(!Coordinate::H8.is_in_file(6));
    }

    #[test]
    fn side_squares_left_side_board() {
        let src_square = Coordinate::A3;
        let side_squares = src_square.side_squares();
        assert!(side_squares.into_iter().eq(vec![Coordinate::B3]));
    }

    #[test]
    fn side_squares_right_side_board() {
        let src_square = Coordinate::H8;
        let side_squares = src_square.side_squares();
        assert!(side_squares.into_iter().eq(vec![Coordinate::G8]));
    }

    #[test]
    fn side_squares_middle_board() {
        let src_square = Coordinate::E4;
        let side_squares = src_square.side_squares();
        assert!(side_squares
            .into_iter()
            .eq(vec![Coordinate::D4, Coordinate::F4]));
    }

    #[test]
    fn vertical_offsets() {
        let src_square = Coordinate::E4;
        assert_eq!(src_square.vertical_offset(1, true), Coordinate::E5);
        assert_eq!(src_square.vertical_offset(2, true), Coordinate::E6);
        assert_eq!(src_square.vertical_offset(3, true), Coordinate::E7);
        assert_eq!(src_square.vertical_offset(4, true), Coordinate::E8);
        assert_eq!(src_square.vertical_offset(1, false), Coordinate::E3);
        assert_eq!(src_square.vertical_offset(2, false), Coordinate::E2);
        assert_eq!(src_square.vertical_offset(3, false), Coordinate::E1);
    }

    #[test]
    #[should_panic]
    fn invalid_vertical_offset_above_board() {
        let src_square = Coordinate::E4;
        src_square.vertical_offset(5, true);
    }

    #[test]
    #[should_panic]
    fn invalid_vertical_offset_below_board() {
        let src_square = Coordinate::E4;
        src_square.vertical_offset(4, false);
    }

    #[test]
    fn horizontal_offsets() {
        let src_square = Coordinate::E4;
        assert_eq!(src_square.horizontal_offset(1, true), Coordinate::D4);
        assert_eq!(src_square.horizontal_offset(2, true), Coordinate::C4);
        assert_eq!(src_square.horizontal_offset(3, true), Coordinate::B4);
        assert_eq!(src_square.horizontal_offset(1, false), Coordinate::F4);
        assert_eq!(src_square.horizontal_offset(2, false), Coordinate::G4);
        assert_eq!(src_square.horizontal_offset(3, false), Coordinate::H4);
    }

    #[test]
    #[should_panic]
    // TODO: Consider adding a test to ensure that we cannot
    // go over the side of the board.
    fn invalid_horizontal_offsets_below_board() {
        let src_square = Coordinate::A1;
        src_square.horizontal_offset(1, true);
    }

    #[test]
    #[should_panic]
    fn invalid_horizontal_offsets_above_board() {
        let src_square = Coordinate::G8;
        src_square.horizontal_offset(2, false);
    }

    #[test]
    fn diagonal_offsets_center_square() {
        let src_square = Coordinate::E4;
        assert_eq!(src_square.diagonal_offset(true, true), Coordinate::D5);
        assert_eq!(src_square.diagonal_offset(true, false), Coordinate::F5);
        assert_eq!(src_square.diagonal_offset(false, true), Coordinate::D3);
        assert_eq!(src_square.diagonal_offset(false, false), Coordinate::F3);
    }

    #[test]
    #[should_panic]
    fn diagonal_offsets_top_edge_square() {
        let src_square = Coordinate::G8;
        src_square.diagonal_offset(true, true);
    }

    #[test]
    #[should_panic]
    fn diagonal_offsets_btm_edge_square() {
        let src_square = Coordinate::G1;
        src_square.diagonal_offset(false, true);
    }

    // TODO: Make these tests work
    // #[test]
    // #[should_panic]
    // fn diagonal_offsets_left_edge_square() {
    //     let src_square = Coordinate::A4;
    //     src_square.diagonal_offset(true, true);
    // }

    // #[test]
    // #[should_panic]
    // fn diagonal_offsets_right_edge_square() {
    //     let src_square = Coordinate::H4;
    //     src_square.diagonal_offset(true, false);
    // }

    #[test]
    fn algebraic_notation_of_squares() {
        assert_eq!(Coordinate::E4.to_algebraic_notation(), "e4".to_string());
        assert_eq!(Coordinate::F5.to_algebraic_notation(), "f5".to_string());
        assert_eq!(Coordinate::C8.to_algebraic_notation(), "c8".to_string());
    }
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
mod castling_rights_tests {
    use super::*;

    #[test]
    fn default_castling_rights_disabled() {
        let castling_rights = CastlingRights::new_with_all_disabled();
        assert!(!castling_rights.get_white_kingside());
        assert!(!castling_rights.get_white_queenside());
        assert!(!castling_rights.get_black_kingside());
        assert!(!castling_rights.get_black_queenside());
    }

    #[test]
    fn default_castling_rights_enabled() {
        let castling_rights = CastlingRights::new_with_all_enabled();
        assert!(castling_rights.get_white_kingside());
        assert!(castling_rights.get_white_queenside());
        assert!(castling_rights.get_black_kingside());
        assert!(castling_rights.get_black_queenside());
    }

    #[test]
    fn enabling_white_kingside() {
        let mut castling_rights = CastlingRights::new_with_all_disabled();

        assert!(!castling_rights.get_white_kingside());
        castling_rights.set_white_kingside(true);
        assert!(castling_rights.get_white_kingside());
        assert!(!castling_rights.get_white_queenside());
        assert!(!castling_rights.get_black_kingside());
        assert!(!castling_rights.get_black_queenside());
    }

    #[test]
    fn disabling_black_queenside() {
        let mut castling_rights = CastlingRights::new_with_all_enabled();

        assert!(castling_rights.get_black_queenside());
        castling_rights.set_black_queenside(false);
        assert!(!castling_rights.get_black_queenside());
        assert!(castling_rights.get_white_kingside());
        assert!(castling_rights.get_white_queenside());
        assert!(castling_rights.get_black_kingside());
    }
}

#[cfg(test)]
mod board_tests {
    use super::*;

    #[test]
    fn apply_simple_piece_displacement_to_board() {
        let mut board = Board::new_empty();
        let piece = Piece {
            color: Color::White,
            piece_type: PieceType::Bishop,
        };

        board.place_piece(Coordinate::E4, piece);

        let m = Move::new(Coordinate::E4, Coordinate::H7, piece, false);

        assert_eq!(board.get_player_color(), Color::White);

        board.apply_move(&m);

        assert_eq!(board.get_player_color(), Color::Black);
        // Check that the piece has moved
        assert!(board.get_from_coordinate(Coordinate::E4).is_none());
        assert_eq!(board.get_from_coordinate(Coordinate::H7).unwrap(), piece);
    }

    #[test]
    fn apply_pawn_capture_to_board() {
        let mut board = Board::new_empty();
        let white_pawn = Piece {
            color: Color::White,
            piece_type: PieceType::Pawn,
        };
        let black_pawn = Piece {
            color: Color::Black,
            piece_type: PieceType::Pawn,
        };

        board.place_piece(Coordinate::E4, white_pawn);
        board.place_piece(Coordinate::F5, black_pawn);

        let m = Move::new(Coordinate::E4, Coordinate::F5, white_pawn, true);

        assert_eq!(board.get_player_color(), Color::White);

        board.apply_move(&m);

        assert_eq!(board.get_player_color(), Color::Black);
        // Check that the white pawn has moved
        assert!(board.get_from_coordinate(Coordinate::E4).is_none());
        assert_eq!(
            board.get_from_coordinate(Coordinate::F5).unwrap(),
            white_pawn
        );
    }

    #[test]
    fn apply_white_kingside_castling_to_board() {
        let mut board = Board::new_empty();
        let king = Piece {
            color: Color::White,
            piece_type: PieceType::King,
        };
        let rook = Piece {
            color: Color::White,
            piece_type: PieceType::Rook,
        };

        board.place_piece(Coordinate::E1, king);
        board.place_piece(Coordinate::H1, rook);

        let m = Move::new_castling(Color::White, true);

        assert_eq!(board.get_player_color(), Color::White);

        board.apply_move(&m);

        assert_eq!(board.get_player_color(), Color::Black);
        assert!(board.get_from_coordinate(Coordinate::E1).is_none());
        assert!(board.get_from_coordinate(Coordinate::H1).is_none());
        assert_eq!(board.get_from_coordinate(Coordinate::G1).unwrap(), king);
        assert_eq!(board.get_from_coordinate(Coordinate::F1).unwrap(), rook);
    }

    #[test]
    fn apply_white_queenside_castling_to_board() {
        let mut board = Board::new_empty();
        let king = Piece {
            color: Color::White,
            piece_type: PieceType::King,
        };
        let rook = Piece {
            color: Color::White,
            piece_type: PieceType::Rook,
        };

        board.place_piece(Coordinate::E1, king);
        board.place_piece(Coordinate::A1, rook);

        let m = Move::new_castling(Color::White, false);

        assert_eq!(board.get_player_color(), Color::White);

        board.apply_move(&m);

        assert_eq!(board.get_player_color(), Color::Black);
        assert!(board.get_from_coordinate(Coordinate::E1).is_none());
        assert!(board.get_from_coordinate(Coordinate::A1).is_none());
        assert_eq!(board.get_from_coordinate(Coordinate::C1).unwrap(), king);
        assert_eq!(board.get_from_coordinate(Coordinate::D1).unwrap(), rook);
    }

    #[test]
    fn apply_black_kingside_castling_to_board() {
        let mut board = Board::new_empty();
        board.set_player_color(Color::Black);
        let king = Piece {
            color: Color::Black,
            piece_type: PieceType::King,
        };
        let rook = Piece {
            color: Color::Black,
            piece_type: PieceType::Rook,
        };

        board.place_piece(Coordinate::E8, king);
        board.place_piece(Coordinate::H8, rook);

        let m = Move::new_castling(Color::Black, true);

        assert_eq!(board.get_player_color(), Color::Black);

        board.apply_move(&m);

        assert_eq!(board.get_player_color(), Color::White);
        assert!(board.get_from_coordinate(Coordinate::E8).is_none());
        assert!(board.get_from_coordinate(Coordinate::H8).is_none());
        assert_eq!(board.get_from_coordinate(Coordinate::G8).unwrap(), king);
        assert_eq!(board.get_from_coordinate(Coordinate::F8).unwrap(), rook);
    }

    #[test]
    fn apply_black_queenside_castling_to_board() {
        let mut board = Board::new_empty();
        board.set_player_color(Color::Black);
        let king = Piece {
            color: Color::Black,
            piece_type: PieceType::King,
        };
        let rook = Piece {
            color: Color::Black,
            piece_type: PieceType::Rook,
        };

        board.place_piece(Coordinate::E8, king);
        board.place_piece(Coordinate::A8, rook);

        let m = Move::new_castling(Color::Black, false);

        assert_eq!(board.get_player_color(), Color::Black);

        board.apply_move(&m);

        assert_eq!(board.get_player_color(), Color::White);
        assert!(board.get_from_coordinate(Coordinate::E8).is_none());
        assert!(board.get_from_coordinate(Coordinate::A8).is_none());
        assert_eq!(board.get_from_coordinate(Coordinate::C8).unwrap(), king);
        assert_eq!(board.get_from_coordinate(Coordinate::D8).unwrap(), rook);
    }

    #[test]
    fn apply_white_en_passant_to_board() {
        let mut board = Board::new_empty();
        let white_pawn = Piece {
            color: Color::White,
            piece_type: PieceType::Pawn,
        };
        let black_pawn = Piece {
            color: Color::Black,
            piece_type: PieceType::Pawn,
        };

        board.place_piece(Coordinate::E5, white_pawn);
        board.place_piece(Coordinate::F5, black_pawn);

        let mut m = Move::new(Coordinate::E5, Coordinate::F6, white_pawn, true);
        m.is_en_passant = true;

        assert_eq!(board.get_player_color(), Color::White);

        board.apply_move(&m);

        assert_eq!(board.get_player_color(), Color::Black);
        // Check that the white pawn has moved
        assert!(board.get_from_coordinate(Coordinate::E5).is_none());
        assert!(board.get_from_coordinate(Coordinate::F5).is_none());
        assert_eq!(
            board.get_from_coordinate(Coordinate::F6).unwrap(),
            white_pawn
        );
    }

    #[test]
    fn apply_black_en_passant_to_board() {
        let mut board = Board::new_empty();
        board.set_player_color(Color::Black);

        let white_pawn = Piece {
            color: Color::White,
            piece_type: PieceType::Pawn,
        };
        let black_pawn = Piece {
            color: Color::Black,
            piece_type: PieceType::Pawn,
        };

        board.place_piece(Coordinate::E4, white_pawn);
        board.place_piece(Coordinate::F4, black_pawn);

        let mut m = Move::new(Coordinate::F4, Coordinate::E3, black_pawn, true);
        m.is_en_passant = true;

        assert_eq!(board.get_player_color(), Color::Black);

        board.apply_move(&m);

        assert_eq!(board.get_player_color(), Color::White);
        // Check that the white pawn has moved
        assert!(board.get_from_coordinate(Coordinate::F4).is_none());
        assert!(board.get_from_coordinate(Coordinate::E4).is_none());
        assert_eq!(
            board.get_from_coordinate(Coordinate::E3).unwrap(),
            black_pawn
        );
    }

    #[test]
    fn apply_promotion_to_board() {
        let mut board = Board::new_empty();

        let white_pawn = Piece {
            color: Color::White,
            piece_type: PieceType::Pawn,
        };
        let white_knight = Piece {
            color: Color::White,
            piece_type: PieceType::Knight,
        };

        board.place_piece(Coordinate::E7, white_pawn);

        let mut m = Move::new(Coordinate::E7, Coordinate::E8, white_pawn, false);
        m.promotes_to = Some(PieceType::Knight);

        assert_eq!(board.get_player_color(), Color::White);

        board.apply_move(&m);

        assert_eq!(board.get_player_color(), Color::Black);
        // Check that the white pawn has moved
        assert!(board.get_from_coordinate(Coordinate::E7).is_none());
        assert_eq!(
            board.get_from_coordinate(Coordinate::E8).unwrap(),
            white_knight
        );
    }

    #[test]
    fn apply_promotion_with_captures_to_board() {
        let mut board = Board::new_empty();

        let white_pawn = Piece {
            color: Color::White,
            piece_type: PieceType::Pawn,
        };
        let white_queen = Piece {
            color: Color::White,
            piece_type: PieceType::Queen,
        };
        let black_rook = Piece {
            color: Color::Black,
            piece_type: PieceType::Rook,
        };

        board.place_piece(Coordinate::E7, white_pawn);
        board.place_piece(Coordinate::F8, black_rook);

        let mut m = Move::new(Coordinate::E7, Coordinate::F8, white_pawn, true);
        m.promotes_to = Some(PieceType::Queen);

        assert_eq!(board.get_player_color(), Color::White);

        board.apply_move(&m);

        assert_eq!(board.get_player_color(), Color::Black);
        // Check that the white pawn has moved
        assert!(board.get_from_coordinate(Coordinate::E7).is_none());
        assert_eq!(
            board.get_from_coordinate(Coordinate::F8).unwrap(),
            white_queen
        );
    }
}
