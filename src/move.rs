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
                let promotion_string = if self.is_promotion && self.promotes_to.is_some() {
                    format!("={}", self.promotes_to.unwrap().to_algebraic_notation())
                } else {
                    "".to_string()
                };
                let piece_name = self.piece.piece_type.to_algebraic_notation();
                let dest_square = self.dest.to_algebraic_notation();
                format!(
                    "{}{}{}{}",
                    piece_name, capture_string, dest_square, promotion_string
                )
            }
        }
    }

    pub fn to_long_algebraic_notation(&self) -> String {
        let promotion_string = if self.is_promotion && self.promotes_to.is_some() {
            self.promotes_to
                .unwrap()
                .to_algebraic_notation()
                .to_lowercase()
        } else {
            "".to_string()
        };
        let src_square = self.src.to_algebraic_notation();
        let dest_square = self.dest.to_algebraic_notation();
        format!("{}{}{}", src_square, dest_square, promotion_string)
    }

    pub fn is_pawn_double_advance(&self) -> bool {
        return self.piece.piece_type == PieceType::Pawn
            && self.dest.rank_difference(self.src).abs() == 2;
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
    fn pawn_capture_promotion_algebraic_notation() {
        let piece = Piece {
            color: Color::White,
            piece_type: PieceType::Pawn,
        };
        let mut m = Move::new(Coordinate::E7, Coordinate::F8, piece, true);
        m.is_promotion = true;
        m.promotes_to = Some(PieceType::Rook);
        assert_eq!(m.to_algebraic_notation(), "exf8=R".to_string());
    }

    #[test]
    fn pawn_promotion_algebraic_notation() {
        let piece = Piece {
            color: Color::White,
            piece_type: PieceType::Pawn,
        };
        let mut m = Move::new(Coordinate::E7, Coordinate::E8, piece, false);
        m.is_promotion = true;
        m.promotes_to = Some(PieceType::Queen);
        assert_eq!(m.to_algebraic_notation(), "e8=Q".to_string());
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
    fn pawn_double_advances() {
        let pawn_advances = [
            (
                Move::new(
                    Coordinate::E2,
                    Coordinate::E4,
                    Piece::new(Color::White, PieceType::Pawn),
                    false,
                ),
                true,
            ),
            (
                Move::new(
                    Coordinate::E2,
                    Coordinate::E3,
                    Piece::new(Color::White, PieceType::Pawn),
                    false,
                ),
                false,
            ),
            (
                Move::new(
                    Coordinate::E7,
                    Coordinate::E5,
                    Piece::new(Color::Black, PieceType::Pawn),
                    false,
                ),
                true,
            ),
            (
                Move::new(
                    Coordinate::E7,
                    Coordinate::E6,
                    Piece::new(Color::Black, PieceType::Pawn),
                    false,
                ),
                false,
            ),
        ];

        for (pawn_advance, is_double) in pawn_advances {
            assert_eq!(pawn_advance.is_pawn_double_advance(), is_double);
        }
    }

    #[test]
    fn simple_long_algebraic_notation() {
        let moves = [
            (
                Move::new(
                    Coordinate::E4,
                    Coordinate::F5,
                    Piece::new(Color::White, PieceType::Pawn),
                    true,
                ),
                "e4f5",
            ),
            (
                Move::new(
                    Coordinate::E7,
                    Coordinate::E6,
                    Piece::new(Color::Black, PieceType::Pawn),
                    false,
                ),
                "e7e6",
            ),
            (
                Move::new(
                    Coordinate::G1,
                    Coordinate::F3,
                    Piece::new(Color::White, PieceType::Knight),
                    false,
                ),
                "g1f3",
            ),
        ];

        for (m, notation) in moves {
            assert_eq!(m.to_long_algebraic_notation(), notation);
        }
    }

    #[test]
    fn test_castling_long_algebraic_notation() {
        let moves = [
            (Move::new_castling(Color::White, true), "e1g1"),
            (Move::new_castling(Color::Black, true), "e8g8"),
            (Move::new_castling(Color::White, false), "e1c1"),
            (Move::new_castling(Color::Black, false), "e8c8"),
        ];

        for (m, notation) in moves {
            assert_eq!(m.to_long_algebraic_notation(), notation);
        }
    }

    #[test]
    fn test_promotion_long_algebraic_notation() {
        let mut m = Move::new(
            Coordinate::E7,
            Coordinate::F8,
            Piece::new(Color::White, PieceType::Pawn),
            true,
        );
        m.is_promotion = true;
        m.promotes_to = Some(PieceType::Rook);

        assert_eq!(m.to_long_algebraic_notation(), "e7f8r");
    }
}
