use colored::Colorize;
use num_enum::TryFromPrimitive;
use std::convert::TryFrom;
use std::fmt;

#[repr(u8)]
#[allow(dead_code)]
#[derive(TryFromPrimitive, Clone, Copy)]
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
}

pub struct Board {
    pieces: [Option<Piece>; 64],
    is_game_over: bool,
    player_turn: Color,
}

impl Board {
    fn new_empty() -> Board {
        Board {
            pieces: [None; 64],
            is_game_over: false,
            player_turn: Color::White,
        }
    }

    pub fn new_starting_pos() -> Board {
        let mut board = Board::new_empty();
        board.pieces[Coordinate::A1 as usize] = Some(Piece {
            color: Color::White,
            piece_type: PieceType::Rook,
        });
        board.pieces[Coordinate::B1 as usize] = Some(Piece {
            color: Color::White,
            piece_type: PieceType::Knight,
        });
        board.pieces[Coordinate::C1 as usize] = Some(Piece {
            color: Color::White,
            piece_type: PieceType::Bishop,
        });
        board.pieces[Coordinate::D1 as usize] = Some(Piece {
            color: Color::White,
            piece_type: PieceType::Queen,
        });
        board.pieces[Coordinate::E1 as usize] = Some(Piece {
            color: Color::White,
            piece_type: PieceType::King,
        });
        board.pieces[Coordinate::F1 as usize] = Some(Piece {
            color: Color::White,
            piece_type: PieceType::Bishop,
        });
        board.pieces[Coordinate::G1 as usize] = Some(Piece {
            color: Color::White,
            piece_type: PieceType::Knight,
        });
        board.pieces[Coordinate::H1 as usize] = Some(Piece {
            color: Color::White,
            piece_type: PieceType::Rook,
        });
        board.pieces[Coordinate::A2 as usize] = Some(Piece {
            color: Color::White,
            piece_type: PieceType::Pawn,
        });
        board.pieces[Coordinate::B2 as usize] = Some(Piece {
            color: Color::White,
            piece_type: PieceType::Pawn,
        });
        board.pieces[Coordinate::C2 as usize] = Some(Piece {
            color: Color::White,
            piece_type: PieceType::Pawn,
        });
        board.pieces[Coordinate::D2 as usize] = Some(Piece {
            color: Color::White,
            piece_type: PieceType::Pawn,
        });
        board.pieces[Coordinate::E2 as usize] = Some(Piece {
            color: Color::White,
            piece_type: PieceType::Pawn,
        });
        board.pieces[Coordinate::F2 as usize] = Some(Piece {
            color: Color::White,
            piece_type: PieceType::Pawn,
        });
        board.pieces[Coordinate::G2 as usize] = Some(Piece {
            color: Color::White,
            piece_type: PieceType::Pawn,
        });
        board.pieces[Coordinate::H2 as usize] = Some(Piece {
            color: Color::White,
            piece_type: PieceType::Pawn,
        });
        board.pieces[Coordinate::A8 as usize] = Some(Piece {
            color: Color::Black,
            piece_type: PieceType::Rook,
        });
        board.pieces[Coordinate::B8 as usize] = Some(Piece {
            color: Color::Black,
            piece_type: PieceType::Knight,
        });
        board.pieces[Coordinate::C8 as usize] = Some(Piece {
            color: Color::Black,
            piece_type: PieceType::Bishop,
        });
        board.pieces[Coordinate::D8 as usize] = Some(Piece {
            color: Color::Black,
            piece_type: PieceType::Queen,
        });
        board.pieces[Coordinate::E8 as usize] = Some(Piece {
            color: Color::Black,
            piece_type: PieceType::King,
        });
        board.pieces[Coordinate::F8 as usize] = Some(Piece {
            color: Color::Black,
            piece_type: PieceType::Bishop,
        });
        board.pieces[Coordinate::G8 as usize] = Some(Piece {
            color: Color::Black,
            piece_type: PieceType::Knight,
        });
        board.pieces[Coordinate::H8 as usize] = Some(Piece {
            color: Color::Black,
            piece_type: PieceType::Rook,
        });
        board.pieces[Coordinate::A7 as usize] = Some(Piece {
            color: Color::Black,
            piece_type: PieceType::Pawn,
        });
        board.pieces[Coordinate::B7 as usize] = Some(Piece {
            color: Color::Black,
            piece_type: PieceType::Pawn,
        });
        board.pieces[Coordinate::C7 as usize] = Some(Piece {
            color: Color::Black,
            piece_type: PieceType::Pawn,
        });
        board.pieces[Coordinate::D7 as usize] = Some(Piece {
            color: Color::Black,
            piece_type: PieceType::Pawn,
        });
        board.pieces[Coordinate::E7 as usize] = Some(Piece {
            color: Color::Black,
            piece_type: PieceType::Pawn,
        });
        board.pieces[Coordinate::F7 as usize] = Some(Piece {
            color: Color::Black,
            piece_type: PieceType::Pawn,
        });
        board.pieces[Coordinate::G7 as usize] = Some(Piece {
            color: Color::Black,
            piece_type: PieceType::Pawn,
        });
        board.pieces[Coordinate::H7 as usize] = Some(Piece {
            color: Color::Black,
            piece_type: PieceType::Pawn,
        });

        board
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
}

#[derive(Clone, Copy, PartialEq)]
pub enum Color {
    White,
    Black,
}

#[derive(Clone, Copy)]
pub enum PieceType {
    Pawn,
    Bishop,
    Knight,
    Rook,
    King,
    Queen,
}

#[derive(Clone, Copy)]
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
