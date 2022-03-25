use crate::board::movements::*;
use crate::board::{Board, Color, Coordinate, Piece, PieceType};
use crate::r#move::Move;
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

pub fn piece_value_difference(p1: PieceType, p2: PieceType) -> i32 {
    get_raw_piece_value(p1) - get_raw_piece_value(p2)
}

// Finds the piece with the least value that is attacking
// a square. Returns the piece and its source square
fn get_smallest_attacker(board: &Board, square: Coordinate) -> Option<(Piece, Coordinate)> {
    let color = board.get_player_color();

    // Here we assume that knights are worth slightly less than bishop
    if let Some(coord) = square_controlled_by_pawn_from(board, color, square) {
        return Some((Piece::new(color, PieceType::Pawn), coord));
    }

    if let Some(coord) = square_controlled_by_knight_from(board, color, square) {
        return Some((Piece::new(color, PieceType::Knight), coord));
    }

    if let Some((_, coord)) =
        square_controlled_by_bishop_or_queen_from(board, color, square, PieceType::Bishop)
    {
        return Some((Piece::new(color, PieceType::Bishop), coord));
    }

    if let Some((_, coord)) =
        square_controlled_by_rook_or_queen_from(board, color, square, PieceType::Rook)
    {
        return Some((Piece::new(color, PieceType::Rook), coord));
    }

    if let Some((_, coord)) =
        square_controlled_by_bishop_or_queen_from(board, color, square, PieceType::Queen)
    {
        return Some((Piece::new(color, PieceType::Queen), coord));
    }

    if let Some((_, coord)) =
        square_controlled_by_rook_or_queen_from(board, color, square, PieceType::Queen)
    {
        return Some((Piece::new(color, PieceType::Queen), coord));
    }

    if let Some(coord) = square_controlled_by_king_from(board, color, square) {
        return Some((Piece::new(color, PieceType::King), coord));
    }
    None
}

// Assume that there is something to be captured on the square
// This function applies moves to the board without undoing it.
pub fn static_exchange_evaluation(mut board: Board, square: Coordinate) -> i32 {
    if let Some((piece, src)) = get_smallest_attacker(&board, square) {
        let victim_piece_type = board
            .get_from_coordinate(square)
            .expect("There should be a piece to capture here")
            .piece_type;
        let attacking_move = Move::new_capture(src, square, piece, victim_piece_type);
        board.apply_move(&attacking_move);
        board.print();

        return get_raw_piece_value(victim_piece_type) - static_exchange_evaluation(board, square);
    } else {
        return 0;
    }
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

    #[test]
    fn get_smallest_attacker_pawn() {
        let mut board = Board::new_empty();
        let pieces = [
            (PieceType::King, Coordinate::D4),
            (PieceType::Queen, Coordinate::A8),
            (PieceType::Rook, Coordinate::E1),
            (PieceType::Bishop, Coordinate::C2),
            (PieceType::Knight, Coordinate::G3),
            (PieceType::Pawn, Coordinate::F3),
        ];
        for (p, coord) in pieces {
            board.place_piece(coord, Piece::new(Color::White, p));
            assert_eq!(
                get_smallest_attacker(&board, Coordinate::E4).unwrap().1,
                coord
            );
        }
    }

    #[test]
    fn static_exchange_evaluation_initiated_by_white_with_simple_hanging_piece() {
        let mut board = Board::new_empty();
        board.set_player_color(Color::White);
        let pieces = [
            (PieceType::Knight, Coordinate::C3, Color::White),
            (PieceType::Pawn, Coordinate::D5, Color::Black),
        ];
        for (p, coord, color) in pieces {
            board.place_piece(coord, Piece::new(color, p));
        }
        assert_eq!(
            static_exchange_evaluation(board, Coordinate::D5),
            get_raw_piece_value(PieceType::Pawn)
        );
    }

    #[test]
    fn static_exchange_evaluation_initiated_by_white_with_defended_piece() {
        let mut board = Board::new_empty();
        board.set_player_color(Color::White);
        let pieces = [
            (PieceType::Knight, Coordinate::C3, Color::White),
            (PieceType::Pawn, Coordinate::D5, Color::Black),
            (PieceType::Knight, Coordinate::F6, Color::Black),
        ];
        for (p, coord, color) in pieces {
            board.place_piece(coord, Piece::new(color, p));
        }
        assert_eq!(
            static_exchange_evaluation(board, Coordinate::D5),
            get_raw_piece_value(PieceType::Pawn) - get_raw_piece_value(PieceType::Knight)
        );
    }

    #[test]
    fn static_exchange_evaluation_initiated_by_white_with_white_advantage() {
        let mut board = Board::new_empty();
        board.set_player_color(Color::White);
        let pieces = [
            (PieceType::Knight, Coordinate::C3, Color::White),
            (PieceType::Pawn, Coordinate::E4, Color::White),
            (PieceType::Knight, Coordinate::F6, Color::Black),
            (PieceType::Pawn, Coordinate::D5, Color::Black),
        ];
        for (p, coord, color) in pieces {
            board.place_piece(coord, Piece::new(color, p));
        }
        assert_eq!(
            static_exchange_evaluation(board, Coordinate::D5),
            get_raw_piece_value(PieceType::Knight)
        );
    }
}
