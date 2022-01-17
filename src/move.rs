use crate::board::{Coordinate, Piece, PieceType};

#[derive(PartialEq, Debug)]
pub enum CastlingSide {
    Kingside,
    Queenside,
    Unknown,
}

#[derive(PartialEq, Debug)]
pub struct Move {
    src: Coordinate,
    dest: Coordinate,
    piece: Piece,
    is_capture: bool,
    castling_side: CastlingSide,
}

impl Move {
    pub fn new(src: Coordinate, dest: Coordinate, piece: Piece, is_capture: bool) -> Self {
        Move {
            src,
            dest,
            piece,
            is_capture,
            castling_side: CastlingSide::Unknown,
        }
    }

    pub fn new_castling(src: Coordinate, dest: Coordinate, piece: Piece, kingside: bool) -> Self {
        Move {
            src,
            dest,
            piece,
            is_capture: false,
            castling_side: if kingside {
                CastlingSide::Kingside
            } else {
                CastlingSide::Queenside
            },
        }
    }

    pub fn to_algebraic_notation(&self) -> String {
        let capture_string = if self.is_capture {
            if self.piece.piece_type == PieceType::Pawn {
                static FILE_LIST: [&str; 8] = ["a", "b", "c", "d", "e", "f", "g", "h"];
                format!("{}x", FILE_LIST[(self.src.get_file() - 1) as usize])
            } else {
                "x".to_string()
            }
        } else {
            "".to_string()
        };
        let piece_name = self.piece.piece_type.to_algebraic_notation();
        let dest_square = self.dest.to_algebraic_notation();
        format!("{}{}{}", piece_name, capture_string, dest_square)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::*;

    #[test]
    fn basic_algebraic_notation() {
        let piece = Piece {
            color: Color::White,
            piece_type: PieceType::Bishop,
        };
        let m = Move::new(Coordinate::E2, Coordinate::E4, piece, false);
        assert_eq!(m.to_algebraic_notation(), "Be4".to_string());
    }

    #[test]
    fn algebraic_notation_with_captures() {
        let piece = Piece {
            color: Color::White,
            piece_type: PieceType::Bishop,
        };
        let m = Move::new(Coordinate::E2, Coordinate::E4, piece, true);
        assert_eq!(m.to_algebraic_notation(), "Bxe4".to_string());
    }

    #[test]
    fn pawn_basic_algebraic_notation() {
        let piece = Piece {
            color: Color::White,
            piece_type: PieceType::Pawn,
        };
        let m = Move::new(Coordinate::E2, Coordinate::E4, piece, false);
        assert_eq!(m.to_algebraic_notation(), "e4".to_string());
    }

    #[test]
    fn pawn_capture_algebraic_notation() {
        let piece = Piece {
            color: Color::White,
            piece_type: PieceType::Pawn,
        };
        let m = Move::new(Coordinate::E4, Coordinate::F5, piece, true);
        assert_eq!(m.to_algebraic_notation(), "exf5".to_string());
    }
}
