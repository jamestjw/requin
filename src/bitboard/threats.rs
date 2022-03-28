use super::*;
use crate::board::*;

use std::slice::Iter;

#[repr(i8)]
#[derive(Clone, Copy)]
pub enum Direction {
    N = 8,
    S = -8,
    E = 1,
    W = -1,
    NE = 8 + 1,
    NW = 8 - 1,
    SW = -8 - 1,
    SE = -8 + 1,
}

impl Direction {
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

pub fn sliding_attacks(piece_type: PieceType, square: Coordinate, occupied: Bitboard) -> Bitboard {
    let mut bitboard = 0;
    let directions = if piece_type == PieceType::Bishop {
        Direction::diagonal_iterator()
    } else {
        Direction::horizontal_vertical_iterator()
    };

    for direction in directions {
        let mut s = square;
        // Check if coordinate + offset is a valid square
        // and check if the current square is occupied
        while coordinate_offset(s, *direction as i8) != 0 && !get_bitboard(occupied, s as usize) {
            s = s.offset(*direction as i8);
            bitboard |= get_bb_for_coordinate(s);
        }
    }

    bitboard
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bishop_attacks_empty_board() {
        let attacks = sliding_attacks(PieceType::Bishop, Coordinate::A1, 0);
        let expected_attacked_squares = [
            Coordinate::B2,
            Coordinate::C3,
            Coordinate::D4,
            Coordinate::E5,
            Coordinate::F6,
            Coordinate::G7,
            Coordinate::H8,
        ];

        let mut expected_bb = 0;

        for sq in expected_attacked_squares {
            expected_bb |= get_bb_for_coordinate(sq);
        }
        assert_eq!(attacks, expected_bb);
    }

    #[test]
    fn rook_attacks_empty_board() {
        let attacks = sliding_attacks(PieceType::Rook, Coordinate::E4, 0);
        let expected_attacked_squares = [
            // Down
            Coordinate::E1,
            Coordinate::E2,
            Coordinate::E3,
            // Up
            Coordinate::E5,
            Coordinate::E6,
            Coordinate::E7,
            Coordinate::E8,
            // Left
            Coordinate::A4,
            Coordinate::B4,
            Coordinate::C4,
            Coordinate::D4,
            // Right
            Coordinate::F4,
            Coordinate::G4,
            Coordinate::H4,
        ];

        let mut expected_bb = 0;

        for sq in expected_attacked_squares {
            expected_bb |= get_bb_for_coordinate(sq);
        }
        assert_eq!(attacks, expected_bb);
    }

    #[test]
    fn bishop_attacks_with_1_piece_obstructing_diagonal() {
        let occupied = get_bb_for_coordinate(Coordinate::D4);
        let attacks = sliding_attacks(PieceType::Bishop, Coordinate::A1, occupied);
        let expected_attacked_squares = [Coordinate::B2, Coordinate::C3, Coordinate::D4];

        let mut expected_bb = 0;

        for sq in expected_attacked_squares {
            expected_bb |= get_bb_for_coordinate(sq);
        }
        assert_eq!(attacks, expected_bb);
    }

    #[test]
    fn bishop_attacks_with_2_pieces_obstructing_diagonal() {
        let occupied =
            get_bb_for_coordinate(Coordinate::D4) | get_bb_for_coordinate(Coordinate::E5);
        let attacks = sliding_attacks(PieceType::Bishop, Coordinate::A1, occupied);
        let expected_attacked_squares = [Coordinate::B2, Coordinate::C3, Coordinate::D4];

        let mut expected_bb = 0;

        for sq in expected_attacked_squares {
            expected_bb |= get_bb_for_coordinate(sq);
        }
        assert_eq!(attacks, expected_bb);
    }

    #[test]
    fn rook_attacks_with_1_piece_obstructing() {
        let occupied = get_bb_for_coordinate(Coordinate::E5)
            | get_bb_for_coordinate(Coordinate::E6)
            | get_bb_for_coordinate(Coordinate::C4);
        let attacks = sliding_attacks(PieceType::Rook, Coordinate::E4, occupied);
        let expected_attacked_squares = [
            // Down
            Coordinate::E1,
            Coordinate::E2,
            Coordinate::E3,
            // Up
            Coordinate::E5,
            // Left
            Coordinate::C4,
            Coordinate::D4,
            // Right
            Coordinate::F4,
            Coordinate::G4,
            Coordinate::H4,
        ];
        let mut expected_bb = 0;

        for sq in expected_attacked_squares {
            expected_bb |= get_bb_for_coordinate(sq);
        }
        assert_eq!(attacks, expected_bb);
    }
}
