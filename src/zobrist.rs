use rand::{Rng, SeedableRng};

use crate::board::{Color, Coordinate, PieceType};

pub type Key = u64;

// Lookup table for Zobrist hash values
pub struct ZobristTable {
    pieces: [[[Key; 64]; 6]; 2], // One for each piece type of each color for each square
    black_to_move: Key,
    castling_rights: [Key; 4],
    en_passant_file: [Key; 8],
}

impl ZobristTable {
    pub fn get_piece(&self, coordinate: Coordinate, piece_type: PieceType, color: Color) -> Key {
        self.pieces[color as usize][piece_type as usize][coordinate as usize]
    }

    pub fn get_black_to_move(&self) -> Key {
        self.black_to_move
    }

    pub fn get_castling_rights(&self, kingside: bool, color: Color) -> Key {
        self.castling_rights[color as usize + kingside as usize]
    }

    // File should be between 1 and 8
    pub fn get_en_passant_file(&self, file: usize) -> Key {
        self.en_passant_file[file - 1]
    }
}

lazy_static! {
    pub static ref ZOBRIST_TABLE: ZobristTable = {
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(42);

        let mut t = ZobristTable {
            pieces: [[[0; 64]; 6]; 2],
            black_to_move: rng.gen::<Key>(),
            castling_rights: [0; 4],
            en_passant_file: [0; 8],
        };

        for coord in 0..64 {
            for color in 0..2 {
                for piece_type in 0..6 {
                    t.pieces[color][piece_type][coord] = rng.gen::<Key>();
                }
            }
        }

        for i in 0..4 {
            t.castling_rights[i] = rng.gen::<Key>();
        }

        for i in 0..8 {
            t.en_passant_file[i] = rng.gen::<Key>();
        }

        t
    };
}
