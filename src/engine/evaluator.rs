use crate::board::{Board, Color, Coordinate, PieceType};
use lazy_static::lazy_static;

// First row is the first rank, first column is the H file
static WHITE_PAWN_POSITIONAL_VALUE: [[f32; 8]; 8] = [
    [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    [0.5, 1.0, 1.0, -2.0, -2.0, 1.0, 1.0, 0.5],
    [0.5, -0.5, -1.0, 0.0, 0.0, -1.0, -0.5, 0.5],
    [0.0, 0.0, 0.0, 2.0, 2.0, 0.0, 0.0, 0.0],
    [0.5, 0.5, 1.0, 2.5, 2.5, 1.0, 0.5, 0.5],
    [1.0, 1.0, 2.0, 3.0, 3.0, 2.0, 1.0, 1.0],
    [5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0],
    [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
];

static WHITE_ROOK_POSITIONAL_VALUE: [[f32; 8]; 8] = [
    [0.0, 0.0, 0.0, 0.5, 0.5, 0.0, 0.0, 0.0],
    [-0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -0.5],
    [-0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -0.5],
    [-0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -0.5],
    [-0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -0.5],
    [-0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -0.5],
    [0.5, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.5],
    [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
];

static WHITE_BISHOP_POSITIONAL_VALUE: [[f32; 8]; 8] = [
    [-2.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -2.0],
    [-1.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.5, -1.0],
    [-1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, -1.0],
    [-1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0, -1.0],
    [-1.0, 0.5, 0.5, 1.0, 1.0, 0.5, 0.5, -1.0],
    [-1.0, 0.0, 0.5, 1.0, 1.0, 0.5, 0.0, -1.0],
    [-1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -1.0],
    [-2.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -2.0],
];

static KNIGHT_POSITIONAL_VALUE: [[f32; 8]; 8] = [
    [-5.0, -4.0, -3.0, -3.0, -3.0, -3.0, -4.0, -5.0],
    [-4.0, -2.0, 0.0, 0.0, 0.0, 0.0, -2.0, -4.0],
    [-3.0, 0.0, 1.0, 1.5, 1.5, 1.0, 0.0, -3.0],
    [-3.0, 0.5, 1.5, 2.0, 2.0, 1.5, 0.5, -3.0],
    [-3.0, 0.0, 1.5, 2.0, 2.0, 1.5, 0.0, -3.0],
    [-3.0, 0.5, 1.0, 1.5, 1.5, 1.0, 0.5, -3.0],
    [-4.0, -2.0, 0.0, 0.5, 0.5, 0.0, -2.0, -4.0],
    [-5.0, -4.0, -3.0, -3.0, -3.0, -3.0, -4.0, -5.0],
];

static QUEEN_POSITIONAL_VALUE: [[f32; 8]; 8] = [
    [-2.0, -1.0, -1.0, -0.5, -0.5, -1.0, -1.0, -2.0],
    [-1.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.0, -1.0],
    [-1.0, 0.5, 0.5, 0.5, 0.5, 0.5, 0.0, -1.0],
    [0.0, 0.0, 0.5, 0.5, 0.5, 0.5, 0.0, -0.5],
    [-0.5, 0.0, 0.5, 0.5, 0.5, 0.5, 0.0, -0.5],
    [-1.0, 0.0, 0.5, 0.5, 0.5, 0.5, 0.0, -1.0],
    [-1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -1.0],
    [-2.0, -1.0, -1.0, -0.5, -0.5, -1.0, -1.0, -2.0],
];

static WHITE_KING_POSITIONAL_VALUE: [[f32; 8]; 8] = [
    [2.0, 3.0, 1.0, 0.0, 0.0, 1.0, 3.0, 2.0],
    [2.0, 2.0, 0.0, 0.0, 0.0, 0.0, 2.0, 2.0],
    [-1.0, -2.0, -2.0, -2.0, -2.0, -2.0, -2.0, -1.0],
    [-2.0, -3.0, -3.0, -4.0, -4.0, -3.0, -3.0, -2.0],
    [-3.0, -4.0, -4.0, -5.0, -5.0, -4.0, -4.0, -3.0],
    [-3.0, -4.0, -4.0, -5.0, -5.0, -4.0, -4.0, -3.0],
    [-3.0, -4.0, -4.0, -5.0, -5.0, -4.0, -4.0, -3.0],
    [-3.0, -4.0, -4.0, -5.0, -5.0, -4.0, -4.0, -3.0],
];

lazy_static! {
    static ref BLACK_PAWN_POSITIONAL_VALUE: [[f32; 8]; 8] = {
        let mut black_pawn_positional_value = WHITE_PAWN_POSITIONAL_VALUE;
        black_pawn_positional_value.reverse();
        black_pawn_positional_value
    };
    static ref BLACK_BISHOP_POSITIONAL_VALUE: [[f32; 8]; 8] = {
        let mut black_bishop_positional_value = WHITE_BISHOP_POSITIONAL_VALUE;
        black_bishop_positional_value.reverse();
        black_bishop_positional_value
    };
    static ref BLACK_ROOK_POSITIONAL_VALUE: [[f32; 8]; 8] = {
        let mut black_rook_positional_value = WHITE_ROOK_POSITIONAL_VALUE;
        black_rook_positional_value.reverse();
        black_rook_positional_value
    };
    static ref BLACK_KING_POSITIONAL_VALUE: [[f32; 8]; 8] = {
        let mut black_king_positional_value = WHITE_KING_POSITIONAL_VALUE;
        black_king_positional_value.reverse();
        black_king_positional_value
    };
}

fn get_raw_piece_value(pt: PieceType) -> i32 {
    match pt {
        PieceType::Pawn => 10,
        PieceType::Rook => 50,
        PieceType::Knight => 30,
        PieceType::Bishop => 30,
        PieceType::Queen => 90,
        PieceType::King => 900,
    }
}

fn get_pawn_positional_value(color: Color, coord: Coordinate) -> f32 {
    match color {
        Color::White => WHITE_PAWN_POSITIONAL_VALUE[coord.get_rank() - 1][coord.get_file() - 1],
        Color::Black => BLACK_PAWN_POSITIONAL_VALUE[coord.get_rank() - 1][coord.get_file() - 1],
    }
}

fn get_knight_positional_value(coord: Coordinate) -> f32 {
    KNIGHT_POSITIONAL_VALUE[coord.get_rank() - 1][coord.get_file() - 1]
}

fn get_bishop_positional_value(color: Color, coord: Coordinate) -> f32 {
    match color {
        Color::White => WHITE_BISHOP_POSITIONAL_VALUE[coord.get_rank() - 1][coord.get_file() - 1],
        Color::Black => BLACK_BISHOP_POSITIONAL_VALUE[coord.get_rank() - 1][coord.get_file() - 1],
    }
}

fn get_rook_positional_value(color: Color, coord: Coordinate) -> f32 {
    match color {
        Color::White => WHITE_ROOK_POSITIONAL_VALUE[coord.get_rank() - 1][coord.get_file() - 1],
        Color::Black => BLACK_ROOK_POSITIONAL_VALUE[coord.get_rank() - 1][coord.get_file() - 1],
    }
}

fn get_queen_positional_value(coord: Coordinate) -> f32 {
    QUEEN_POSITIONAL_VALUE[coord.get_rank() - 1][coord.get_file() - 1]
}

fn get_king_positional_value(color: Color, coord: Coordinate) -> f32 {
    match color {
        Color::White => WHITE_KING_POSITIONAL_VALUE[coord.get_rank() - 1][coord.get_file() - 1],
        Color::Black => BLACK_KING_POSITIONAL_VALUE[coord.get_rank() - 1][coord.get_file() - 1],
    }
}

fn get_piece_positional_value(color: Color, piece_type: PieceType, coord: Coordinate) -> f32 {
    match piece_type {
        PieceType::Pawn => get_pawn_positional_value(color, coord),
        PieceType::Bishop => get_bishop_positional_value(color, coord),
        PieceType::King => get_king_positional_value(color, coord),
        PieceType::Rook => get_rook_positional_value(color, coord),
        PieceType::Queen => get_queen_positional_value(coord),
        PieceType::Knight => get_knight_positional_value(coord),
    }
}

fn get_piece_value(color: Color, piece_type: PieceType, coord: Coordinate) -> f32 {
    let val = get_raw_piece_value(piece_type) as f32
        + get_piece_positional_value(color, piece_type, coord);

    if color == Color::White {
        val
    } else {
        -val
    }
}

pub fn evaluate_board(board: &Board) -> f32 {
    let mut score = 0.0;
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
            (Color::White, Coordinate::E4, 2.0),
            (Color::White, Coordinate::E7, 5.0),
            (Color::Black, Coordinate::E4, 2.5),
            (Color::Black, Coordinate::E2, 5.0),
        ];

        for (color, coord, eval) in test_cases.iter() {
            assert_eq!(get_pawn_positional_value(*color, *coord), *eval);
        }
    }

    #[test]
    fn test_get_knight_positional_value() {
        let test_cases = [
            (Coordinate::E4, 2.0),
            (Coordinate::E8, -3.0),
            (Coordinate::H8, -5.0),
        ];

        for (coord, eval) in test_cases.iter() {
            assert_eq!(get_knight_positional_value(*coord), *eval);
        }
    }

    #[test]
    fn test_get_bishop_positional_value() {
        let test_cases = [
            (Color::White, Coordinate::G2, 0.5),
            (Color::White, Coordinate::A8, -2.0),
            (Color::Black, Coordinate::B8, -1.0),
            (Color::Black, Coordinate::E5, 1.0),
        ];

        for (color, coord, eval) in test_cases.iter() {
            assert_eq!(get_bishop_positional_value(*color, *coord), *eval);
        }
    }

    #[test]
    fn test_get_rook_positional_value() {
        let test_cases = [
            (Color::White, Coordinate::A1, 0.0),
            (Color::White, Coordinate::D1, 0.5),
            (Color::White, Coordinate::A2, -0.5),
            (Color::White, Coordinate::G7, 1.0),
            (Color::Black, Coordinate::E8, 0.5),
            (Color::Black, Coordinate::H4, -0.5),
        ];

        for (color, coord, eval) in test_cases.iter() {
            assert_eq!(get_rook_positional_value(*color, *coord), *eval);
        }
    }

    #[test]
    fn test_get_queen_positional_value() {
        let test_cases = [
            (Coordinate::A1, -2.0),
            (Coordinate::D4, 0.5),
            (Coordinate::H8, -2.0),
        ];

        for (coord, eval) in test_cases.iter() {
            assert_eq!(get_queen_positional_value(*coord), *eval);
        }
    }

    #[test]
    fn test_get_king_positional_value() {
        let test_cases = [
            (Color::White, Coordinate::G1, 3.0),
            (Color::White, Coordinate::E5, -5.0),
            (Color::Black, Coordinate::G8, 3.0),
            (Color::Black, Coordinate::C2, -4.0),
        ];

        for (color, coord, eval) in test_cases.iter() {
            assert_eq!(get_king_positional_value(*color, *coord), *eval);
        }
    }
}
