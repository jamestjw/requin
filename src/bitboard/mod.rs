mod threats;

use crate::board::*;

use lazy_static::lazy_static;
use std::convert::TryFrom;

type Bitboard = u64;

lazy_static! {
    static ref COORDINATE_BBS: [Bitboard; 64] = {
        let mut bbs = [0; 64];
        for i in 0..64 {
            bbs[i] = 1 << i;
        }
        bbs
    };
}

fn get_bitboard(bitboard: Bitboard, idx: usize) -> bool {
    (bitboard & (1 << idx)) != 0
}

fn set_bitboard(bitboard: Bitboard, idx: usize) -> Bitboard {
    bitboard | (1 << idx)
}

fn unset_bitboard(bitboard: Bitboard, idx: usize) -> Bitboard {
    bitboard & !(1 << idx)
}

fn get_bb_for_coordinate(coord: Coordinate) -> Bitboard {
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
}
