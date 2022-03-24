use crate::board::{Board, Color, Coordinate, PieceType};
use lazy_static::lazy_static;

// First row is the first rank, first column is the H file
static WHITE_PAWN_POSITIONAL_VALUE: [[i32; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [5, 10, 10, -20, -20, 10, 10, 5],
    [5, -5, -10, 0, 0, -10, -5, 5],
    [0, 0, 0, 20, 20, 0, 0, 0],
    [5, 5, 10, 25, 25, 10, 5, 5],
    [10, 10, 20, 30, 30, 20, 10, 10],
    [50, 50, 50, 50, 50, 50, 50, 50],
    [0, 0, 0, 0, 0, 0, 0, 0],
];

static WHITE_ROOK_POSITIONAL_VALUE: [[i32; 8]; 8] = [
    [0, 0, 0, 5, 5, 0, 0, 0],
    [-5, 0, 0, 0, 0, 0, 0, -5],
    [-5, 0, 0, 0, 0, 0, 0, -5],
    [-5, 0, 0, 0, 0, 0, 0, -5],
    [-5, 0, 0, 0, 0, 0, 0, -5],
    [-5, 0, 0, 0, 0, 0, 0, -5],
    [5, 10, 10, 10, 10, 10, 10, 5],
    [0, 0, 0, 0, 0, 0, 0, 0],
];

static WHITE_BISHOP_POSITIONAL_VALUE: [[i32; 8]; 8] = [
    [-20, -10, -10, -10, -10, -10, -10, -20],
    [-10, 5, 0, 0, 0, 0, 5, -10],
    [-10, 10, 10, 10, 10, 10, 10, -10],
    [-10, 0, 10, 10, 10, 10, 0, -10],
    [-10, 5, 5, 10, 10, 5, 5, -10],
    [-10, 0, 5, 10, 10, 5, 0, -10],
    [-10, 0, 0, 0, 0, 0, 0, -10],
    [-20, -10, -10, -10, -10, -10, -10, -20],
];

static KNIGHT_POSITIONAL_VALUE: [[i32; 8]; 8] = [
    [-50, -40, -30, -30, -30, -30, -40, -50],
    [-40, -20, 0, 0, 0, 0, -20, -40],
    [-30, 0, 10, 15, 15, 10, 0, -30],
    [-30, 5, 15, 20, 20, 15, 5, -30],
    [-30, 0, 15, 20, 20, 15, 0, -30],
    [-30, 5, 10, 15, 15, 10, 5, -30],
    [-40, -20, 0, 5, 5, 0, -20, -40],
    [-50, -40, -30, -30, -30, -30, -40, -50],
];

static QUEEN_POSITIONAL_VALUE: [[i32; 8]; 8] = [
    [-20, -10, -10, -5, -5, -10, -10, -20],
    [-10, 0, 5, 0, 0, 0, 0, -10],
    [-10, 5, 5, 5, 5, 5, 0, -10],
    [0, 0, 5, 5, 5, 5, 0, -5],
    [-5, 0, 5, 5, 5, 5, 0, -5],
    [-10, 0, 5, 5, 5, 5, 0, -10],
    [-10, 0, 0, 0, 0, 0, 0, -10],
    [-20, -10, -10, -5, -5, -10, -10, -20],
];

static WHITE_KING_POSITIONAL_VALUE: [[i32; 8]; 8] = [
    [20, 30, 10, 0, 0, 10, 30, 20],
    [20, 20, 0, 0, 0, 0, 20, 20],
    [-10, -20, -20, -20, -20, -20, -20, -10],
    [-20, -30, -30, -40, -40, -30, -30, -20],
    [-30, -40, -40, -50, -50, -40, -40, -30],
    [-30, -40, -40, -50, -50, -40, -40, -30],
    [-30, -40, -40, -50, -50, -40, -40, -30],
    [-30, -40, -40, -50, -50, -40, -40, -30],
];

lazy_static! {
    static ref BLACK_PAWN_POSITIONAL_VALUE: [[i32; 8]; 8] = {
        let mut black_pawn_positional_value = WHITE_PAWN_POSITIONAL_VALUE;
        black_pawn_positional_value.reverse();
        black_pawn_positional_value
    };
    static ref BLACK_BISHOP_POSITIONAL_VALUE: [[i32; 8]; 8] = {
        let mut black_bishop_positional_value = WHITE_BISHOP_POSITIONAL_VALUE;
        black_bishop_positional_value.reverse();
        black_bishop_positional_value
    };
    static ref BLACK_ROOK_POSITIONAL_VALUE: [[i32; 8]; 8] = {
        let mut black_rook_positional_value = WHITE_ROOK_POSITIONAL_VALUE;
        black_rook_positional_value.reverse();
        black_rook_positional_value
    };
    static ref BLACK_KING_POSITIONAL_VALUE: [[i32; 8]; 8] = {
        let mut black_king_positional_value = WHITE_KING_POSITIONAL_VALUE;
        black_king_positional_value.reverse();
        black_king_positional_value
    };
}

fn get_raw_piece_value(pt: PieceType) -> i32 {
    match pt {
        PieceType::Pawn => 100,
        PieceType::Rook => 500,
        PieceType::Knight => 300,
        PieceType::Bishop => 300,
        PieceType::Queen => 900,
        PieceType::King => 9000,
    }
}

fn get_pawn_positional_value(color: Color, coord: Coordinate) -> i32 {
    match color {
        Color::White => WHITE_PAWN_POSITIONAL_VALUE[coord.get_rank() - 1][coord.get_file() - 1],
        Color::Black => BLACK_PAWN_POSITIONAL_VALUE[coord.get_rank() - 1][coord.get_file() - 1],
    }
}

fn get_knight_positional_value(coord: Coordinate) -> i32 {
    KNIGHT_POSITIONAL_VALUE[coord.get_rank() - 1][coord.get_file() - 1]
}

fn get_bishop_positional_value(color: Color, coord: Coordinate) -> i32 {
    match color {
        Color::White => WHITE_BISHOP_POSITIONAL_VALUE[coord.get_rank() - 1][coord.get_file() - 1],
        Color::Black => BLACK_BISHOP_POSITIONAL_VALUE[coord.get_rank() - 1][coord.get_file() - 1],
    }
}

fn get_rook_positional_value(color: Color, coord: Coordinate) -> i32 {
    match color {
        Color::White => WHITE_ROOK_POSITIONAL_VALUE[coord.get_rank() - 1][coord.get_file() - 1],
        Color::Black => BLACK_ROOK_POSITIONAL_VALUE[coord.get_rank() - 1][coord.get_file() - 1],
    }
}

fn get_queen_positional_value(coord: Coordinate) -> i32 {
    QUEEN_POSITIONAL_VALUE[coord.get_rank() - 1][coord.get_file() - 1]
}

fn get_king_positional_value(color: Color, coord: Coordinate) -> i32 {
    match color {
        Color::White => WHITE_KING_POSITIONAL_VALUE[coord.get_rank() - 1][coord.get_file() - 1],
        Color::Black => BLACK_KING_POSITIONAL_VALUE[coord.get_rank() - 1][coord.get_file() - 1],
    }
}

fn get_piece_positional_value(color: Color, piece_type: PieceType, coord: Coordinate) -> i32 {
    match piece_type {
        PieceType::Pawn => get_pawn_positional_value(color, coord),
        PieceType::Bishop => get_bishop_positional_value(color, coord),
        PieceType::King => get_king_positional_value(color, coord),
        PieceType::Rook => get_rook_positional_value(color, coord),
        PieceType::Queen => get_queen_positional_value(coord),
        PieceType::Knight => get_knight_positional_value(coord),
    }
}

fn get_piece_value(color: Color, piece_type: PieceType, coord: Coordinate) -> i32 {
    let val = get_raw_piece_value(piece_type) as i32
        + get_piece_positional_value(color, piece_type, coord);

    if color == Color::White {
        val
    } else {
        -val
    }
}

pub fn evaluate_board(board: &Board) -> i32 {
    let mut score = 0;
    for (coord, piece) in board.get_all_pieces() {
        score += get_piece_value(piece.color, piece.piece_type, coord);
    }
    score
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_pawn_positional_value() {
        let test_cases = [
            (Color::White, Coordinate::E4, 20),
            (Color::White, Coordinate::E7, 50),
            (Color::Black, Coordinate::E4, 25),
            (Color::Black, Coordinate::E2, 50),
        ];

        for (color, coord, eval) in test_cases.iter() {
            assert_eq!(get_pawn_positional_value(*color, *coord), *eval);
        }
    }

    #[test]
    fn test_get_knight_positional_value() {
        let test_cases = [
            (Coordinate::E4, 20),
            (Coordinate::E8, -30),
            (Coordinate::H8, -50),
        ];

        for (coord, eval) in test_cases.iter() {
            assert_eq!(get_knight_positional_value(*coord), *eval);
        }
    }

    #[test]
    fn test_get_bishop_positional_value() {
        let test_cases = [
            (Color::White, Coordinate::G2, 5),
            (Color::White, Coordinate::A8, -20),
            (Color::Black, Coordinate::B8, -10),
            (Color::Black, Coordinate::E5, 10),
        ];

        for (color, coord, eval) in test_cases.iter() {
            assert_eq!(get_bishop_positional_value(*color, *coord), *eval);
        }
    }

    #[test]
    fn test_get_rook_positional_value() {
        let test_cases = [
            (Color::White, Coordinate::A1, 0),
            (Color::White, Coordinate::D1, 5),
            (Color::White, Coordinate::A2, -5),
            (Color::White, Coordinate::G7, 10),
            (Color::Black, Coordinate::E8, 5),
            (Color::Black, Coordinate::H4, -5),
        ];

        for (color, coord, eval) in test_cases.iter() {
            assert_eq!(get_rook_positional_value(*color, *coord), *eval);
        }
    }

    #[test]
    fn test_get_queen_positional_value() {
        let test_cases = [
            (Coordinate::A1, -20),
            (Coordinate::D4, 5),
            (Coordinate::H8, -20),
        ];

        for (coord, eval) in test_cases.iter() {
            assert_eq!(get_queen_positional_value(*coord), *eval);
        }
    }

    #[test]
    fn test_get_king_positional_value() {
        let test_cases = [
            (Color::White, Coordinate::G1, 30),
            (Color::White, Coordinate::E5, -50),
            (Color::Black, Coordinate::G8, 30),
            (Color::Black, Coordinate::C2, -40),
        ];

        for (color, coord, eval) in test_cases.iter() {
            assert_eq!(get_king_positional_value(*color, *coord), *eval);
        }
    }
}
