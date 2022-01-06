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
}
