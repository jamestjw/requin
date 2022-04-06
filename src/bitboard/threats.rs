use super::*;
use crate::board::*;

use lazy_static::lazy_static;
use rand::Rng;
use std::boxed::Box;
use strum::IntoEnumIterator;

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
    static ref BISHOP_MAGICS: ([Magic; 64], Box<[u64]>) = {
        let table: Vec<Bitboard> = vec![0; 0x1480];
        let mut table = table.into_boxed_slice();
        let magics = init_magics(PieceType::Bishop, &mut table);
        (magics, table)
    };
    static ref ROOK_MAGICS: ([Magic; 64], Box<[u64]>) = {
        let table: Vec<Bitboard> = vec![0; 0x19000];
        let mut table = table.into_boxed_slice();
        let magics = init_magics(PieceType::Rook, &mut table);
        (magics, table)
    };
    static ref PATH_BETWEEN_SQUARES_BB: [[Bitboard; 64]; 64] = {
        let mut table = [[0; 64]; 64];
        for src in Coordinate::iter() {
            for dest in Coordinate::iter() {
                // Squares are on the same rank or
                if src.get_rank() == dest.get_rank() || src.get_file() == dest.get_file() {
                    // Make the other square occupied so that the path doesn't
                    // go beyond it
                    let b = get_sliding_attacks_occupied(PieceType::Rook, src, dest.to_bb()) & get_sliding_attacks_occupied(PieceType::Rook, dest, src.to_bb());
                    table[src as usize][dest as usize] |= b;
                } else {
                    let b = get_sliding_attacks_occupied(PieceType::Bishop, src, dest.to_bb()) & get_sliding_attacks_occupied(PieceType::Bishop, dest, src.to_bb());
                    table[src as usize][dest as usize] |= b;
                }

                table[src as usize][dest as usize] |= dest.to_bb();
            }
        }
        table
    };
    static ref EDGE_TO_EDGE_BB: [[Bitboard; 64]; 64] = {
        let mut table = [[0; 64]; 64];
        for src in Coordinate::iter() {
            for dest in Coordinate::iter() {
                // Squares are on the same rank or
                if src.get_rank() == dest.get_rank() || src.get_file() == dest.get_file() {
                    // The entire line between the two squares and the two squares themselves
                    let b = (get_sliding_attacks(PieceType::Rook, src) & get_sliding_attacks(PieceType::Rook, dest)) | src.to_bb() | dest.to_bb();
                    table[src as usize][dest as usize] |= b;
                } else {
                    let b = (get_sliding_attacks(PieceType::Bishop, src) & get_sliding_attacks(PieceType::Bishop, dest) )| src.to_bb() | dest.to_bb();
                    table[src as usize][dest as usize] |= b;
                }
            }
        }
        table
    };
    static ref PAWN_ATTACKS_BB: [[Bitboard; 64]; 2] = {
        let mut table = [[0; 64]; 2];
        for color in [Color::White, Color::Black] {
            let direction = match color {
                Color::White => Direction::N,
                Color::Black => Direction::S,
            };
            for coord in Coordinate::iter() {
                let forward_sq = coordinate_offset(coord, direction as i8);
                if forward_sq != 0 {
                    let forward_sq_coord = Coordinate::from_bb(forward_sq);
                    table[color as usize][coord as usize] |= coordinate_offset(forward_sq_coord, Direction::E as i8) |
                                                                coordinate_offset(forward_sq_coord, Direction::W as i8);
                }
            }
        }
        table
    };
    // Squares that are attacked by pieces from particular squares while disregarding all obstacles.
    // NOTE: Only knight and king moves are populated.
    static ref PIECE_TYPE_ATTACKS: [[Bitboard; 64]; 6] = {
        let mut table = [[0; 64]; 6];

        for (i, coord) in Coordinate::iter().enumerate() {
            // Populate king moves
            for direction in Direction::iter() {
                table[PieceType::King as usize][i] |= coordinate_offset(coord, direction as i8);
            }

            // Populate knight moves
            let rank = coord.get_rank();
            let file = coord.get_file();

            if rank < 7 && file < 8 {
                //   ___>
                //   |
                //   |
                table[PieceType::Knight as usize][i] |= Coordinate::try_from(i + 17).unwrap().to_bb();
            }

            if rank < 8 && file < 7 {
                //   ____>
                //   |
                table[PieceType::Knight as usize][i] |= Coordinate::try_from(i + 10).unwrap().to_bb();

            }

            if rank < 7 && file > 1 {
                //  <___
                //     |
                //     |
                table[PieceType::Knight as usize][i] |= Coordinate::try_from(i + 15).unwrap().to_bb();

            }

            if rank < 8 && file > 2 {
                //    <|
                //     |______
                table[PieceType::Knight as usize][i] |= Coordinate::try_from(i + 6).unwrap().to_bb();

            }

            if rank > 2 && file < 8 {
                //     |
                //     |
                //  <__|
                table[PieceType::Knight as usize][i] |= Coordinate::try_from(i - 15).unwrap().to_bb();

            }

            if rank > 1 && file < 7 {
                //  |
                //  |____>
                table[PieceType::Knight as usize][i] |= Coordinate::try_from(i - 6).unwrap().to_bb();

            }

            if rank > 2 && file > 1 {
                //    |
                //    |
                // <__|
                table[PieceType::Knight as usize][i] |= Coordinate::try_from(i - 17).unwrap().to_bb();

            }

            if rank > 1 && file > 2 {
                //      |
                // <____|
                table[PieceType::Knight as usize][i] |= Coordinate::try_from(i - 10).unwrap().to_bb();

            }
        }
        table
    };
}

fn init_magics(piece_type: PieceType, table: &mut Box<[u64]>) -> ([Magic; 64]) {
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
fn find_magic(m: &mut Magic, attacks: &[Bitboard], occupancy: &[Bitboard], table: &mut Box<[u64]>) {
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

fn sliding_attacks(piece_type: PieceType, square: Coordinate, occupied: Bitboard) -> Bitboard {
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

pub fn get_sliding_attacks_occupied(
    piece_type: PieceType,
    square: Coordinate,
    occupied: Bitboard,
) -> Bitboard {
    match piece_type {
        PieceType::Bishop => {
            let m = BISHOP_MAGICS.0[square as usize];
            let index = m.index(occupied);
            BISHOP_MAGICS.1[m.attacks_offset + index]
        }
        PieceType::Rook => {
            let m = ROOK_MAGICS.0[square as usize];
            let index = m.index(occupied);
            ROOK_MAGICS.1[m.attacks_offset + index]
        }
        PieceType::Queen => {
            get_sliding_attacks_occupied(PieceType::Bishop, square, occupied)
                | get_sliding_attacks_occupied(PieceType::Rook, square, occupied)
        }
        _ => panic!("Invalid piece type"),
    }
}

pub fn get_sliding_attacks(piece_type: PieceType, square: Coordinate) -> Bitboard {
    match piece_type {
        PieceType::Bishop => {
            let m = BISHOP_MAGICS.0[square as usize];
            BISHOP_MAGICS.1[m.attacks_offset]
        }
        PieceType::Rook => {
            let m = ROOK_MAGICS.0[square as usize];
            ROOK_MAGICS.1[m.attacks_offset]
        }
        _ => panic!("Invalid piece type"),
    }
}

// Returns a bitboard to point to squares that contain pieces that are blocking
// an attack on the target square, i.e. this implies that if any of these pieces
// move, the target square would directly be under attack.
// Candidates refers to squares that contain pieces for the attacking side,
// these squares contain potential blockers (so that we have the flexibility
// of finding pinned pieces OR discovered checkers) and also the attacking
// pieces.
// Target refers to the target square
pub fn sliding_attack_blockers(
    horizontal_sliders: Bitboard,
    diagonal_sliders: Bitboard,
    all_pieces: Bitboard,
    candidates: Bitboard,
    target: Coordinate,
) -> Bitboard {
    let mut blockers = 0;
    // Filter the pieces that attack the target square, either directly
    // or via an x-ray.
    let mut attackers = ((get_sliding_attacks(PieceType::Rook, target) & horizontal_sliders)
        | (get_sliding_attacks(PieceType::Bishop, target) & diagonal_sliders))
        & candidates;
    // The ones that aren't attacking are potential blockers
    let potential_blockers = all_pieces ^ attackers;

    while attackers != 0 {
        let (attacker_sq, popped_attackers) = pop_lsb(attackers);
        attackers = popped_attackers;
        let blockers_for_attacker =
            path_between(target, Coordinate::from_bb(attacker_sq)) & potential_blockers;

        // Check if there is exactly one blocker
        if blockers_for_attacker != 0 && !more_than_one(blockers_for_attacker) {
            blockers |= blockers_for_attacker;
        }
    }

    blockers
}

// Returns a Bitboard containing a path between two squares, this does not include
// the source but includes the destination. If there is no direct path, i.e. no
// straight line between the two squares, this contains the destination only
pub fn path_between(src: Coordinate, dest: Coordinate) -> Bitboard {
    PATH_BETWEEN_SQUARES_BB[src as usize][dest as usize]
}

// Returns a Bitboard containing a path from one edge of the board to the other
// while intersecting both src and dest squares. This only works with horizontal,
// vertical and diagonal paths. For two squares that do not get intersected by
// any straight line, this function returns a Bitboard containing both src
// and dest coordinates only.
pub fn edge_to_edge_bb(src: Coordinate, dest: Coordinate) -> Bitboard {
    EDGE_TO_EDGE_BB[src as usize][dest as usize]
}

pub fn get_pawn_attacks_bb(color: Color, src: Coordinate) -> Bitboard {
    PAWN_ATTACKS_BB[color as usize][src as usize]
}

// Currently only implemented for knights and kings
pub fn get_piece_attacks_bb(piece_type: PieceType, src: Coordinate) -> Bitboard {
    PIECE_TYPE_ATTACKS[piece_type as usize][src as usize]
}

pub fn get_pawn_attacks_from_bb(mut pawns: Bitboard, color: Color) -> Bitboard {
    let mut attacks = 0;
    while pawns != 0 {
        let (pawn, popped_pawns) = pop_lsb(pawns);
        pawns = popped_pawns;
        attacks |= get_pawn_attacks_bb(color, Coordinate::from_bb(pawn));
    }

    attacks
}

pub fn init_tables() {
    lazy_static::initialize(&BISHOP_MAGICS);
    lazy_static::initialize(&ROOK_MAGICS);
    lazy_static::initialize(&PATH_BETWEEN_SQUARES_BB);
    lazy_static::initialize(&EDGE_TO_EDGE_BB);
    lazy_static::initialize(&PAWN_ATTACKS_BB);
    lazy_static::initialize(&PIECE_TYPE_ATTACKS);
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
        let attacks = get_sliding_attacks_occupied(PieceType::Bishop, Coordinate::A1, 0);
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
        let attacks = get_sliding_attacks_occupied(PieceType::Bishop, Coordinate::A1, occupied);
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
        let attacks = get_sliding_attacks_occupied(PieceType::Rook, Coordinate::E4, 0);
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
        let attacks = get_sliding_attacks_occupied(PieceType::Bishop, Coordinate::A1, occupied);
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
        let attacks = get_sliding_attacks_occupied(PieceType::Bishop, Coordinate::A1, occupied);
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
        let attacks = get_sliding_attacks_occupied(PieceType::Rook, Coordinate::E4, occupied);
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

    #[test]
    fn get_sliding_attacks_nothing_occupied() {
        for coord in Coordinate::iter() {
            assert_eq!(
                get_sliding_attacks(PieceType::Bishop, coord,),
                get_sliding_attacks_occupied(PieceType::Bishop, coord, 0)
            );
            assert_eq!(
                get_sliding_attacks(PieceType::Rook, coord,),
                get_sliding_attacks_occupied(PieceType::Rook, coord, 0)
            );
        }
    }

    #[test]
    fn horizontal_path_between_two_squares() {
        let path_bb = path_between(Coordinate::A2, Coordinate::G2);
        let expected_squares = [
            Coordinate::B2,
            Coordinate::C2,
            Coordinate::D2,
            Coordinate::E2,
            Coordinate::F2,
            Coordinate::G2,
        ];
        let mut expected_bb = 0;

        for sq in expected_squares {
            expected_bb |= get_bb_for_coordinate(sq);
        }
        assert_eq!(path_bb, expected_bb);
    }

    #[test]
    fn vertical_path_between_two_squares() {
        let path_bb = path_between(Coordinate::D3, Coordinate::D7);
        let expected_squares = [
            Coordinate::D4,
            Coordinate::D5,
            Coordinate::D6,
            Coordinate::D7,
        ];
        let mut expected_bb = 0;

        for sq in expected_squares {
            expected_bb |= get_bb_for_coordinate(sq);
        }
        assert_eq!(path_bb, expected_bb);
    }

    #[test]
    fn diagonal_path_between_two_squares() {
        let path_bb = path_between(Coordinate::G3, Coordinate::D6);
        let expected_squares = [Coordinate::F4, Coordinate::E5, Coordinate::D6];
        let mut expected_bb = 0;

        for sq in expected_squares {
            expected_bb |= get_bb_for_coordinate(sq);
        }
        assert_eq!(path_bb, expected_bb);
    }

    #[test]
    fn sliding_attack_bishop_blocker() {
        let target_square = Coordinate::E4;
        let horizontal_sliders = 0;
        let diagonal_sliders = Coordinate::B7.to_bb();
        let blockers = Coordinate::C6.to_bb();
        let all_pieces = diagonal_sliders | horizontal_sliders | blockers | target_square.to_bb();
        let candidates = all_pieces;

        assert_eq!(
            sliding_attack_blockers(
                horizontal_sliders,
                diagonal_sliders,
                all_pieces,
                candidates,
                target_square
            ),
            blockers
        );
    }

    #[test]
    fn sliding_attack_bishop_double_blockers() {
        let target_square = Coordinate::E4;
        let horizontal_sliders = 0;
        let diagonal_sliders = Coordinate::B7.to_bb();
        let blockers = Coordinate::C6.to_bb() | Coordinate::D5.to_bb();
        let all_pieces = diagonal_sliders | horizontal_sliders | blockers | target_square.to_bb();
        let candidates = all_pieces;

        // It doesn't count if two pieces are blocking
        assert_eq!(
            sliding_attack_blockers(
                horizontal_sliders,
                diagonal_sliders,
                all_pieces,
                candidates,
                target_square
            ),
            0
        );
    }

    #[test]
    fn sliding_attack_bishop_no_blockers() {
        let target_square = Coordinate::E4;
        let horizontal_sliders = 0;
        let diagonal_sliders = Coordinate::B7.to_bb();
        let blockers = 0;
        let all_pieces = diagonal_sliders | horizontal_sliders | blockers | target_square.to_bb();
        let candidates = all_pieces;

        // It doesn't count if it is a direct attack
        assert_eq!(
            sliding_attack_blockers(
                horizontal_sliders,
                diagonal_sliders,
                all_pieces,
                candidates,
                target_square
            ),
            0
        );
    }

    #[test]
    fn sliding_attack_rook_blocker() {
        let target_square = Coordinate::E4;
        let horizontal_sliders = Coordinate::H4.to_bb();
        let diagonal_sliders = 0;
        let blockers = Coordinate::F4.to_bb();
        let all_pieces = diagonal_sliders | horizontal_sliders | blockers | target_square.to_bb();
        let candidates = all_pieces;

        assert_eq!(
            sliding_attack_blockers(
                horizontal_sliders,
                diagonal_sliders,
                all_pieces,
                candidates,
                target_square
            ),
            blockers
        );
    }

    #[test]
    fn sliding_attack_rook_double_blockers() {
        let target_square = Coordinate::E4;
        let horizontal_sliders = Coordinate::H4.to_bb();
        let diagonal_sliders = 0;
        let blockers = Coordinate::G4.to_bb() | Coordinate::F4.to_bb();
        let all_pieces = diagonal_sliders | horizontal_sliders | blockers | target_square.to_bb();
        let candidates = all_pieces;

        // Doesn't count when two pieces are blocking
        assert_eq!(
            sliding_attack_blockers(
                horizontal_sliders,
                diagonal_sliders,
                all_pieces,
                candidates,
                target_square
            ),
            0
        );
    }

    #[test]
    fn sliding_attack_rook_no_blockers() {
        let target_square = Coordinate::E4;
        let horizontal_sliders = Coordinate::H4.to_bb();
        let diagonal_sliders = 0;
        let blockers = 0;
        let all_pieces = diagonal_sliders | horizontal_sliders | blockers | target_square.to_bb();
        let candidates = all_pieces;

        // Doesn't count when it is a direct attack
        assert_eq!(
            sliding_attack_blockers(
                horizontal_sliders,
                diagonal_sliders,
                all_pieces,
                candidates,
                target_square
            ),
            0
        );
    }

    #[test]
    fn edge_to_edge_horizontal() {
        let bb = edge_to_edge_bb(Coordinate::A2, Coordinate::G2);
        let expected_squares = [
            Coordinate::A2,
            Coordinate::B2,
            Coordinate::C2,
            Coordinate::D2,
            Coordinate::E2,
            Coordinate::F2,
            Coordinate::G2,
            Coordinate::H2,
        ];
        let mut expected_bb = 0;

        for sq in expected_squares {
            expected_bb |= get_bb_for_coordinate(sq);
        }
        assert_eq!(bb, expected_bb);
    }

    #[test]
    fn edge_to_edge_vertical() {
        let bb = edge_to_edge_bb(Coordinate::E4, Coordinate::E8);
        let expected_squares = [
            Coordinate::E1,
            Coordinate::E2,
            Coordinate::E3,
            Coordinate::E4,
            Coordinate::E5,
            Coordinate::E6,
            Coordinate::E7,
            Coordinate::E8,
        ];
        let mut expected_bb = 0;

        for sq in expected_squares {
            expected_bb |= get_bb_for_coordinate(sq);
        }
        assert_eq!(bb, expected_bb);
    }

    #[test]
    fn edge_to_edge_diagonal() {
        let bb = edge_to_edge_bb(Coordinate::B2, Coordinate::G7);
        let expected_squares = [
            Coordinate::A1,
            Coordinate::B2,
            Coordinate::C3,
            Coordinate::D4,
            Coordinate::E5,
            Coordinate::F6,
            Coordinate::G7,
            Coordinate::H8,
        ];
        let mut expected_bb = 0;

        for sq in expected_squares {
            expected_bb |= get_bb_for_coordinate(sq);
        }
        assert_eq!(bb, expected_bb);
    }

    #[test]
    fn edge_to_edge_without_reasonable_path() {
        let bb = edge_to_edge_bb(Coordinate::B2, Coordinate::C4);
        let expected_squares = [Coordinate::B2, Coordinate::C4];
        let mut expected_bb = 0;

        for sq in expected_squares {
            expected_bb |= get_bb_for_coordinate(sq);
        }
        assert_eq!(bb, expected_bb);
    }

    #[test]
    fn pawn_attacks_bb() {
        assert_eq!(
            get_pawn_attacks_bb(Color::White, Coordinate::E4),
            Coordinate::D5.to_bb() | Coordinate::F5.to_bb()
        );
        assert_eq!(
            get_pawn_attacks_bb(Color::White, Coordinate::H7),
            Coordinate::G8.to_bb()
        );
        assert_eq!(get_pawn_attacks_bb(Color::White, Coordinate::D8), 0);
        assert_eq!(
            get_pawn_attacks_bb(Color::Black, Coordinate::E4),
            Coordinate::D3.to_bb() | Coordinate::F3.to_bb()
        );
        assert_eq!(
            get_pawn_attacks_bb(Color::Black, Coordinate::A2),
            Coordinate::B1.to_bb()
        );
        assert_eq!(get_pawn_attacks_bb(Color::Black, Coordinate::C1), 0);
    }
}
