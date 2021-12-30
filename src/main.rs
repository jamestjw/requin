use colored::Colorize;
use std::fmt;

#[repr(u8)]
enum Coordinate {
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

struct Board {
    pieces: [Option<Piece>; 64],
}

impl Board {
    fn new_empty() -> Board {
        Board { pieces: [None; 64] }
    }

    fn new_starting_pos() -> Board {
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

    fn print(&self) {
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
}

#[derive(Clone, Copy)]
enum Color {
    White,
    Black,
}

#[derive(Clone, Copy)]
enum PieceType {
    Pawn,
    Bishop,
    Knight,
    Rook,
    King,
    Queen,
}

#[derive(Clone, Copy)]
struct Piece {
    color: Color,
    piece_type: PieceType,
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let to_print = match self.piece_type {
            PieceType::Pawn => "P",
            PieceType::Bishop => "B",
            PieceType::Knight => "N",
            PieceType::Rook => "R",
            PieceType::King => "K",
            PieceType::Queen => "Q",
        };

        match self.color {
            Color::White => write!(f, "{}", to_print.blue()),
            Color::Black => write!(f, "{}", to_print.red()),
        }
    }
}

fn main() {
    let board = Board::new_starting_pos();
    board.print();
}
