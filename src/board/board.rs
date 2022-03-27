use crate::board::movements::is_square_controlled_by_player;
use crate::r#move::{CastlingSide, Move};
use colored::Colorize;
use num_enum::TryFromPrimitive;
use std::convert::TryFrom;
use std::fmt;

use regex::Regex;
use strum_macros::EnumIter;

pub static FILE_LIST: [&str; 8] = ["a", "b", "c", "d", "e", "f", "g", "h"];

#[repr(usize)]
#[allow(dead_code)]
#[derive(TryFromPrimitive, Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
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
    pub fn new_from_rank_file(rank: usize, file: usize) -> Coordinate {
        Coordinate::try_from((rank - 1) * 8 + (file - 1)).unwrap()
    }

    pub fn new_from_algebraic_notation(s: &str) -> Coordinate {
        let file = file_to_index(&s.chars().nth(0).unwrap().to_string());
        let rank = s.chars().nth(1).unwrap().to_digit(10).unwrap() as usize;

        Coordinate::new_from_rank_file(rank, file)
    }

    pub fn new_from_long_algebraic_notation(
        s: &str,
    ) -> (Coordinate, Coordinate, Option<PieceType>) {
        lazy_static! {
            static ref MOVE_STRING_REGEX: Regex =
                Regex::new(r"^([a-h][1-8])([a-h][1-8])([nbrq])?$").unwrap();
        }
        let m = MOVE_STRING_REGEX.captures(s).unwrap();
        let promotion_piece_type = m
            .get(3)
            .map(|p| PieceType::new_from_string(p.as_str()).unwrap());

        (
            Coordinate::new_from_algebraic_notation(&m[1]),
            Coordinate::new_from_algebraic_notation(&m[2]),
            promotion_piece_type,
        )
    }

    // Returns the coordinate of the square some vertical offset
    // from this one. The offset can either be a front or
    // backward offset.
    pub fn vertical_offset(&self, offset: usize, front: bool) -> Coordinate {
        let offset_val = if front {
            *self as usize + 8 * offset
        } else {
            *self as usize - 8 * offset
        };
        return Coordinate::try_from(offset_val).expect("Invalid coordinate");
    }

    // Returns the coordinate of the square some horizontal offset
    // from this one. The offset can either be a left or
    // right offset.
    pub fn horizontal_offset(&self, offset: usize, left: bool) -> Coordinate {
        let offset_val = if left {
            *self as usize - offset
        } else {
            *self as usize + offset
        };
        return Coordinate::try_from(offset_val).expect("Invalid coordinate");
    }

    // Returns the coordinate of the square that is at a diagonal
    // offset from this one. The combination of the front and left
    // booleans determine which one of four directions the offset
    // should be.
    pub fn diagonal_offset(&self, front: bool, left: bool) -> Coordinate {
        let mut offset_val = *self as usize;

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
    pub fn get_rank(&self) -> usize {
        (*self as usize / 8) + 1
    }

    // Returns value 1..=8 corresponding to the file of the square
    pub fn get_file(&self) -> usize {
        (*self as usize % 8) + 1
    }

    // Returns true if this coordinate is in a certain rank
    // Rank should be between 1 to 8
    pub fn is_in_rank(&self, rank: usize) -> bool {
        if rank < 1 || rank > 8 {
            panic!("Invalid parameter passed to `is_in_rank`.");
        }

        return self.get_rank() == rank;
    }

    // Returns true if this coordinate is in a certain file
    // File should be between 1 to 8
    pub fn is_in_file(&self, file: usize) -> bool {
        if file < 1 || file > 8 {
            panic!("Invalid parameter passed to `is_in_file`.");
        }
        return self.get_file() == file;
    }

    pub fn side_squares(&self) -> Vec<Coordinate> {
        let mut res = vec![];
        // i8 because the idx could be negative after applying the offset
        let coord_idx = *self as usize;
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

    pub fn rank_difference(&self, coord: Coordinate) -> i8 {
        return self.get_rank() as i8 - coord.get_rank() as i8;
    }

    pub fn file_difference(&self, coord: Coordinate) -> i8 {
        return self.get_file() as i8 - coord.get_file() as i8;
    }
}

bitfield! {
    #[derive(Clone, Copy, Debug, PartialEq)]
    struct CastlingRights(u8);
    get_white_kingside, set_white_kingside: 0;
    get_white_queenside, set_white_queenside: 1;
    get_black_kingside, set_black_kingside: 2;
    get_black_queenside, set_black_queenside: 3;
}

impl CastlingRights {
    pub fn new_with_all_disabled() -> Self {
        CastlingRights(0)
    }

    pub fn new_with_all_enabled() -> Self {
        CastlingRights(0b1111)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Phase {
    Midgame,
    Endgame,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Board {
    pieces: [Option<Piece>; 64],
    player_turn: Color,
    castling_rights: CastlingRights,
    en_passant_square: Option<Coordinate>,
    // Non-pawn material
    npm: i32, // Note: Caching this to avoid constant recalculation
}

impl Board {
    pub fn new_empty() -> Board {
        Board {
            pieces: [None; 64],
            player_turn: Color::White,
            castling_rights: CastlingRights::new_with_all_disabled(),
            en_passant_square: None,
            npm: 0,
        }
    }

    pub fn new_starting_pos() -> Board {
        let mut board = Board::new_empty();

        // At the starting position, both players have their castling rights
        board.castling_rights = CastlingRights::new_with_all_enabled();

        let pieces = [
            (Coordinate::A1, Color::White, PieceType::Rook),
            (Coordinate::B1, Color::White, PieceType::Knight),
            (Coordinate::C1, Color::White, PieceType::Bishop),
            (Coordinate::D1, Color::White, PieceType::Queen),
            (Coordinate::E1, Color::White, PieceType::King),
            (Coordinate::F1, Color::White, PieceType::Bishop),
            (Coordinate::G1, Color::White, PieceType::Knight),
            (Coordinate::H1, Color::White, PieceType::Rook),
            (Coordinate::A2, Color::White, PieceType::Pawn),
            (Coordinate::B2, Color::White, PieceType::Pawn),
            (Coordinate::C2, Color::White, PieceType::Pawn),
            (Coordinate::D2, Color::White, PieceType::Pawn),
            (Coordinate::E2, Color::White, PieceType::Pawn),
            (Coordinate::F2, Color::White, PieceType::Pawn),
            (Coordinate::G2, Color::White, PieceType::Pawn),
            (Coordinate::H2, Color::White, PieceType::Pawn),
            (Coordinate::A8, Color::Black, PieceType::Rook),
            (Coordinate::B8, Color::Black, PieceType::Knight),
            (Coordinate::C8, Color::Black, PieceType::Bishop),
            (Coordinate::D8, Color::Black, PieceType::Queen),
            (Coordinate::E8, Color::Black, PieceType::King),
            (Coordinate::F8, Color::Black, PieceType::Bishop),
            (Coordinate::G8, Color::Black, PieceType::Knight),
            (Coordinate::H8, Color::Black, PieceType::Rook),
            (Coordinate::A7, Color::Black, PieceType::Pawn),
            (Coordinate::B7, Color::Black, PieceType::Pawn),
            (Coordinate::C7, Color::Black, PieceType::Pawn),
            (Coordinate::D7, Color::Black, PieceType::Pawn),
            (Coordinate::E7, Color::Black, PieceType::Pawn),
            (Coordinate::F7, Color::Black, PieceType::Pawn),
            (Coordinate::G7, Color::Black, PieceType::Pawn),
            (Coordinate::H7, Color::Black, PieceType::Pawn),
        ];

        for (coord, color, piece_type) in pieces {
            board.place_piece(coord, Piece::new(color, piece_type));
        }

        board.update_npm();
        board
    }

    pub fn place_piece(&mut self, coord: Coordinate, piece: Piece) {
        self.pieces[coord as usize] = Some(piece);
    }

    pub fn remove_piece(&mut self, coord: Coordinate) {
        self.pieces[coord as usize] = None;
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

    pub fn get_pieces(&self) -> &[Option<Piece>] {
        &self.pieces
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
            let dest_piece = self.pieces[m.dest as usize].replace(original_piece.unwrap());

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

            // Check if captured piece is a rook to disable castling rights
            if let Some(dest_piece) = dest_piece {
                if dest_piece.color == player_color.other_color()
                    && dest_piece.piece_type == PieceType::Rook
                {
                    match player_color {
                        Color::Black => {
                            if m.dest == Coordinate::A1 {
                                self.disable_castling(Color::White, false);
                            } else if m.dest == Coordinate::H1 {
                                self.disable_castling(Color::White, true);
                            }
                        }
                        Color::White => {
                            if m.dest == Coordinate::A8 {
                                self.disable_castling(Color::Black, false);
                            } else if m.dest == Coordinate::H8 {
                                self.disable_castling(Color::Black, true);
                            }
                        }
                    }
                }
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

        // Check if move takes away castling rights
        if m.piece.piece_type == PieceType::King {
            self.disable_castling(player_color, true);
            self.disable_castling(player_color, false);
        } else if m.piece.piece_type == PieceType::Rook {
            match player_color {
                Color::White => {
                    if m.src == Coordinate::A1 {
                        self.disable_castling(player_color, false);
                    } else if m.src == Coordinate::H1 {
                        self.disable_castling(player_color, true);
                    }
                }
                Color::Black => {
                    if m.src == Coordinate::A8 {
                        self.disable_castling(player_color, false);
                    } else if m.src == Coordinate::H8 {
                        self.disable_castling(player_color, true);
                    }
                }
            }
        }

        // Set en passant square
        self.en_passant_square = if m.is_pawn_double_advance() {
            // Get the square 'behind' the pawn
            match m.piece.color {
                Color::White => Some(m.dest.vertical_offset(1, false)),
                Color::Black => Some(m.dest.vertical_offset(1, true)),
            }
        } else {
            None
        };

        self.player_turn = self.get_opposing_player_color();

        // TODO: Find a more efficient way of doing this
        self.update_npm();
    }

    pub fn apply_move_with_src_dest(
        &mut self,
        src: Coordinate,
        dest: Coordinate,
        promotes_to: Option<PieceType>,
    ) -> Result<(), &'static str> {
        let src_piece = match self.get_from_coordinate(src) {
            Some(p) => p,
            None => return Err("Missing piece on source square"),
        };
        let (captured_piece_type, is_en_passant) = match self.get_from_coordinate(dest) {
            Some(p) => {
                if p.color == src_piece.color {
                    return Err("Illegal capture");
                }
                (Some(p.piece_type), false)
            }
            None => {
                // Check for en passant
                match self.en_passant_square {
                    Some(eps) => {
                        if src_piece.piece_type == PieceType::Pawn && dest == eps {
                            (Some(PieceType::Pawn), true)
                        } else {
                            (None, false)
                        }
                    }
                    None => (None, false),
                }
            }
        };

        // TODO: Verify if the piece is allowed to move from
        // src to dest
        let mut m = match captured_piece_type {
            Some(piece_type) => Move::new_capture(src, dest, src_piece, piece_type),
            None => Move::new(src, dest, src_piece),
        };
        m.is_en_passant = is_en_passant;
        m.promotes_to = promotes_to;

        // Check for castling
        if src_piece.piece_type == PieceType::King && src.file_difference(dest).abs() == 2 {
            // Assume that we have a legal move and skip other checks for
            // performance
            let is_kingside = dest.get_file() == 7;
            m.castling_side = if is_kingside {
                CastlingSide::Kingside
            } else {
                CastlingSide::Queenside
            };
        }
        self.apply_move(&m);
        Ok(())
    }

    pub fn get_king_coordinate(&self, color: Color) -> Coordinate {
        for (i, piece) in self.pieces.iter().enumerate() {
            match piece {
                Some(piece) => {
                    if piece.color == color && piece.piece_type == PieceType::King {
                        return Coordinate::try_from(i as usize).unwrap();
                    }
                }
                None => {}
            }
        }

        panic!("Missing {:?} king on the board", color);
    }

    pub fn is_in_check(&self) -> bool {
        let king_coord = self.get_king_coordinate(self.get_player_color());
        is_square_controlled_by_player(self, self.get_opposing_player_color(), king_coord)
    }

    // If a pawn can capture this square, it means that it is
    // able to capture en passant
    pub fn get_en_passant_square(&self) -> Option<Coordinate> {
        self.en_passant_square
    }

    pub fn set_en_passant_square(&mut self, coord: Coordinate) {
        self.en_passant_square = Some(coord);
    }

    pub fn update_npm(&mut self) {
        self.npm = crate::engine::non_pawn_material(self);
    }

    pub fn get_npm(&self) -> i32 {
        self.npm
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

#[derive(Debug, Clone, Copy, PartialEq, EnumIter)]
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
            "B" | "b" => PieceType::Bishop,
            "N" | "n" => PieceType::Knight,
            "R" | "r" => PieceType::Rook,
            "K" | "k" => PieceType::King,
            "Q" | "q" => PieceType::Queen,
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

impl Piece {
    pub fn new(color: Color, piece_type: PieceType) -> Self {
        Piece { color, piece_type }
    }
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

// Converts "a" to 1, "b" to 2 and so on, panics if it gets an invalid string
pub fn file_to_index(s: &str) -> usize {
    1 + FILE_LIST.iter().position(|f| s.eq(*f)).unwrap()
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

        let m = Move::new(Coordinate::E4, Coordinate::H7, piece);

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

        let m = Move::new_capture(Coordinate::E4, Coordinate::F5, white_pawn, PieceType::Pawn);

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

        let mut m = Move::new_capture(Coordinate::E5, Coordinate::F6, white_pawn, PieceType::Pawn);
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

        let mut m = Move::new_capture(Coordinate::F4, Coordinate::E3, black_pawn, PieceType::Pawn);
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

        let mut m = Move::new(Coordinate::E7, Coordinate::E8, white_pawn);
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

        let mut m = Move::new_capture(Coordinate::E7, Coordinate::F8, white_pawn, PieceType::Rook);
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

    #[test]
    fn white_king_movement_removes_castling_rights() {
        let mut board = Board::new_empty();

        let white_king = Piece {
            color: Color::White,
            piece_type: PieceType::King,
        };

        board.place_piece(Coordinate::E1, white_king);
        board.enable_castling(Color::White, true);
        board.enable_castling(Color::White, false);

        let m = Move::new(Coordinate::E1, Coordinate::E2, white_king);

        assert_eq!(board.get_player_color(), Color::White);
        assert!(board.may_castle(Color::White, true));
        assert!(board.may_castle(Color::White, false));

        board.apply_move(&m);

        assert!(!board.may_castle(Color::White, true));
        assert!(!board.may_castle(Color::White, false));
    }

    #[test]
    fn white_queens_rook_movement_removes_queenside_castling_rights() {
        let mut board = Board::new_empty();

        let white_rook = Piece {
            color: Color::White,
            piece_type: PieceType::Rook,
        };

        board.place_piece(Coordinate::A1, white_rook);
        board.enable_castling(Color::White, true);
        board.enable_castling(Color::White, false);

        let m = Move::new(Coordinate::A1, Coordinate::A2, white_rook);

        assert_eq!(board.get_player_color(), Color::White);
        assert!(board.may_castle(Color::White, true));
        assert!(board.may_castle(Color::White, false));

        board.apply_move(&m);

        assert!(board.may_castle(Color::White, true));
        assert!(!board.may_castle(Color::White, false));
    }

    #[test]
    fn white_kings_rook_movement_removes_kingside_castling_rights() {
        let mut board = Board::new_empty();

        let white_rook = Piece {
            color: Color::White,
            piece_type: PieceType::Rook,
        };

        board.place_piece(Coordinate::H1, white_rook);
        board.enable_castling(Color::White, true);
        board.enable_castling(Color::White, false);

        let m = Move::new(Coordinate::H1, Coordinate::H2, white_rook);

        assert_eq!(board.get_player_color(), Color::White);
        assert!(board.may_castle(Color::White, true));
        assert!(board.may_castle(Color::White, false));

        board.apply_move(&m);

        assert!(!board.may_castle(Color::White, true));
        assert!(board.may_castle(Color::White, false));
    }

    #[test]
    fn black_king_movement_removes_castling_rights() {
        let mut board = Board::new_empty();
        board.set_player_color(Color::Black);

        let black_king = Piece {
            color: Color::Black,
            piece_type: PieceType::King,
        };

        board.place_piece(Coordinate::E1, black_king);
        board.enable_castling(Color::Black, true);
        board.enable_castling(Color::Black, false);

        let m = Move::new(Coordinate::E1, Coordinate::E2, black_king);

        assert_eq!(board.get_player_color(), Color::Black);
        assert!(board.may_castle(Color::Black, true));
        assert!(board.may_castle(Color::Black, false));

        board.apply_move(&m);

        assert!(!board.may_castle(Color::Black, true));
        assert!(!board.may_castle(Color::Black, false));
    }

    #[test]
    fn black_queens_rook_movement_removes_queenside_castling_rights() {
        let mut board = Board::new_empty();
        board.set_player_color(Color::Black);

        let black_rook = Piece {
            color: Color::Black,
            piece_type: PieceType::Rook,
        };

        board.place_piece(Coordinate::A8, black_rook);
        board.enable_castling(Color::Black, true);
        board.enable_castling(Color::Black, false);

        let m = Move::new(Coordinate::A8, Coordinate::A7, black_rook);

        assert_eq!(board.get_player_color(), Color::Black);
        assert!(board.may_castle(Color::Black, true));
        assert!(board.may_castle(Color::Black, false));

        board.apply_move(&m);

        assert!(board.may_castle(Color::Black, true));
        assert!(!board.may_castle(Color::Black, false));
    }

    #[test]
    fn black_kings_rook_movement_removes_kingside_castling_rights() {
        let mut board = Board::new_empty();
        board.set_player_color(Color::Black);

        let black_rook = Piece {
            color: Color::Black,
            piece_type: PieceType::Rook,
        };

        board.place_piece(Coordinate::H8, black_rook);
        board.enable_castling(Color::Black, true);
        board.enable_castling(Color::Black, false);

        let m = Move::new(Coordinate::H8, Coordinate::H7, black_rook);

        assert_eq!(board.get_player_color(), Color::Black);
        assert!(board.may_castle(Color::Black, true));
        assert!(board.may_castle(Color::Black, false));

        board.apply_move(&m);

        assert!(!board.may_castle(Color::Black, true));
        assert!(board.may_castle(Color::Black, false));
    }

    #[test]
    fn capturing_kingside_rook_disables_white_kingside_castling() {
        let mut board = Board::new_empty();
        let king = Piece {
            color: Color::White,
            piece_type: PieceType::King,
        };
        let rook = Piece {
            color: Color::White,
            piece_type: PieceType::Rook,
        };
        let enemy_rook = Piece {
            color: Color::Black,
            piece_type: PieceType::Rook,
        };

        board.place_piece(Coordinate::E1, king);
        board.place_piece(Coordinate::H1, rook);
        board.place_piece(Coordinate::H8, enemy_rook);

        board.enable_castling(Color::White, true);

        let m = Move::new_capture(Coordinate::H8, Coordinate::H1, enemy_rook, PieceType::Rook);

        assert!(board.may_castle(Color::White, true));
        board.apply_move(&m);
        assert!(!board.may_castle(Color::White, true));
    }

    #[test]
    fn capturing_queenside_rook_disables_white_queenside_castling() {
        let mut board = Board::new_empty();
        let king = Piece {
            color: Color::White,
            piece_type: PieceType::King,
        };
        let rook = Piece {
            color: Color::White,
            piece_type: PieceType::Rook,
        };
        let enemy_rook = Piece {
            color: Color::Black,
            piece_type: PieceType::Rook,
        };

        board.place_piece(Coordinate::E1, king);
        board.place_piece(Coordinate::A1, rook);
        board.place_piece(Coordinate::A8, enemy_rook);

        board.enable_castling(Color::White, false);

        let m = Move::new_capture(Coordinate::A8, Coordinate::A1, enemy_rook, PieceType::Rook);

        assert!(board.may_castle(Color::White, false));
        board.apply_move(&m);
        assert!(!board.may_castle(Color::White, false));
    }

    #[test]
    fn capturing_kingside_rook_disables_black_kingside_castling() {
        let mut board = Board::new_empty();
        let king = Piece {
            color: Color::Black,
            piece_type: PieceType::King,
        };
        let rook = Piece {
            color: Color::Black,
            piece_type: PieceType::Rook,
        };
        let enemy_rook = Piece {
            color: Color::White,
            piece_type: PieceType::Rook,
        };

        board.place_piece(Coordinate::E8, king);
        board.place_piece(Coordinate::H8, rook);
        board.place_piece(Coordinate::H1, enemy_rook);

        board.enable_castling(Color::Black, true);

        let m = Move::new_capture(Coordinate::H1, Coordinate::H8, enemy_rook, PieceType::Rook);

        assert!(board.may_castle(Color::Black, true));
        board.apply_move(&m);
        assert!(!board.may_castle(Color::Black, true));
    }

    #[test]
    fn capturing_queenside_rook_disables_black_queenside_castling() {
        let mut board = Board::new_empty();
        let king = Piece {
            color: Color::Black,
            piece_type: PieceType::King,
        };
        let rook = Piece {
            color: Color::Black,
            piece_type: PieceType::Rook,
        };
        let enemy_rook = Piece {
            color: Color::White,
            piece_type: PieceType::Rook,
        };

        board.place_piece(Coordinate::E8, king);
        board.place_piece(Coordinate::A8, rook);
        board.place_piece(Coordinate::A1, enemy_rook);

        board.enable_castling(Color::Black, false);

        let m = Move::new_capture(Coordinate::A1, Coordinate::A8, enemy_rook, PieceType::Rook);

        assert!(board.may_castle(Color::Black, false));
        board.apply_move(&m);
        assert!(!board.may_castle(Color::Black, false));
    }

    #[test]
    fn apply_simple_move_with_src_dest() {
        let mut board = Board::new_empty();
        let piece = Piece {
            color: Color::White,
            piece_type: PieceType::Bishop,
        };

        board.place_piece(Coordinate::E4, piece);

        assert!(board
            .apply_move_with_src_dest(Coordinate::E4, Coordinate::H7, None)
            .is_ok());

        assert!(board.get_from_coordinate(Coordinate::E4).is_none());
        assert_eq!(board.get_from_coordinate(Coordinate::H7).unwrap(), piece);
    }

    #[test]
    fn apply_capture_move_with_src_dest() {
        let mut board = Board::new_empty();
        let piece = Piece::new(Color::White, PieceType::Knight);
        let enemy_piece = Piece::new(Color::Black, PieceType::Pawn);

        board.place_piece(Coordinate::E4, piece);
        board.place_piece(Coordinate::F6, enemy_piece);

        assert!(board
            .apply_move_with_src_dest(Coordinate::E4, Coordinate::F6, None)
            .is_ok());

        assert!(board.get_from_coordinate(Coordinate::E4).is_none());
        assert_eq!(board.get_from_coordinate(Coordinate::F6).unwrap(), piece);
    }

    #[test]
    fn apply_en_passant_capture_move_with_src_dest() {
        let mut board = Board::new_empty();
        let piece = Piece::new(Color::White, PieceType::Pawn);
        let enemy_piece = Piece::new(Color::Black, PieceType::Pawn);

        board.place_piece(Coordinate::E5, piece);
        board.place_piece(Coordinate::D5, enemy_piece);
        board.set_en_passant_square(Coordinate::D6);

        assert!(board
            .apply_move_with_src_dest(Coordinate::E5, Coordinate::D6, None)
            .is_ok());

        assert!(board.get_from_coordinate(Coordinate::E5).is_none());
        assert!(board.get_from_coordinate(Coordinate::D5).is_none());
        assert_eq!(board.get_from_coordinate(Coordinate::D6).unwrap(), piece);
    }

    #[test]
    fn apply_white_kingside_castling_with_src_dest() {
        let mut board = Board::new_empty();
        let king = Piece::new(Color::White, PieceType::King);
        let rook = Piece::new(Color::White, PieceType::Rook);

        board.place_piece(Coordinate::E1, king);
        board.place_piece(Coordinate::H1, rook);

        assert!(board
            .apply_move_with_src_dest(Coordinate::E1, Coordinate::G1, None)
            .is_ok());

        assert!(board.get_from_coordinate(Coordinate::E1).is_none());
        assert!(board.get_from_coordinate(Coordinate::H1).is_none());
        assert_eq!(board.get_from_coordinate(Coordinate::G1).unwrap(), king);
        assert_eq!(board.get_from_coordinate(Coordinate::F1).unwrap(), rook);
    }

    #[test]
    fn apply_white_queenside_castling_with_src_dest() {
        let mut board = Board::new_empty();
        let king = Piece::new(Color::White, PieceType::King);
        let rook = Piece::new(Color::White, PieceType::Rook);

        board.place_piece(Coordinate::E1, king);
        board.place_piece(Coordinate::A1, rook);

        assert!(board
            .apply_move_with_src_dest(Coordinate::E1, Coordinate::C1, None)
            .is_ok());

        assert!(board.get_from_coordinate(Coordinate::E1).is_none());
        assert!(board.get_from_coordinate(Coordinate::A1).is_none());
        assert_eq!(board.get_from_coordinate(Coordinate::C1).unwrap(), king);
        assert_eq!(board.get_from_coordinate(Coordinate::D1).unwrap(), rook);
    }

    #[test]
    fn apply_black_kingside_castling_with_src_dest() {
        let mut board = Board::new_empty();
        let king = Piece::new(Color::Black, PieceType::King);
        let rook = Piece::new(Color::Black, PieceType::Rook);

        board.place_piece(Coordinate::E8, king);
        board.place_piece(Coordinate::H8, rook);

        assert!(board
            .apply_move_with_src_dest(Coordinate::E8, Coordinate::G8, None)
            .is_ok());

        assert!(board.get_from_coordinate(Coordinate::E8).is_none());
        assert!(board.get_from_coordinate(Coordinate::H8).is_none());
        assert_eq!(board.get_from_coordinate(Coordinate::G8).unwrap(), king);
        assert_eq!(board.get_from_coordinate(Coordinate::F8).unwrap(), rook);
    }

    #[test]
    fn apply_black_queenside_castling_with_src_dest() {
        let mut board = Board::new_empty();
        let king = Piece::new(Color::Black, PieceType::King);
        let rook = Piece::new(Color::Black, PieceType::Rook);

        board.place_piece(Coordinate::E8, king);
        board.place_piece(Coordinate::A8, rook);

        assert!(board
            .apply_move_with_src_dest(Coordinate::E8, Coordinate::C8, None)
            .is_ok());

        assert!(board.get_from_coordinate(Coordinate::E8).is_none());
        assert!(board.get_from_coordinate(Coordinate::A8).is_none());
        assert_eq!(board.get_from_coordinate(Coordinate::C8).unwrap(), king);
        assert_eq!(board.get_from_coordinate(Coordinate::D8).unwrap(), rook);
    }

    #[test]
    fn apply_promotion_with_src_dest() {
        let mut board = Board::new_empty();
        let piece = Piece::new(Color::White, PieceType::Pawn);

        board.place_piece(Coordinate::E7, piece);

        assert!(board
            .apply_move_with_src_dest(Coordinate::E7, Coordinate::E8, Some(PieceType::Knight))
            .is_ok());

        assert!(board.get_from_coordinate(Coordinate::E7).is_none());
        assert_eq!(
            board.get_from_coordinate(Coordinate::E8).unwrap(),
            Piece::new(Color::White, PieceType::Knight)
        );
    }
}
