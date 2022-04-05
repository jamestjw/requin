#![allow(dead_code)]

mod threats;

use crate::board::*;

use lazy_static::lazy_static;
use std::convert::TryFrom;
use std::slice::Iter;

pub use threats::{
    edge_to_edge_bb, get_pawn_attacks_bb, get_pawn_attacks_from_bb, get_piece_attacks_bb,
    get_sliding_attacks_occupied, init_tables, path_between, sliding_attack_blockers,
};

pub type Bitboard = u64;

lazy_static! {
    static ref COORDINATE_BBS: [Bitboard; 64] = {
        let mut bbs = [0; 64];
        for i in 0..64 {
            bbs[i] = 1 << i;
        }
        bbs
    };
}

pub static RANK_1_BB: Bitboard = 0b11111111;
pub static RANK_2_BB: Bitboard = RANK_1_BB << (8 * 1);
pub static RANK_3_BB: Bitboard = RANK_1_BB << (8 * 2);
pub static RANK_4_BB: Bitboard = RANK_1_BB << (8 * 3);
pub static RANK_5_BB: Bitboard = RANK_1_BB << (8 * 4);
pub static RANK_6_BB: Bitboard = RANK_1_BB << (8 * 5);
pub static RANK_7_BB: Bitboard = RANK_1_BB << (8 * 6);
pub static RANK_8_BB: Bitboard = RANK_1_BB << (8 * 7);

pub static A_FILE_BB: Bitboard = 0x0101010101010101;
pub static B_FILE_BB: Bitboard = A_FILE_BB << 1;
pub static C_FILE_BB: Bitboard = A_FILE_BB << 2;
pub static D_FILE_BB: Bitboard = A_FILE_BB << 3;
pub static E_FILE_BB: Bitboard = A_FILE_BB << 4;
pub static F_FILE_BB: Bitboard = A_FILE_BB << 5;
pub static G_FILE_BB: Bitboard = A_FILE_BB << 6;
pub static H_FILE_BB: Bitboard = A_FILE_BB << 7;

pub fn get_bitboard(bitboard: Bitboard, idx: usize) -> bool {
    (bitboard & (1 << idx)) != 0
}

pub fn set_bitboard(bitboard: Bitboard, idx: usize) -> Bitboard {
    bitboard | (1 << idx)
}

pub fn unset_bitboard(bitboard: Bitboard, idx: usize) -> Bitboard {
    bitboard & !(1 << idx)
}

pub fn get_bb_for_coordinate(coord: Coordinate) -> Bitboard {
    COORDINATE_BBS[coord as usize]
}

// Returns an empty bitboard for invalid offsets
fn coordinate_offset(coord: Coordinate, offset: i8) -> Bitboard {
    let dest = coord as i8 + offset;
    if Coordinate::is_valid(dest) {
        let dest_coord = Coordinate::try_from(dest as usize).unwrap();
        // Check if the offset is reasonable
        if dest_coord.rank_difference(coord).abs() <= 1
            && dest_coord.file_difference(coord).abs() <= 1
        {
            COORDINATE_BBS[dest as usize]
        } else {
            // Invalid offset
            0
        }
    } else {
        0
    }
}

// Get bitboard that corresponds to the rank of this coordinate
fn get_rank_bb_for_coordinate(coord: Coordinate) -> Bitboard {
    RANK_1_BB << (8 * (coord.get_rank() - 1))
}

// Get bitboard that corresponds to the file of this coordinate
fn get_file_bb_for_coordinate(coord: Coordinate) -> Bitboard {
    A_FILE_BB << (coord.get_file() - 1)
}

// TODO: Investigate the usage of de Bruijn sequences to do this
// https://stackoverflow.com/questions/757059/position-of-least-significant-bit-that-is-set
pub fn lsb(b: Bitboard) -> Bitboard {
    b & (1u64.wrapping_shl(b.trailing_zeros()))
}

// Returns the LSB and the popped version of the Bitboard
pub fn pop_lsb(b: Bitboard) -> (Bitboard, Bitboard) {
    let lsb = lsb(b);
    (lsb, b & !lsb)
}

pub fn more_than_one(b: Bitboard) -> bool {
    // If popping the LSB results in something that is non-zero,
    // we can immediately tell that there is more than one bit that
    // is set
    pop_lsb(b).1 != 0
}

use strum_macros::EnumIter;

#[repr(i8)]
#[derive(Clone, Copy, EnumIter)]
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

#[cfg(test)]
mod bitboard_tests {
    use super::*;

    #[test]
    fn bitboard_set() {
        let mut bitboard = 0;
        assert_eq!(get_bitboard(bitboard, 5), false);
        bitboard = set_bitboard(bitboard, 5);
        assert_eq!(get_bitboard(bitboard, 5), true);
        bitboard = unset_bitboard(bitboard, 5);
        assert_eq!(get_bitboard(bitboard, 5), false);
    }

    #[test]
    fn coordinate_bbs() {
        assert_eq!(get_bb_for_coordinate(Coordinate::A1), 0b1);
        assert_eq!(get_bb_for_coordinate(Coordinate::B1), 0b10);
        assert_eq!(get_bb_for_coordinate(Coordinate::C1), 0b100);
        assert_eq!(get_bb_for_coordinate(Coordinate::A2), 0b100000000);
        assert_eq!(get_bb_for_coordinate(Coordinate::B2), 0b1000000000);
        assert_eq!(get_bb_for_coordinate(Coordinate::C2), 0b10000000000);
    }

    #[test]
    fn file_bb() {
        let file_bb = get_file_bb_for_coordinate(Coordinate::G6);
        let expected_squares = [
            Coordinate::G1,
            Coordinate::G2,
            Coordinate::G3,
            Coordinate::G4,
            Coordinate::G5,
            Coordinate::G6,
            Coordinate::G7,
            Coordinate::G8,
        ];

        let mut expected_bb = 0;

        for sq in expected_squares {
            expected_bb |= get_bb_for_coordinate(sq);
        }
        assert_eq!(file_bb, expected_bb);
    }

    #[test]
    fn rank_bb() {
        let rank_bb = get_rank_bb_for_coordinate(Coordinate::D5);
        let expected_squares = [
            Coordinate::A5,
            Coordinate::B5,
            Coordinate::C5,
            Coordinate::D5,
            Coordinate::E5,
            Coordinate::F5,
            Coordinate::G5,
            Coordinate::H5,
        ];

        let mut expected_bb = 0;

        for sq in expected_squares {
            expected_bb |= get_bb_for_coordinate(sq);
        }
        assert_eq!(rank_bb, expected_bb);
    }

    #[test]
    fn test_lsb() {
        let b = 0b1010101100;
        assert_eq!(lsb(b), 0b100);
    }

    #[test]
    fn test_pop_lsb() {
        let b = 0b1010101100;
        assert_eq!(pop_lsb(b), (0b100, 0b1010101000));
    }

    #[test]
    fn test_more_than_one() {
        assert!(more_than_one(0b11));
        assert!(more_than_one(0b110));
        assert!(!more_than_one(0b0));
        assert!(!more_than_one(0b1));
        assert!(!more_than_one(0b10));
    }
}
