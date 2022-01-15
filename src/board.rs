use colored::Colorize;
use num_enum::TryFromPrimitive;
use std::convert::TryFrom;
use std::fmt;

#[repr(u8)]
#[allow(dead_code)]
#[derive(TryFromPrimitive, Debug, Clone, Copy, PartialEq)]
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

    // Returns true if this coordinate is in a certain rank
    // Rank should be between 1 to 8
    pub fn is_in_rank(&self, rank: u8) -> bool {
        if rank < 1 || rank > 8 {
            panic!("Invalid parameter passed to `is_in_rank`.`");
        }
        let coord_val = *self as u8;

        return coord_val >= (rank - 1) * 8 && coord_val < rank * 8;
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
}

pub struct Board {
    pieces: [Option<Piece>; 64],
    is_game_over: bool,
    player_turn: Color,
}

impl Board {
    pub fn new_empty() -> Board {
        Board {
            pieces: [None; 64],
            is_game_over: false,
            player_turn: Color::White,
        }
    }

    pub fn new_starting_pos() -> Board {
        let mut board = Board::new_empty();
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
        for (i, piece) in self.pieces.iter().enumerate() {
            // We add spaces before and after the piece
            match piece {
                Some(p) => print!(" {} ", p),
                None => {
                    // Use 'x' to represent an empty square
                    print!(" x ");
                }
            }

            // Newline after 8 pieces
            if (i + 1) % 8 == 0 {
                println!("");
            }
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

    pub fn get_opposing_player_color(&self) -> Color {
        self.player_turn.other_color()
    }

    pub fn is_square_occupied(&self, coord: Coordinate) -> bool {
        return self.pieces[coord as usize].is_some();
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn other_color(&self) -> Color {
        if *self == Color::White {
            Color::Black
        } else {
            Color::White
        }
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
enum Direction {
    N = 0,
    NE = 1,
    E = 2,
    SE = 3,
    S = 4,
    SW = 5,
    W = 6,
    NW = 7,
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

    fn get(&self, src: Coordinate, dir: Direction) -> Option<Coordinate> {
        self.table[src as usize][dir as usize]
    }
}

#[cfg(test)]
mod coord_tests {
    use super::*;

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
