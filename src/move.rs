use crate::board::{Color, Coordinate, Piece, PieceType, FILE_LIST};

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum CastlingSide {
    Kingside,
    Queenside,
    Unknown,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Move {
    pub src: Coordinate,
    pub dest: Coordinate,
    pub piece: Piece,
    pub is_capture: bool,
    pub is_en_passant: bool,
    pub castling_side: CastlingSide,
    pub promotes_to: Option<PieceType>,
    pub is_promotion: bool,
}

impl Move {
    pub fn new(src: Coordinate, dest: Coordinate, piece: Piece, is_capture: bool) -> Self {
        Move {
            src,
            dest,
            piece,
            is_capture,
            castling_side: CastlingSide::Unknown,
            is_en_passant: false,
            promotes_to: None,
            is_promotion: false,
        }
    }

    pub fn new_castling(color: Color, kingside: bool) -> Self {
        let king_src = match color {
            Color::White => Coordinate::E1,
            Color::Black => Coordinate::E8,
        };

        let king_dest = match (color, kingside) {
            (Color::White, true) => Coordinate::G1,
            (Color::White, false) => Coordinate::C1,
            (Color::Black, true) => Coordinate::G8,
            (Color::Black, false) => Coordinate::C8,
        };

        let king = Piece {
            color,
            piece_type: PieceType::King,
        };

        Move {
            src: king_src,
            dest: king_dest,
            is_capture: false,
            piece: king,
            castling_side: if kingside {
                CastlingSide::Kingside
            } else {
                CastlingSide::Queenside
            },
            is_en_passant: false,
            is_promotion: false,
            promotes_to: None,
        }
    }

    pub fn eligible_for_en_passant(&self) -> bool {
        // .abs() only works on signed integers
        self.piece.piece_type == PieceType::Pawn && (self.src as i8 - self.dest as i8).abs() == 16
    }

    pub fn to_algebraic_notation(&self) -> String {
        match self.castling_side {
            CastlingSide::Kingside => "O-O".to_string(),
            CastlingSide::Queenside => "O-O-O".to_string(),
            _ => {
                let capture_string = if self.is_capture {
                    if self.piece.piece_type == PieceType::Pawn {
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

    #[test]
    fn kingside_castling_algebraic_notation() {
        let m = Move::new_castling(Color::White, true);
        assert_eq!(m.to_algebraic_notation(), "O-O".to_string());
    }

    #[test]
    fn queenside_castling_algebraic_notation() {
        let m = Move::new_castling(Color::White, false);
        assert_eq!(m.to_algebraic_notation(), "O-O-O".to_string());
    }

    #[test]
    fn en_passant_eligibility_white() {
        let piece = Piece {
            color: Color::White,
            piece_type: PieceType::Pawn,
        };
        let m = Move::new(Coordinate::E2, Coordinate::E4, piece, false);

        assert!(m.eligible_for_en_passant());
    }

    #[test]
    fn en_passant_eligibility_black() {
        let piece = Piece {
            color: Color::Black,
            piece_type: PieceType::Pawn,
        };
        let m = Move::new(Coordinate::E7, Coordinate::E5, piece, false);

        assert!(m.eligible_for_en_passant());
    }

    #[test]
    fn en_passant_ineligibility_white() {
        let piece = Piece {
            color: Color::White,
            piece_type: PieceType::Pawn,
        };
        let m = Move::new(Coordinate::E5, Coordinate::E6, piece, false);

        assert!(!m.eligible_for_en_passant());
    }

    #[test]
    fn en_passant_ineligibility_black() {
        let piece = Piece {
            color: Color::Black,
            piece_type: PieceType::Pawn,
        };
        let m = Move::new(Coordinate::E7, Coordinate::E6, piece, false);

        assert!(!m.eligible_for_en_passant());
    }
}
