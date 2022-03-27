use crate::board::movements::*;
use crate::board::{Board, Color, Coordinate, Phase, Piece, PieceType};
use crate::r#move::Move;

use lazy_static::lazy_static;
use strum::IntoEnumIterator;

static MIDGAME_PHASE_LIMIT: i32 = 15258; // Upper bound of midgame material value
static ENDGAME_PHASE_LIMIT: i32 = 3915; // Lower bound of endgame material value
static MIDGAME_SCALE: i32 = 128;

#[derive(Debug, Clone, Copy)]
struct Score(i32, i32);

impl Score {
    pub fn get_for_phase(&self, p: Phase) -> i32 {
        if p == Phase::Midgame {
            self.0
        } else {
            self.1
        }
    }
}

// Positional values are inspired by Stockfish 14
// https://github.com/official-stockfish/Stockfish
#[rustfmt::skip]
// First row is the first rank, first column is the A file
static WHITE_PAWN_POSITIONAL_VALUE: [[Score; 8]; 8] = [
    [ Score( 0,  0), Score(  0,   0), Score(  0,   0), Score(  0,  0), Score( 0,   0), Score(0, 0), Score(0, 0), Score(0, 0)], 
    [ Score( 2, -8), Score(  4,  -6), Score( 11,   9), Score( 18,  5), Score(16,  16), Score(21, 6), Score(9, -6), Score(-3, -18)], 
    [ Score(-9, -9), Score(-15,  -7), Score( 11, -10), Score( 15,  5), Score(31,   2), Score(23, 3), Score(6, -8), Score(-20, -5)], 
    [ Score(-3,  7), Score(-20,   1), Score(  8,  -8), Score( 19, -2), Score(39, -14), Score(17, -13), Score(2, -11), Score(-5, -6)], 
    [ Score(11, 12), Score( -4,   6), Score(-11,   2), Score(  2, -6), Score(11,  -5), Score(0, -4), Score(-12, 14), Score(5, 9)], 
    [ Score( 3, 27), Score(-11,  18), Score( -6,  19), Score( 22, 29), Score(-8,  30), Score(-5, 9), Score(-14, 8), Score(-11, 14)], 
    [ Score(-7, -1), Score(  6, -14), Score( -2,  13), Score(-11, 22), Score( 4,  24), Score(-14, 17), Score(10, 7), Score(-9, 7)], 
    [ Score( 0,  0), Score(  0,   0), Score(  0,   0), Score(  0,  0), Score( 0,   0), Score(0, 0), Score(0, 0), Score(0, 0)]
];

#[rustfmt::skip]
static WHITE_ROOK_POSITIONAL_VALUE: [[Score; 8]; 8] = [
   [ Score(-31, -9), Score(-20,-13), Score(-14,-10), Score(-5, -9), Score(-5, -9), Score(-14,-10), Score(-20,-13), Score(-31, -9) ],
   [ Score(-21,-12), Score(-13, -9), Score( -8, -1), Score( 6, -2), Score( 6, -2), Score( -8, -1), Score(-13, -9), Score(-21,-12) ],
   [ Score(-25,  6), Score(-11, -8), Score( -1, -2), Score( 3, -6), Score( 3, -6), Score( -1, -2), Score(-11, -8), Score(-25,  6) ],
   [ Score(-13, -6), Score( -5,  1), Score( -4, -9), Score(-6,  7), Score(-6,  7), Score( -4, -9), Score( -5,  1), Score(-13, -6) ],
   [ Score(-27, -5), Score(-15,  8), Score( -4,  7), Score( 3, -6), Score( 3, -6), Score( -4,  7), Score(-15,  8), Score(-27, -5) ],
   [ Score(-22,  6), Score( -2,  1), Score(  6, -7), Score(12, 10), Score(12, 10), Score(  6, -7), Score( -2,  1), Score(-22,  6) ],
   [ Score( -2,  4), Score( 12,  5), Score( 16, 20), Score(18, -5), Score(18, -5), Score( 16, 20), Score( 12,  5), Score( -2,  4) ],
   [ Score(-17, 18), Score(-19,  0), Score( -1, 19), Score( 9, 13), Score( 9, 13), Score( -1, 19), Score(-19,  0), Score(-17, 18) ]
];

#[rustfmt::skip]
static WHITE_BISHOP_POSITIONAL_VALUE: [[Score; 8]; 8] = [
    [ Score(-37,-40), Score(-4 ,-21), Score( -6,-26), Score(-16, -8), Score(-16, -8), Score( -6,-26), Score(-4 ,-21), Score(-37,-40)],
    [ Score(-11,-26), Score(  6, -9), Score( 13,-12), Score(  3,  1), Score(  3,  1), Score( 13,-12), Score(  6, -9), Score(-11,-26)],
    [ Score(-5 ,-11), Score( 15, -1), Score( -4, -1), Score( 12,  7), Score( 12,  7), Score( -4, -1), Score( 15, -1), Score(-5 ,-11)],
    [ Score(-4 ,-14), Score(  8, -4), Score( 18,  0), Score( 27, 12), Score( 27, 12), Score( 18,  0), Score(  8, -4), Score(-4 ,-14)],
    [ Score(-8 ,-12), Score( 20, -1), Score( 15,-10), Score( 22, 11), Score( 22, 11), Score( 15,-10), Score( 20, -1), Score(-8 ,-12)],
    [ Score(-11,-21), Score(  4,  4), Score(  1,  3), Score(  8,  4), Score(  8,  4), Score(  1,  3), Score(  4,  4), Score(-11,-21)],
    [ Score(-12,-22), Score(-10,-14), Score(  4, -1), Score(  0,  1), Score(  0,  1), Score(  4, -1), Score(-10,-14), Score(-12,-22)],
    [ Score(-34,-32), Score(  1,-29), Score(-10,-26), Score(-16,-17), Score(-16,-17), Score(-10,-26), Score(  1,-29), Score(-34,-32)]
];

#[rustfmt::skip]
static WHITE_KNIGHT_POSITIONAL_VALUE: [[Score; 8]; 8] = [
    [
        Score(-175, -96),
        Score(-92, -65),
        Score(-74, -49),
        Score(-73, -21),
        Score(-73, -21),
        Score(-74, -49),
        Score(-92, -65),
        Score(-175, -96),
    ],
    [
        Score(-77, -67),
        Score(-41, -54),
        Score(-27, -18),
        Score(-15, 8),
        Score(-15, 8),
        Score(-27, -18),
        Score(-41, -54),
        Score(-77, -67),
    ],
    [
        Score(-61, -40),
        Score(-17, -27),
        Score(6, -8),
        Score(12, 29),
        Score(12, 29),
        Score(6, -8),
        Score(-17, -27),
        Score(-61, -40),
    ],
    [
        Score(-35, -35),
        Score(8, -2),
        Score(40, 13),
        Score(49, 28),
        Score(49, 28),
        Score(40, 13),
        Score(8, -2),
        Score(-35, -35),
    ],
    [
        Score(-34, -45),
        Score(13, -16),
        Score(44, 9),
        Score(51, 39),
        Score(51, 39),
        Score(44, 9),
        Score(13, -16),
        Score(-34, -45),
    ],
    [
        Score(-9, -51),
        Score(22, -44),
        Score(58, -16),
        Score(53, 17),
        Score(53, 17),
        Score(58, -16),
        Score(22, -44),
        Score(-9, -51),
    ],
    [
        Score(-67, -69),
        Score(-27, -50),
        Score(4, -51),
        Score(37, 12),
        Score(37, 12),
        Score(4, -51),
        Score(-27, -50),
        Score(-67, -69),
    ],
    [
        Score(-201, -100),
        Score(-83, -88),
        Score(-56, -56),
        Score(-26, -17),
        Score(-26, -17),
        Score(-56, -56),
        Score(-83, -88),
        Score(-201, -100),
    ],
];

#[rustfmt::skip]
static WHITE_QUEEN_POSITIONAL_VALUE: [[Score; 8]; 8] = [
    [
        Score(3, -69),
        Score(-5, -57),
        Score(-5, -47),
        Score(4, -26),
        Score(4, -26),
        Score(-5, -47),
        Score(-5, -57),
        Score(3, -69),
    ],
    [
        Score(-3, -54),
        Score(5, -31),
        Score(8, -22),
        Score(12, -4),
        Score(12, -4),
        Score(8, -22),
        Score(5, -31),
        Score(-3, -54),
    ],
    [
        Score(-3, -39),
        Score(6, -18),
        Score(13, -9),
        Score(7, 3),
        Score(7, 3),
        Score(13, -9),
        Score(6, -18),
        Score(-3, -39),
    ],
    [
        Score(4, -23),
        Score(5, -3),
        Score(9, 13),
        Score(8, 24),
        Score(8, 24),
        Score(9, 13),
        Score(5, -3),
        Score(4, -23),
    ],
    [
        Score(0, -29),
        Score(14, -6),
        Score(12, 9),
        Score(5, 21),
        Score(5, 21),
        Score(12, 9),
        Score(14, -6),
        Score(0, -29),
    ],
    [
        Score(-4, -38),
        Score(10, -18),
        Score(6, -11),
        Score(8, 1),
        Score(8, 1),
        Score(6, -11),
        Score(10, -18),
        Score(-4, -38),
    ],
    [
        Score(-5, -50),
        Score(6, -27),
        Score(10, -24),
        Score(8, -8),
        Score(8, -8),
        Score(10, -24),
        Score(6, -27),
        Score(-5, -50),
    ],
    [
        Score(-2, -74),
        Score(-2, -52),
        Score(1, -43),
        Score(-2, -34),
        Score(-2, -34),
        Score(1, -43),
        Score(-2, -52),
        Score(-2, -74),
    ],
];

#[rustfmt::skip]
static WHITE_KING_POSITIONAL_VALUE: [[Score; 8]; 8] = [
    [
        Score(271, 1),
        Score(327, 45),
        Score(271, 85),
        Score(198, 76),
        Score(198, 76),
        Score(271, 85),
        Score(327, 45),
        Score(271, 1),
    ],
    [
        Score(278, 53),
        Score(303, 100),
        Score(234, 133),
        Score(179, 135),
        Score(179, 135),
        Score(234, 133),
        Score(303, 100),
        Score(278, 53),
    ],
    [
        Score(195, 88),
        Score(258, 130),
        Score(169, 169),
        Score(120, 175),
        Score(120, 175),
        Score(169, 169),
        Score(258, 130),
        Score(195, 88),
    ],
    [
        Score(164, 103),
        Score(190, 156),
        Score(138, 172),
        Score(98, 172),
        Score(98, 172),
        Score(138, 172),
        Score(190, 156),
        Score(164, 103),
    ],
    [
        Score(154, 96),
        Score(179, 166),
        Score(105, 199),
        Score(70, 199),
        Score(70, 199),
        Score(105, 199),
        Score(179, 166),
        Score(154, 96),
    ],
    [
        Score(123, 92),
        Score(145, 172),
        Score(81, 184),
        Score(31, 191),
        Score(31, 191),
        Score(81, 184),
        Score(145, 172),
        Score(123, 92),
    ],
    [
        Score(88, 47),
        Score(120, 121),
        Score(65, 116),
        Score(33, 131),
        Score(33, 131),
        Score(65, 116),
        Score(120, 121),
        Score(88, 47),
    ],
    [
        Score(59, 11),
        Score(89, 59),
        Score(45, 73),
        Score(-1, 78),
        Score(-1, 78),
        Score(45, 73),
        Score(89, 59),
        Score(59, 11),
    ],
];

lazy_static! {
    static ref PIECE_POSITIONAL_VALUES: [[[[Score; 8]; 8]; 6]; 2] = {
        let mut vals = [[[[Score(0, 0); 8]; 8]; 6]; 2];

        for piece_type in PieceType::iter() {
            let mut scores = match piece_type {
                PieceType::Pawn => WHITE_PAWN_POSITIONAL_VALUE,
                PieceType::Bishop => WHITE_BISHOP_POSITIONAL_VALUE,
                PieceType::Knight => WHITE_KNIGHT_POSITIONAL_VALUE,
                PieceType::Rook => WHITE_ROOK_POSITIONAL_VALUE,
                PieceType::Queen => WHITE_QUEEN_POSITIONAL_VALUE,
                PieceType::King => WHITE_KING_POSITIONAL_VALUE,
            };

            vals[Color::White as usize][piece_type as usize] = scores;
            scores.reverse();
            vals[Color::Black as usize][piece_type as usize] = scores;
        }
        vals
    };
}

pub fn get_raw_piece_value(pt: PieceType, phase: Phase) -> i32 {
    if phase == Phase::Midgame {
        match pt {
            PieceType::Pawn => 126,
            PieceType::Knight => 781,
            PieceType::Bishop => 825,
            PieceType::Rook => 1276,
            PieceType::Queen => 2538,
            PieceType::King => 0,
        }
    } else {
        match pt {
            PieceType::Pawn => 208,
            PieceType::Knight => 854,
            PieceType::Bishop => 915,
            PieceType::Rook => 1380,
            PieceType::Queen => 2682,
            PieceType::King => 0,
        }
    }
}

fn get_piece_positional_value(
    color: Color,
    piece_type: PieceType,
    coord: Coordinate,
    phase: Phase,
) -> i32 {
    PIECE_POSITIONAL_VALUES[color as usize][piece_type as usize][coord.get_rank() - 1]
        [coord.get_file() - 1]
        .get_for_phase(phase)
}

fn get_piece_value(color: Color, piece_type: PieceType, coord: Coordinate, phase: Phase) -> i32 {
    let val = get_raw_piece_value(piece_type, phase) as i32
        + get_piece_positional_value(color, piece_type, coord, phase);

    if color == Color::White {
        val
    } else {
        -val
    }
}

// Returns something between 0 and MIDGAME_SCALE, the more pieces there are
// the higher the value is
fn calculate_phase(board: &Board) -> i32 {
    let npm = ENDGAME_PHASE_LIMIT.max(MIDGAME_PHASE_LIMIT.min(board.get_npm()));
    return ((npm - ENDGAME_PHASE_LIMIT) * MIDGAME_SCALE)
        / (MIDGAME_PHASE_LIMIT - ENDGAME_PHASE_LIMIT);
}

pub fn evaluate_board(board: &Board) -> i32 {
    let mut midgame_score = 0;
    let mut endgame_score = 0;

    let phase = calculate_phase(board);

    for (coord, piece) in board.get_all_pieces() {
        midgame_score += get_piece_value(piece.color, piece.piece_type, coord, Phase::Midgame);
        endgame_score += get_piece_value(piece.color, piece.piece_type, coord, Phase::Endgame);
    }
    (midgame_score * phase + (endgame_score * (MIDGAME_SCALE - phase))) / MIDGAME_SCALE
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
fn static_exchange_evaluation(mut board: Board, square: Coordinate) -> i32 {
    if let Some((piece, src)) = get_smallest_attacker(&board, square) {
        let victim_piece_type = board
            .get_from_coordinate(square)
            .expect("There should be a piece to capture here")
            .piece_type;
        let attacking_move = Move::new_capture(src, square, piece, victim_piece_type);
        board.apply_move(&attacking_move);

        return get_raw_piece_value(victim_piece_type, Phase::Midgame)
            - static_exchange_evaluation(board, square);
    } else {
        return 0;
    }
}

pub fn static_exchange_evaluation_capture(mut board: Board, m: &Move) -> i32 {
    // TODO: Deal with en passant
    if let Some(victim_piece) = board.get_from_coordinate(m.dest) {
        board.apply_move(m);
        return get_raw_piece_value(victim_piece.piece_type, Phase::Midgame)
            - static_exchange_evaluation(board, m.dest);
    } else {
        return 0;
    }
}

pub fn non_pawn_material(board: &Board) -> i32 {
    let mut val = 0;
    for (_, piece) in board.get_all_pieces() {
        if piece.piece_type != PieceType::Pawn {
            val += get_raw_piece_value(piece.piece_type, Phase::Midgame);
        }
    }
    val
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_pawn_positional_value() {
        let test_cases = [
            (Color::White, Coordinate::E4, 39, Phase::Midgame),
            (Color::White, Coordinate::E7, 4, Phase::Midgame),
            (Color::Black, Coordinate::E4, 11, Phase::Midgame),
            (Color::Black, Coordinate::E2, 4, Phase::Midgame),
            (Color::Black, Coordinate::A2, -7, Phase::Midgame),
            (Color::White, Coordinate::D7, 22, Phase::Endgame),
            (Color::White, Coordinate::D6, 29, Phase::Endgame),
        ];

        for (color, coord, eval, phase) in test_cases.iter() {
            assert_eq!(
                get_piece_positional_value(*color, PieceType::Pawn, *coord, *phase),
                *eval
            );
        }
    }

    #[test]
    fn test_get_knight_positional_value() {
        let test_cases = [
            (Color::White, Coordinate::E4, 49, Phase::Midgame),
            (Color::White, Coordinate::E8, -26, Phase::Midgame),
            (Color::White, Coordinate::H8, -201, Phase::Midgame),
            (Color::Black, Coordinate::E4, 51, Phase::Midgame),
            (Color::Black, Coordinate::E8, -73, Phase::Midgame),
            (Color::Black, Coordinate::H8, -175, Phase::Midgame),
            (Color::White, Coordinate::A1, -96, Phase::Endgame),
            (Color::White, Coordinate::E5, 39, Phase::Endgame),
        ];

        for (color, coord, eval, phase) in test_cases.iter() {
            assert_eq!(
                get_piece_positional_value(*color, PieceType::Knight, *coord, *phase),
                *eval
            );
        }
    }

    #[test]
    fn test_get_bishop_positional_value() {
        let test_cases = [
            (Color::White, Coordinate::G2, 6, Phase::Midgame),
            (Color::White, Coordinate::A8, -34, Phase::Midgame),
            (Color::Black, Coordinate::B8, -4, Phase::Midgame),
            (Color::Black, Coordinate::E5, 27, Phase::Midgame),
            (Color::White, Coordinate::C4, 0, Phase::Endgame),
            (Color::Black, Coordinate::G1, -29, Phase::Endgame),
        ];

        for (color, coord, eval, phase) in test_cases.iter() {
            assert_eq!(
                get_piece_positional_value(*color, PieceType::Bishop, *coord, *phase),
                *eval
            );
        }
    }

    #[test]
    fn test_get_rook_positional_value() {
        let test_cases = [
            (Color::White, Coordinate::A1, -31, Phase::Midgame),
            (Color::White, Coordinate::D1, -5, Phase::Midgame),
            (Color::White, Coordinate::A2, -21, Phase::Midgame),
            (Color::White, Coordinate::G7, 12, Phase::Midgame),
            (Color::Black, Coordinate::E8, -5, Phase::Midgame),
            (Color::Black, Coordinate::H4, -27, Phase::Midgame),
            (Color::White, Coordinate::C7, 20, Phase::Endgame),
            (Color::Black, Coordinate::D2, -5, Phase::Endgame),
        ];

        for (color, coord, eval, phase) in test_cases.iter() {
            assert_eq!(
                get_piece_positional_value(*color, PieceType::Rook, *coord, *phase),
                *eval
            );
        }
    }

    #[test]
    fn test_get_queen_positional_value() {
        let test_cases = [
            (Color::White, Coordinate::A1, 3, Phase::Midgame),
            (Color::White, Coordinate::D4, 8, Phase::Midgame),
            (Color::White, Coordinate::H8, -2, Phase::Midgame),
            (Color::White, Coordinate::C1, -47, Phase::Endgame),
            (Color::White, Coordinate::H8, -74, Phase::Endgame),
        ];

        for (color, coord, eval, phase) in test_cases.iter() {
            assert_eq!(
                get_piece_positional_value(*color, PieceType::Queen, *coord, *phase),
                *eval
            );
        }
    }

    #[test]
    fn test_get_king_positional_value() {
        let test_cases = [
            (Color::White, Coordinate::G1, 327, Phase::Midgame),
            (Color::White, Coordinate::E5, 70, Phase::Midgame),
            (Color::Black, Coordinate::G8, 327, Phase::Midgame),
            (Color::Black, Coordinate::C2, 65, Phase::Midgame),
            (Color::White, Coordinate::A1, 1, Phase::Endgame),
            (Color::Black, Coordinate::D4, 199, Phase::Endgame),
        ];

        for (color, coord, eval, phase) in test_cases.iter() {
            assert_eq!(
                get_piece_positional_value(*color, PieceType::King, *coord, *phase),
                *eval
            );
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
            get_raw_piece_value(PieceType::Pawn, Phase::Midgame)
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
            get_raw_piece_value(PieceType::Pawn, Phase::Midgame)
                - get_raw_piece_value(PieceType::Knight, Phase::Midgame)
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
            get_raw_piece_value(PieceType::Knight, Phase::Midgame)
        );
    }

    #[test]
    fn phase_calculation_with_full_board() {
        let board = Board::new_starting_pos();
        assert_eq!(calculate_phase(&board), MIDGAME_SCALE);
    }

    #[test]
    fn phase_calculation_with_empty_board() {
        let board = Board::new_empty();
        assert_eq!(calculate_phase(&board), 0);
    }

    #[test]
    fn calculate_non_pawn_material() {
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
        }

        assert_eq!(
            non_pawn_material(&board),
            get_raw_piece_value(PieceType::Queen, Phase::Midgame)
                + get_raw_piece_value(PieceType::Rook, Phase::Midgame)
                + get_raw_piece_value(PieceType::Bishop, Phase::Midgame)
                + get_raw_piece_value(PieceType::Knight, Phase::Midgame)
        )
    }
}
