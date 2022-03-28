use super::*;
use crate::board::*;

use lazy_static::lazy_static;
use rand::Rng;
use std::slice::Iter;
use strum::IntoEnumIterator;

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

#[derive(Copy, Clone, Debug)]
struct Magic {
    mask: Bitboard,
    magic: Bitboard,
    attacks_offset: usize, // Offset into the table for this particular magic
    shift: usize,
}

impl Magic {
    pub fn new() -> Magic {
        Magic {
            mask: 0,
            magic: 0,
            attacks_offset: 0,
            shift: 0,
        }
    }

    // Calculate the index into the table for this occupancy
    pub fn index(&self, occupancy: Bitboard) -> usize {
        ((occupancy & self.mask).wrapping_mul(self.magic) >> self.shift) as usize
    }
}

lazy_static! {
    static ref BISHOP_MAGICS: ([Magic; 64], [Bitboard; 0x1480]) = {
        let mut table = [0; 0x1480];
        let magics = init_magics(PieceType::Bishop, &mut table);
        (magics, table)
    };
    static ref ROOK_MAGICS: ([Magic; 64], [Bitboard; 0x19000]) = {
        let mut table = [0; 0x19000];
        let magics = init_magics(PieceType::Rook, &mut table);
        (magics, table)
    };
}

fn init_magics(piece_type: PieceType, table: &mut [Bitboard]) -> ([Magic; 64]) {
    // TODO: Is there a way to create an array without initialization?
    let mut magics = [Magic::new(); 64];
    let mut current_offset = 0;

    for coord in Coordinate::iter() {
        // We ignore squares on the edges when calculating piece occupancies,
        // e.g. for a rook that is on E4, we don't care whether or not the A4 square is occupied.
        let edges = ((RANK_1_BB | RANK_8_BB) & !get_rank_bb_for_coordinate(coord))
            | ((A_FILE_BB | H_FILE_BB) & !get_file_bb_for_coordinate(coord));

        let mut m = Magic::new();
        // The mask includes all the squares that a piece attacks (while ignoring
        // squares on the edges of the board). This is essentially the squares from
        // which pieces may block the attack.
        m.mask = sliding_attacks(piece_type, coord, 0) & !edges;
        // The index must be big enough to contain all possible subsets of the mask,
        // hence we identify the number of non-zero bits in the mask to calculate
        // how many bits we need to eventually shift to get the index. Note: the index
        // refers to the N most significant bits of the 64bit integer.
        m.shift = 64 - m.mask.count_ones() as usize;
        m.attacks_offset = current_offset;

        let mut size = 0;
        let mut b = 0;

        let mut temp_attacks: [Bitboard; 4096] = [0; 4096];
        let mut temp_occupancies: [Bitboard; 4096] = [0; 4096];
        loop {
            temp_occupancies[size] = b;
            temp_attacks[size] = sliding_attacks(piece_type, coord, b);

            size += 1;
            b = ((b | !m.mask).wrapping_add(1)) & m.mask;
            if b == 0 {
                break;
            }
        }

        find_magic(&mut m, &temp_attacks, &temp_occupancies, table);

        // Increase the offset by the size used by this iteration
        current_offset += size;
        magics[coord as usize] = m;
    }
    magics
}

fn random_u64() -> u64 {
    let mut rng = rand::thread_rng();

    let u1 = rng.gen::<u64>() & 0xFFFF;
    let u2 = rng.gen::<u64>() & 0xFFFF;
    let u3 = rng.gen::<u64>() & 0xFFFF;
    let u4 = rng.gen::<u64>() & 0xFFFF;
    u1 | (u2 << 16) | (u3 << 32) | (u4 << 48)
}

fn random_u64_fewbits() -> u64 {
    random_u64() & random_u64() & random_u64()
}

// Inspired by https://www.chessprogramming.org/index.php?title=Looking_for_Magics
fn find_magic(m: &mut Magic, attacks: &[Bitboard], occupancy: &[Bitboard], table: &mut [Bitboard]) {
    let n = m.mask.count_ones();
    let mut epochs = [0; 4096];
    let mut current_epoch = 0;
    loop {
        // Find a suitable magic number
        loop {
            m.magic = random_u64_fewbits();
            // Not sure what this condition is for
            if (m.mask.wrapping_mul(m.magic) >> 56).count_ones() >= 6 {
                break;
            }
        }

        current_epoch += 1;

        let mut i = 0;
        let mut fail = false;

        loop {
            let j = m.index(occupancy[i]);
            // If the entry in the table is from the previous epoch, overwrite it
            if epochs[j] < current_epoch {
                epochs[j] = current_epoch;
                table[j + m.attacks_offset] = attacks[i];
            // If the table in the entry is from the current epoch, verify that is
            // is valid
            } else if table[j + m.attacks_offset] != attacks[i] {
                fail = true;
            }

            i += 1;

            // 1 << n because there are 2^n possible subsets of the mask
            // If the current magic number failed the validation, we restart
            // the loop to find another one.
            if !(!fail && i < (1 << n)) {
                break;
            }
        }

        if !fail {
            return;
        }
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

pub fn get_sliding_attacks_by_magic(
    piece_type: PieceType,
    square: Coordinate,
    occupied: Bitboard,
) -> Bitboard {
    match piece_type {
        PieceType::Bishop => {
            let m = BISHOP_MAGICS.0[square as usize];
            let index = m.index(occupied);
            BISHOP_MAGICS.1[m.attacks_offset + index]
            // 0
        }
        PieceType::Rook => {
            let m = ROOK_MAGICS.0[square as usize];
            let index = m.index(occupied);
            ROOK_MAGICS.1[m.attacks_offset + index]
        }
        _ => panic!("Invalid piece type"),
    }
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

#[cfg(test)]
mod magic_bitboards {
    use super::*;

    #[test]
    fn magically_get_bishop_attacks_empty_board() {
        let attacks = get_sliding_attacks_by_magic(PieceType::Bishop, Coordinate::A1, 0);
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
    fn magically_get_bishop_attacks_empty_board_self_obstruction() {
        // Prove that self obstruction doesn't matter
        let occupied = get_bb_for_coordinate(Coordinate::A1);
        let attacks = get_sliding_attacks_by_magic(PieceType::Bishop, Coordinate::A1, occupied);
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
    fn magically_get_rook_attacks_empty_board() {
        let attacks = get_sliding_attacks_by_magic(PieceType::Rook, Coordinate::E4, 0);
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
    fn magically_get_bishop_attacks_with_1_piece_obstructing_diagonal() {
        let occupied = get_bb_for_coordinate(Coordinate::D4);
        let attacks = get_sliding_attacks_by_magic(PieceType::Bishop, Coordinate::A1, occupied);
        let expected_attacked_squares = [Coordinate::B2, Coordinate::C3, Coordinate::D4];

        let mut expected_bb = 0;

        for sq in expected_attacked_squares {
            expected_bb |= get_bb_for_coordinate(sq);
        }
        assert_eq!(attacks, expected_bb);
    }

    #[test]
    fn magically_get_bishop_attacks_with_2_pieces_obstructing_diagonal() {
        let occupied =
            get_bb_for_coordinate(Coordinate::D4) | get_bb_for_coordinate(Coordinate::E5);
        let attacks = get_sliding_attacks_by_magic(PieceType::Bishop, Coordinate::A1, occupied);
        let expected_attacked_squares = [Coordinate::B2, Coordinate::C3, Coordinate::D4];

        let mut expected_bb = 0;

        for sq in expected_attacked_squares {
            expected_bb |= get_bb_for_coordinate(sq);
        }
        assert_eq!(attacks, expected_bb);
    }

    #[test]
    fn magically_get_rook_attacks_with_1_piece_obstructing() {
        let occupied = get_bb_for_coordinate(Coordinate::E5)
            | get_bb_for_coordinate(Coordinate::E6)
            | get_bb_for_coordinate(Coordinate::C4);
        let attacks = get_sliding_attacks_by_magic(PieceType::Rook, Coordinate::E4, occupied);
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
