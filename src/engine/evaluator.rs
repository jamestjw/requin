use crate::bitboard::*;
use crate::board::{
    dist_from_edge, relative_rank, Board, Color, Coordinate, Phase, Piece, PieceType,
};
use crate::generator::get_attackers_of_square_bb;
use crate::r#move::Move;

use lazy_static::lazy_static;
use std::convert::TryFrom;
use std::ops::{Add, AddAssign, Mul, Neg, Sub};
use strum::IntoEnumIterator;

static MIDGAME_PHASE_LIMIT: i32 = 15258; // Upper bound of midgame material value
static ENDGAME_PHASE_LIMIT: i32 = 3915; // Lower bound of endgame material value
static MIDGAME_SCALE: i32 = 128;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Score(i32, i32);

impl Score {
    pub fn get_for_phase(&self, p: Phase) -> i32 {
        if p == Phase::Midgame {
            self.0
        } else {
            self.1
        }
    }
}

impl Add for Score {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Score(self.0 + other.0, self.1 + other.1)
    }
}

impl Sub for Score {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Score(self.0 - other.0, self.1 - other.1)
    }
}

impl Mul<i32> for Score {
    type Output = Self;

    fn mul(self, other: i32) -> Self {
        Score(self.0 * other, self.1 * other)
    }
}

impl Neg for Score {
    type Output = Self;

    fn neg(self) -> Self {
        Score(-self.0, -self.1)
    }
}

impl AddAssign for Score {
    fn add_assign(&mut self, other: Self) {
        *self = Score(self.0 + other.0, self.1 + other.1);
    }
}

// Positional values are inspired by Stockfish 14
// https://github.com/official-stockfish/Stockfish
#[rustfmt::skip]
// First row is the first rank, first column is the A file
static WHITE_PAWN_POSITIONAL_VALUE: [[Score; 8]; 8] = [
    [ Score( 0,  0), Score(  0,   0), Score(  0,   0), Score(  0,  0), Score( 0,   0), Score(0, 0), Score(0, 0), Score(0, 0)], 
    [ Score( 2, -8), Score(  2,  -6), Score( 11,   9), Score( 16,  5), Score(16,  16), Score(21, 6), Score(9, -6), Score(-3, -18)], 
    [ Score( 5, -9), Score(  5,  -7), Score( 11, -10), Score( 25,  5), Score(31,   2), Score(23, 3), Score(6, -8), Score(-20, -5)], 
    [ Score(10,  7), Score( 10,   1), Score( 15,  -8), Score( 35, -2), Score(39, -14), Score(17, -13), Score(2, -11), Score(-5, -6)], 
    [ Score(11, 12), Score( -4,   6), Score(-11,   2), Score(  2, -6), Score(11,  -5), Score(0, -4), Score(-12, 14), Score(5, 9)], 
    [ Score(12, 27), Score(-11,  18), Score( -6,  19), Score( 22, 29), Score(-8,  30), Score(-5, 9), Score(-14, 8), Score(-11, 14)], 
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
// static WHITE_KNIGHT_POSITIONAL_VALUE: [[Score; 8]; 8] = [ 
//     [ Score(-175,  -96), Score(-92, -65), Score(-74, -49), Score(-73, -21), Score(-73, -21), Score(-74, -49), Score(-92, -65), Score(-175,  -96) ],
//     [ Score( -77,  -67), Score(-41, -54), Score(-27, -18), Score(-15,   8), Score(-15,   8), Score(-27, -18), Score(-41, -54), Score( -77,  -67) ],
//     [ Score( -61,  -40), Score(-17, -27), Score(  6,  -8), Score( 12,  29), Score( 12,  29), Score(  6,  -8), Score(-17, -27), Score( -61,  -40) ],
//     [ Score( -35,  -35), Score(  8,  -2), Score( 40,  13), Score( 49,  28), Score( 49,  28), Score( 40,  13), Score(  8,  -2), Score( -35,  -35) ],
//     [ Score( -34,  -45), Score( 13, -16), Score( 44,   9), Score( 51,  39), Score( 51,  39), Score( 44,   9), Score( 13, -16), Score( -34,  -45) ],
//     [ Score(  -9,  -51), Score( 22, -44), Score( 58, -16), Score( 53,  17), Score( 53,  17), Score( 58, -16), Score( 22, -44), Score(  -9,  -51) ],
//     [ Score( -67,  -69), Score(-27, -50), Score(  4, -51), Score( 37,  12), Score( 37,  12), Score(  4, -51), Score(-27, -50), Score( -67,  -69) ],
//     [ Score(-201, -100), Score(-83, -88), Score(-56, -56), Score(-26, -17), Score(-26, -17), Score(-56, -56), Score(-83, -88), Score(-201, -100) ],
// ];
static WHITE_KNIGHT_POSITIONAL_VALUE: [[Score; 8]; 8] = [ 
    [ Score(-175,  -96), Score(-92, -65), Score(-74, -49), Score(-73, -21), Score(-73, -21), Score(-74, -49), Score(-92, -65), Score(-175,  -96) ],
    [ Score( -77,  -67), Score(-41, -54), Score(-27, -18), Score(-15,   8), Score(-15,   8), Score(-27, -18), Score(-41, -54), Score( -77,  -67) ],
    [ Score( -61,  -40), Score(-17, -27), Score( 25,  -8), Score( 12,  29), Score( 12,  29), Score( 25,  -8), Score(-17, -27), Score( -61,  -40) ],
    [ Score( -35,  -35), Score(  8,  -2), Score( 40,  13), Score( 49,  28), Score( 49,  28), Score( 40,  13), Score(  8,  -2), Score( -35,  -35) ],
    [ Score( -34,  -45), Score( 13, -16), Score( 44,   9), Score( 51,  39), Score( 51,  39), Score( 44,   9), Score( 13, -16), Score( -34,  -45) ],
    [ Score(  -9,  -51), Score( 22, -44), Score( 58, -16), Score( 53,  17), Score( 53,  17), Score( 58, -16), Score( 22, -44), Score(  -9,  -51) ],
    [ Score( -67,  -69), Score(-27, -50), Score(  4, -51), Score( 37,  12), Score( 37,  12), Score(  4, -51), Score(-27, -50), Score( -67,  -69) ],
    [ Score(-201, -100), Score(-83, -88), Score(-56, -56), Score(-26, -17), Score(-26, -17), Score(-56, -56), Score(-83, -88), Score(-201, -100) ],
];

#[rustfmt::skip]
static WHITE_QUEEN_POSITIONAL_VALUE: [[Score; 8]; 8] = [
    [ Score( 3, -69), Score(-5, -57), Score(-5, -47), Score( 4, -26), Score( 4, -26), Score(-5, -47), Score(-5, -57), Score( 3, -69) ],
    [ Score(-3, -54), Score( 5, -31), Score( 8, -22), Score(12,  -4), Score(12,  -4), Score( 8, -22), Score( 5, -31), Score(-3, -54) ],
    [ Score(-3, -39), Score( 6, -18), Score(13,  -9), Score( 7,   3), Score( 7,   3), Score(13,  -9), Score( 6, -18), Score(-3, -39) ],
    [ Score( 4, -23), Score( 5,  -3), Score( 9,  13), Score( 8,  24), Score( 8,  24), Score( 9,  13), Score( 5,  -3), Score( 4, -23) ],
    [ Score( 0, -29), Score(14, - 6), Score(12,   9), Score( 5,  21), Score( 5,  21), Score(12,   9), Score(14,  -6), Score( 0, -29) ],
    [ Score(-4, -38), Score(10, -18), Score( 6, -11), Score( 8,   1), Score( 8,   1), Score( 6, -11), Score(10, -18), Score(-4, -38) ],
    [ Score(-5, -50), Score( 6, -27), Score(10, -24), Score( 8,  -8), Score( 8,  -8), Score(10, -24), Score( 6, -27), Score(-5, -50) ],
    [ Score(-2, -74), Score(-2, -52), Score( 1, -43), Score(-2, -34), Score(-2, -34), Score( 1, -43), Score(-2, -52), Score(-2, -74) ]
];

#[rustfmt::skip]
static WHITE_KING_POSITIONAL_VALUE: [[Score; 8]; 8] = [ 
    [ Score(271,  1), Score(327, 45), Score(271,  85), Score(198,  76), Score(198,  76), Score(271,  85), Score(327,  45), Score(271,   1) ],
    [ Score(278, 53), Score(303,100), Score(234, 133), Score(179, 135), Score(179,  35), Score(234, 133), Score(303, 100), Score(278,  53) ],
    [ Score(195, 88), Score(258,130), Score(169, 169), Score(120, 175), Score(120, 175), Score(169, 169), Score(258, 130), Score(195,  88) ],
    [ Score(164,103), Score(190,156), Score(138, 172), Score( 98, 172), Score( 98, 172), Score(138, 172), Score(190, 156), Score(164, 103) ],
    [ Score(154, 96), Score(179,166), Score(105, 199), Score( 70, 199), Score( 70, 199), Score(105, 199), Score(179, 166), Score(154,  96) ],
    [ Score(123, 92), Score(145,172), Score( 81, 184), Score( 31, 191), Score( 31, 191), Score( 81, 184), Score(145, 172), Score(123,  92) ],
    [ Score( 88, 47), Score(120,121), Score( 65, 116), Score( 33, 131), Score( 33, 131), Score( 65, 116), Score(120, 121), Score( 88,  47) ],
    [ Score( 59, 11), Score( 89, 59), Score( 45,  73), Score( -1,  78), Score( -1,  78), Score( 45,  73), Score( 89,  59), Score( 59,  11) ],
];

static KING_ATTACKERS_WEIGHT: [i32; 7] = [0, 50, 75, 88, 94, 97, 99];

// Based on distance from the edge of the board, i.e. in the following order H file, G file, F file, E file
// Based on the rank, rank 1 is the bonus when there are no pawns sheltering the king
static KING_PAWN_DEFENDER_BONUS: [[Score; 7]; 4] = [
    // Pushing the pawn to H3 is fine, pushing it further is not great
    [
        Score(-5, 0),
        Score(75, 0),
        Score(75, 0),
        Score(50, 0),
        Score(40, 0),
        Score(30, 0),
        Score(35, 0),
    ],
    // Not having a G-pawn is terrible, pushing it to g3 is not terrible, pushing it to g4 or further is bad
    [
        Score(-50, 0),
        Score(75, 0),
        Score(30, 0),
        Score(-50, 0),
        Score(-40, 0),
        Score(-30, 0),
        Score(-20, 0),
    ],
    // Not having an F-pawn is not horrible, pushing it isn't too terrible
    [
        Score(-5, 0),
        Score(75, 0),
        Score(30, 0),
        Score(0, 0),
        Score(0, 0),
        Score(0, 0),
        Score(0, 0),
    ],
    // If the king is on the F-file, it doesn't quite matter where the E-pawn is, the king should castle
    [
        Score(-40, 0),
        Score(-12, 0),
        Score(-30, 0),
        Score(-50, 0),
        Score(-50, 0),
        Score(-50, 0),
        Score(-50, 0),
    ],
];

// Bonus for a passed pawn being on a certain rank
static PASSED_PAWN_BONUS: [Score; 7] = [
    Score(0, 0),
    Score(10, 0),
    Score(20, 0),
    Score(30, 0),
    Score(65, 0),
    Score(90, 0),
    Score(130, 0),
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
    static ref MOBILITY_BONUS: [[Score; 28]; 6] = {
        let mut table = [[Score(0, 0); 28]; 6];

        // Knight mobility
        let knight_scores = [
            Score(-62,-79), Score(-53,-57), Score(-12,-31), Score( -3,-17), Score(  3,  7), Score( 12, 13),
            Score( 21, 16), Score( 28, 21), Score( 37, 26)
        ];

        let bishop_scores = [ Score(-47,-59), Score(-20,-25), Score( 14, -8), Score( 29, 12), Score( 39, 21), Score( 53, 40),
            Score( 53, 56), Score( 60, 58), Score( 62, 65), Score( 69, 72), Score( 78, 78), Score( 83, 87),
            Score( 91, 88), Score( 96, 98)
        ];

        let rook_scores = [ Score(-60,-82), Score(-24,-15), Score(  0, 17) ,Score(  3, 43), Score(  4, 72), Score( 14,100),
            Score( 20,102), Score( 30,122), Score( 41,133), Score(41 ,139), Score( 41,153), Score( 45,160),
            Score( 57,165), Score( 58,170), Score( 67,175)
        ];
        let queen_scores = [ Score(-29,-49), Score(-16,-29), Score( -8, -8), Score( -8, 17), Score( 18, 39), Score( 25, 54),
            Score( 23, 59), Score( 37, 73), Score( 41, 76), Score( 54, 95), Score( 65, 95) ,Score( 68,101),
            Score( 69,124), Score( 70,128), Score( 70,132), Score( 70,133) ,Score( 71,136), Score( 72,140),
            Score( 74,147), Score( 76,149), Score( 90,153), Score(104,169), Score(105,171), Score(106,171),
            Score(112,178), Score(114,185), Score(114,187), Score(119,221)
        ];


        table[PieceType::Knight as usize] = [knight_scores[knight_scores.len() - 1]; 28];
        for (i, score) in knight_scores.iter().enumerate() {
            table[PieceType::Knight as usize][i] = *score;
        }

        table[PieceType::Bishop as usize] = [bishop_scores[bishop_scores.len() - 1]; 28];
        for (i, score) in bishop_scores.iter().enumerate() {
            table[PieceType::Bishop as usize][i] = *score;
        }

        table[PieceType::Rook as usize] = [rook_scores[rook_scores.len() - 1]; 28];
        for (i, score) in rook_scores.iter().enumerate() {
            table[PieceType::Rook as usize][i] = *score;
        }

        table[PieceType::Queen as usize] = [queen_scores[queen_scores.len() - 1]; 28];
        for (i, score) in queen_scores.iter().enumerate() {
            table[PieceType::Queen as usize][i] = *score;
        }

        table
    };


    static ref KING_ATTACKERS_PENALTY: [Score; 6] = {
        let mut table: [Score; 6] = [Score(0, 0); 6];
        table[PieceType::Bishop as usize] =  Score(-20, 0);
        table[PieceType::Knight as usize] =  Score(-20, 0);
        table[PieceType::Rook as usize] =  Score(-40, 0);
        table[PieceType::Queen as usize] =  Score(-80, 0);
        table
    };
}

pub fn get_raw_piece_value(pt: PieceType) -> Score {
    match pt {
        PieceType::Pawn => Score(126, 208),
        PieceType::Knight => Score(781, 854),
        PieceType::Bishop => Score(825, 915),
        PieceType::Rook => Score(1276, 1380),
        PieceType::Queen => Score(2538, 2682),
        PieceType::King => Score(0, 0),
    }
}

fn get_piece_positional_value(color: Color, piece_type: PieceType, coord: Coordinate) -> Score {
    PIECE_POSITIONAL_VALUES[color as usize][piece_type as usize][coord.get_rank() - 1]
        [coord.get_file() - 1]
}

fn get_piece_value(color: Color, piece_type: PieceType, coord: Coordinate) -> Score {
    let val =
        get_raw_piece_value(piece_type) + get_piece_positional_value(color, piece_type, coord);

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

fn calculate_mobility_area(board: &Board) -> [Bitboard; 2] {
    let mut area = [0; 2];

    for color in [Color::White, Color::Black] {
        let low_ranks = if color.is_white() {
            RANK_2_BB | RANK_3_BB
        } else {
            RANK_6_BB | RANK_7_BB
        };
        // Get pawns that are either obstructed by another piece or are
        // on the lower ranks
        let shifted_pieces = if color.is_white() {
            board.get_all_pieces_bb().wrapping_shr(8)
        } else {
            board.get_all_pieces_bb().wrapping_shl(8)
        };
        let b = board.get_piece_type_bb_for_color(PieceType::Pawn, color)
            & (low_ranks | shifted_pieces);

        // Squares that are not occupied by blocked pawns, pinned pieces and kings/queens
        // are considered to make up the mobility area
        // These squares should also not be attacked by an enemy pawn.
        area[color as usize] = !(b
            | board.get_piece_types_bb_for_color(PieceType::King, PieceType::Queen, color)
            | board.get_king_shields(color)
            | get_pawn_attacks_from_bb(
                board.get_piece_type_bb_for_color(PieceType::Pawn, color.other_color()),
                color.other_color(),
            ));
    }

    area
}

pub fn evaluate_board(board: &Board) -> i32 {
    let mut score = Score(0, 0);

    let phase = calculate_phase(board);

    for (i, piece) in board.get_pieces().iter().enumerate() {
        if let Some(piece) = piece {
            let coord = Coordinate::try_from(i).unwrap();
            score += get_piece_value(piece.color, piece.piece_type, coord);
        }
    }

    // Calculate piece mobility bonuses
    let mobility_area = calculate_mobility_area(board);
    for pt in [
        PieceType::Knight,
        PieceType::Bishop,
        PieceType::Rook,
        PieceType::Queen,
    ] {
        score += calculate_piece_type_mobility(
            board,
            pt,
            Color::White,
            mobility_area[Color::White as usize],
        ) - calculate_piece_type_mobility(
            board,
            pt,
            Color::Black,
            mobility_area[Color::Black as usize],
        );
    }

    // Calculate king safety
    score +=
        calculate_king_safety(board, Color::White) - calculate_king_safety(board, Color::Black);

    // Calculate passed pawns bonus
    score += calculate_passed_pawns_bonus(board, Color::White)
        - calculate_passed_pawns_bonus(board, Color::Black);

    let midgame_score = score.get_for_phase(Phase::Midgame);
    let endgame_score = score.get_for_phase(Phase::Endgame);

    (midgame_score * phase + (endgame_score * (MIDGAME_SCALE - phase))) / MIDGAME_SCALE
}

// Finds the piece with the least value that is attacking
// a square. Returns the piece and its source square
fn get_smallest_attacker(board: &Board, square: Coordinate) -> Option<(Piece, Coordinate)> {
    let color = board.get_player_color();
    let attacks = get_attackers_of_square_bb(board, square, color, board.get_all_pieces_bb());

    // In ascending order of piece value
    for pt in [
        PieceType::Pawn,
        PieceType::Knight,
        PieceType::Bishop,
        PieceType::Rook,
        PieceType::Queen,
        PieceType::King,
    ] {
        let attacker_square = attacks & board.get_piece_type_bb_for_color(pt, color);
        if attacker_square != 0 {
            return Some((
                Piece::new(color, pt),
                Coordinate::from_bb(lsb(attacker_square)),
            ));
        }
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
        return get_raw_piece_value(victim_piece_type).get_for_phase(Phase::Midgame)
            - static_exchange_evaluation(board, square);
    } else {
        return 0;
    }
}

pub fn static_exchange_evaluation_capture(mut board: Board, m: &Move) -> i32 {
    // TODO: Deal with en passant
    if let Some(victim_piece) = board.get_from_coordinate(m.dest) {
        board.apply_move(m);
        return get_raw_piece_value(victim_piece.piece_type).get_for_phase(Phase::Midgame)
            - static_exchange_evaluation(board, m.dest);
    } else {
        return 0;
    }
}

pub fn non_pawn_material(board: &Board) -> i32 {
    let mut val = 0;
    for piece in board.get_pieces() {
        if let Some(piece) = piece {
            if piece.piece_type != PieceType::Pawn {
                val += get_raw_piece_value(piece.piece_type).get_for_phase(Phase::Midgame);
            }
        }
    }
    val
}

fn get_piece_mobility_score(num: usize, pt: PieceType) -> Score {
    MOBILITY_BONUS[pt as usize][num]
}

// Supported piece types: Knight, Bishop, Rook, Queen.
fn calculate_piece_type_mobility(
    board: &Board,
    piece_type: PieceType,
    color: Color,
    mobility_area: Bitboard,
) -> Score {
    let mut score = Score(0, 0);
    let mut squares_with_pieces = board.get_piece_type_bb_for_color(piece_type, color);
    while squares_with_pieces != 0 {
        let (curr_sq, popped_pieces) = pop_lsb(squares_with_pieces);
        squares_with_pieces = popped_pieces;

        let mut attacked_squares = match piece_type {
            // Take into account batteries and x-rays
            PieceType::Bishop => get_sliding_attacks_occupied(
                PieceType::Bishop,
                Coordinate::from_bb(curr_sq),
                board.get_all_pieces_bb() ^ board.get_piece_type_bb(PieceType::Queen),
            ),
            // Take into account stacked rooks and queens
            PieceType::Rook => get_sliding_attacks_occupied(
                PieceType::Rook,
                Coordinate::from_bb(curr_sq),
                board.get_all_pieces_bb()
                    ^ board.get_piece_type_bb(PieceType::Queen)
                    ^ board.get_piece_type_bb_for_color(PieceType::Queen, color),
            ),
            PieceType::Queen => get_sliding_attacks_occupied(
                PieceType::Queen,
                Coordinate::from_bb(curr_sq),
                board.get_all_pieces_bb(),
            ),
            PieceType::Knight => {
                get_piece_attacks_bb(PieceType::Knight, Coordinate::from_bb(curr_sq))
            }
            _ => panic!("Unexpected piece type: {:?}", piece_type),
        };

        // Pinned pieces may only move without leaving the defense of the king
        if board.get_king_shields(color) & curr_sq != 0 {
            attacked_squares &= edge_to_edge_bb(
                board.get_king_coordinate(color).expect("Missing king"),
                Coordinate::from_bb(curr_sq),
            );
        }

        score += get_piece_mobility_score(
            (mobility_area & attacked_squares).count_ones() as usize,
            piece_type,
        );
    }

    score
}

fn calculate_king_safety(board: &Board, color: Color) -> Score {
    calculate_king_attackers_penalty(board, color) + calculate_king_pawn_formation(board, color)
}

// Inspired by https://www.chessprogramming.org/King_Safety - Attacking King Zone
fn calculate_king_attackers_penalty(board: &Board, color: Color) -> Score {
    let mut num_attackers = 0;
    let mut score = Score(0, 0);

    if let Some(king_coord) = board.get_king_coordinate(color) {
        // The 8 squares around the king
        let king_ring = get_piece_attacks_bb(PieceType::King, king_coord);

        for piece_type in [
            PieceType::Knight,
            PieceType::Bishop,
            PieceType::Rook,
            PieceType::Queen,
        ] {
            let mut squares_with_pieces =
                board.get_piece_type_bb_for_color(piece_type, color.other_color());
            while squares_with_pieces != 0 {
                let (curr_sq, popped_pieces) = pop_lsb(squares_with_pieces);
                squares_with_pieces = popped_pieces;

                let attacked_squares = match piece_type {
                    // Take into account batteries and x-rays
                    PieceType::Bishop => get_sliding_attacks_occupied(
                        PieceType::Bishop,
                        Coordinate::from_bb(curr_sq),
                        board.get_all_pieces_bb(),
                    ),
                    // Take into account stacked rooks and queens
                    PieceType::Rook => get_sliding_attacks_occupied(
                        PieceType::Rook,
                        Coordinate::from_bb(curr_sq),
                        board.get_all_pieces_bb(),
                    ),
                    PieceType::Queen => get_sliding_attacks_occupied(
                        PieceType::Queen,
                        Coordinate::from_bb(curr_sq),
                        board.get_all_pieces_bb(),
                    ),
                    PieceType::Knight => {
                        get_piece_attacks_bb(PieceType::Knight, Coordinate::from_bb(curr_sq))
                    }
                    _ => panic!("Unexpected piece type: {:?}", piece_type),
                } & king_ring;

                if attacked_squares != 0 {
                    num_attackers += 1;
                    score += get_king_attackers_penalty(piece_type)
                        * attacked_squares.count_ones() as i32;
                }
            }
        }

        let weight = KING_ATTACKERS_WEIGHT[num_attackers.min(6)] as f64 / 100.0;
        score = Score(
            (weight * score.0 as f64) as i32,
            (weight * score.1 as f64) as i32,
        );
    }

    score
}

// Assign bonuses and penalties based on the pawn formation in front
// of the king
fn calculate_king_pawn_formation(board: &Board, color: Color) -> Score {
    let mut score = Score(0, 0);

    if let Some(king_coord) = board.get_king_coordinate(color) {
        // We take into the king's file along with the two files by its side
        // We also handle the case where the king is on the A or H file, in which
        // case we assume that the king is on the B or G file to cover all files
        // in front of the king.
        let center_file = king_coord
            .get_file()
            .min(7) // G file
            .max(2); // B file
                     // Get pawns that are 'in front' of the king
        let defending_pawns = board.get_piece_type_bb_for_color(PieceType::Pawn, color)
            & !get_forward_ranks_bb_for_color(king_coord.get_rank(), color.other_color());

        for file in (center_file - 1)..=(center_file + 1) {
            let file_bb = FILES_BB[file - 1];
            let file_defenders = defending_pawns & file_bb;
            // Get the pawn that is closest to the king, we want the lowest bit,
            // which is why we use the opposite color.
            let closest_defender = frontmost_bit(file_defenders, color.other_color());

            // We get a relative rank, so that we can use the same lookup tables based
            // on the distance between the pawn and the king.
            let rel_defender_rank = if closest_defender != 0 {
                relative_rank(Coordinate::from_bb(closest_defender).get_rank(), color) - 1
            } else {
                0 // The 0th index contains the score when there is no pawn sheltering the king
            };
            // We use a symmetric table for both sides of the board
            let distance_to_edge = dist_from_edge(file);

            score += KING_PAWN_DEFENDER_BONUS[distance_to_edge][rel_defender_rank];
        }
    }
    score
}

fn calculate_passed_pawns_bonus(board: &Board, color: Color) -> Score {
    let mut score = Score(0, 0);
    let mut our_pawns = board.get_piece_type_bb_for_color(PieceType::Pawn, color);
    let enemy_pawns = board.get_piece_type_bb_for_color(PieceType::Pawn, color.other_color());
    let enemy_pawns_attack = get_pawn_attacks_bb_for_bitboard(color.other_color(), enemy_pawns);

    while our_pawns != 0 {
        let (current_pawn, popped_pawns) = pop_lsb(our_pawns);
        let current_pawn_coordinate = Coordinate::from_bb(current_pawn);
        our_pawns = popped_pawns;

        let pawn_path = get_pawn_front_squares(current_pawn_coordinate, color);

        // Identify if a current pawn is a passed pawn by checking if the
        // squares in its path are occupied or attacked by enemy pawns
        if (pawn_path & (enemy_pawns | enemy_pawns_attack)) == 0 {
            let rel_rank = relative_rank(current_pawn_coordinate.get_rank(), color);
            score += PASSED_PAWN_BONUS[rel_rank - 1];
        }
    }
    score
}

fn get_king_attackers_penalty(pt: PieceType) -> Score {
    KING_ATTACKERS_PENALTY[pt as usize]
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
                get_piece_positional_value(*color, PieceType::Pawn, *coord).get_for_phase(*phase),
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
                get_piece_positional_value(*color, PieceType::Knight, *coord).get_for_phase(*phase),
                *eval
            );
        }
    }

    #[test]
    fn test_get_bishop_positional_value() {
        let test_cases = [
            (Color::White, Coordinate::G2, 6, Phase::Midgame),
            (Color::White, Coordinate::A8, -34, Phase::Midgame),
            (Color::White, Coordinate::D3, 12, Phase::Midgame),
            (Color::White, Coordinate::C4, 18, Phase::Midgame),
            (Color::Black, Coordinate::B8, -4, Phase::Midgame),
            (Color::Black, Coordinate::E5, 27, Phase::Midgame),
            (Color::White, Coordinate::C4, 0, Phase::Endgame),
            (Color::Black, Coordinate::G1, -29, Phase::Endgame),
        ];

        for (color, coord, eval, phase) in test_cases.iter() {
            assert_eq!(
                get_piece_positional_value(*color, PieceType::Bishop, *coord).get_for_phase(*phase),
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
                get_piece_positional_value(*color, PieceType::Rook, *coord).get_for_phase(*phase),
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
                get_piece_positional_value(*color, PieceType::Queen, *coord).get_for_phase(*phase),
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
                get_piece_positional_value(*color, PieceType::King, *coord).get_for_phase(*phase),
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
            board.update_board_state();
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
            board.update_board_state();
        }
        assert_eq!(
            static_exchange_evaluation(board, Coordinate::D5),
            get_raw_piece_value(PieceType::Pawn).get_for_phase(Phase::Midgame)
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
            get_raw_piece_value(PieceType::Pawn).get_for_phase(Phase::Midgame)
                - get_raw_piece_value(PieceType::Knight).get_for_phase(Phase::Midgame)
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
            get_raw_piece_value(PieceType::Knight).get_for_phase(Phase::Midgame)
        );
    }

    #[test]
    fn static_exchange_evaluation_with_battery_queen_behind() {
        let mut board = Board::new_empty();
        board.set_player_color(Color::White);
        let pieces = [
            (PieceType::Queen, Coordinate::A1, Color::White),
            (PieceType::Bishop, Coordinate::B2, Color::White),
            (PieceType::Pawn, Coordinate::D4, Color::White),
            (PieceType::Pawn, Coordinate::E5, Color::Black),
            (PieceType::Rook, Coordinate::E8, Color::Black),
            (PieceType::Bishop, Coordinate::H8, Color::Black),
        ];
        for (p, coord, color) in pieces {
            board.place_piece(coord, Piece::new(color, p));
        }
        assert_eq!(
            static_exchange_evaluation(board, Coordinate::E5),
            get_raw_piece_value(PieceType::Rook).get_for_phase(Phase::Midgame)
        );
    }

    #[test]
    fn static_exchange_evaluation_with_battery_bishop_behind() {
        let mut board = Board::new_empty();
        board.set_player_color(Color::White);
        let pieces = [
            (PieceType::Queen, Coordinate::B2, Color::White),
            (PieceType::Bishop, Coordinate::A1, Color::White),
            (PieceType::Pawn, Coordinate::D4, Color::White),
            (PieceType::Pawn, Coordinate::E5, Color::Black),
            (PieceType::Rook, Coordinate::E8, Color::Black),
            (PieceType::Bishop, Coordinate::H8, Color::Black),
        ];
        for (p, coord, color) in pieces {
            board.place_piece(coord, Piece::new(color, p));
        }
        assert_eq!(
            static_exchange_evaluation(board, Coordinate::E5),
            get_raw_piece_value(PieceType::Rook).get_for_phase(Phase::Midgame)
                + get_raw_piece_value(PieceType::Bishop).get_for_phase(Phase::Midgame)
                - get_raw_piece_value(PieceType::Queen).get_for_phase(Phase::Midgame)
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
            get_raw_piece_value(PieceType::Queen).get_for_phase(Phase::Midgame)
                + get_raw_piece_value(PieceType::Rook).get_for_phase(Phase::Midgame)
                + get_raw_piece_value(PieceType::Bishop).get_for_phase(Phase::Midgame)
                + get_raw_piece_value(PieceType::Knight).get_for_phase(Phase::Midgame)
        )
    }

    #[test]
    fn evaluate_piece_mobility_for_bishop() {
        let mut board = Board::new_empty();
        let pieces = [
            (PieceType::Bishop, Coordinate::A1),
            (PieceType::Queen, Coordinate::B2),
        ];
        for (p, coord) in pieces {
            board.place_piece(coord, Piece::new(Color::White, p));
        }

        // println!(
        //     "{}",
        //     board.get_all_pieces_bb() ^ board.get_piece_type_bb(PieceType::Queen)
        // );
    }

    #[test]
    fn mobility_area_empty_board() {
        let mut board = Board::new_empty();
        let pieces = [];
        for (p, coord) in pieces {
            board.place_piece(coord, Piece::new(Color::White, p));
        }
        board.init();
        // Full mobility on an empty board
        assert_eq!(
            calculate_mobility_area(&board),
            [0xffffffffffffffff, 0xffffffffffffffff]
        );
    }

    #[test]
    fn mobility_area_king_queen() {
        let mut board = Board::new_empty();
        let pieces = [
            (PieceType::King, Coordinate::C2),
            (PieceType::Queen, Coordinate::H5),
        ];
        for (p, coord) in pieces {
            board.place_piece(coord, Piece::new(Color::White, p));
        }
        board.init();

        // White mobility area does not include where the king and queen area
        let white_mobility = 0xffffffffffffffff ^ Coordinate::C2.to_bb() ^ Coordinate::H5.to_bb();
        let black_mobility = 0xffffffffffffffff;
        assert_eq!(
            calculate_mobility_area(&board),
            [white_mobility, black_mobility]
        );
    }

    #[test]
    fn mobility_area_with_pinned_pieces() {
        let mut board = Board::new_empty();
        let pieces = [
            (PieceType::King, Coordinate::C2, Color::White),
            (PieceType::Bishop, Coordinate::C3, Color::White),
            (PieceType::Queen, Coordinate::C8, Color::Black),
        ];
        for (p, coord, color) in pieces {
            board.place_piece(coord, Piece::new(color, p));
        }
        board.init();

        // White mobility area does not include where the king and its defender are
        let white_mobility = 0xffffffffffffffff ^ Coordinate::C2.to_bb() ^ Coordinate::C3.to_bb();
        let black_mobility = 0xffffffffffffffff ^ Coordinate::C8.to_bb();
        assert_eq!(
            calculate_mobility_area(&board),
            [white_mobility, black_mobility]
        );
    }

    #[test]
    fn mobility_area_with_squares_controlled_enemy_pawns_for_white() {
        let mut board = Board::new_empty();
        let pieces = [
            (PieceType::Pawn, Coordinate::D4, Color::Black),
            (PieceType::Pawn, Coordinate::C3, Color::Black),
        ];
        for (p, coord, color) in pieces {
            board.place_piece(coord, Piece::new(color, p));
        }
        board.init();

        // White mobility area does not include squares that are controlled by enemy pawns
        let white_mobility = 0xffffffffffffffff
            ^ Coordinate::B2.to_bb()
            ^ Coordinate::D2.to_bb()
            ^ Coordinate::C3.to_bb()
            ^ Coordinate::E3.to_bb();
        let black_mobility = 0xffffffffffffffff;

        assert_eq!(
            calculate_mobility_area(&board),
            [white_mobility, black_mobility]
        );
    }

    #[test]
    fn mobility_area_with_squares_controlled_enemy_pawns_for_black() {
        let mut board = Board::new_empty();
        let pieces = [
            (PieceType::Pawn, Coordinate::D6, Color::White),
            (PieceType::Pawn, Coordinate::C7, Color::White),
        ];
        for (p, coord, color) in pieces {
            board.place_piece(coord, Piece::new(color, p));
        }
        board.init();

        // Mobility area does not include squares that are controlled by enemy pawns
        let white_mobility = 0xffffffffffffffff;
        let black_mobility = 0xffffffffffffffff
            ^ Coordinate::B8.to_bb()
            ^ Coordinate::D8.to_bb()
            ^ Coordinate::C7.to_bb()
            ^ Coordinate::E7.to_bb();

        assert_eq!(
            calculate_mobility_area(&board),
            [white_mobility, black_mobility]
        );
    }

    #[test]
    fn mobility_area_with_blocked_pawns_for_white() {
        let mut board = Board::new_empty();
        let pieces = [
            (PieceType::Pawn, Coordinate::E4, Color::White),
            (PieceType::Pawn, Coordinate::E5, Color::White),
        ];
        for (p, coord, color) in pieces {
            board.place_piece(coord, Piece::new(color, p));
        }
        board.init();

        // Mobility area does not include pawns that are obstructed
        let white_mobility = 0xffffffffffffffff ^ Coordinate::E4.to_bb();
        let black_mobility = 0xffffffffffffffff
            ^ Coordinate::D5.to_bb()
            ^ Coordinate::F5.to_bb()
            ^ Coordinate::D6.to_bb()
            ^ Coordinate::F6.to_bb();

        assert_eq!(
            calculate_mobility_area(&board),
            [white_mobility, black_mobility]
        );
    }

    #[test]
    fn mobility_area_with_blocked_pawns_for_black() {
        let mut board = Board::new_empty();
        let pieces = [
            (PieceType::Pawn, Coordinate::E4, Color::Black),
            (PieceType::Pawn, Coordinate::E5, Color::Black),
        ];
        for (p, coord, color) in pieces {
            board.place_piece(coord, Piece::new(color, p));
        }
        board.init();

        // Mobility area does not include pawns that are obstructed
        let white_mobility = 0xffffffffffffffff
            ^ Coordinate::D3.to_bb()
            ^ Coordinate::F3.to_bb()
            ^ Coordinate::D4.to_bb()
            ^ Coordinate::F4.to_bb();
        let black_mobility = 0xffffffffffffffff ^ Coordinate::E5.to_bb();

        assert_eq!(
            calculate_mobility_area(&board),
            [white_mobility, black_mobility]
        );
    }

    #[test]
    fn mobility_area_with_paws_on_low_ranks_white() {
        let mut board = Board::new_empty();
        let pieces = [
            (PieceType::Pawn, Coordinate::D2, Color::White),
            (PieceType::Pawn, Coordinate::E3, Color::White),
            (PieceType::Pawn, Coordinate::F4, Color::White),
        ];
        for (p, coord, color) in pieces {
            board.place_piece(coord, Piece::new(color, p));
        }
        board.init();

        // Mobility area does not include pawns that are on low ranks
        let white_mobility = 0xffffffffffffffff ^ Coordinate::D2.to_bb() ^ Coordinate::E3.to_bb();
        let black_mobility = 0xffffffffffffffff
            ^ Coordinate::C3.to_bb()
            ^ Coordinate::E3.to_bb()
            ^ Coordinate::D4.to_bb()
            ^ Coordinate::F4.to_bb()
            ^ Coordinate::E5.to_bb()
            ^ Coordinate::G5.to_bb();

        assert_eq!(
            calculate_mobility_area(&board),
            [white_mobility, black_mobility]
        );
    }

    #[test]
    fn mobility_area_with_paws_on_low_ranks_black() {
        let mut board = Board::new_empty();
        let pieces = [
            (PieceType::Pawn, Coordinate::D7, Color::Black),
            (PieceType::Pawn, Coordinate::E6, Color::Black),
            (PieceType::Pawn, Coordinate::F5, Color::Black),
        ];
        for (p, coord, color) in pieces {
            board.place_piece(coord, Piece::new(color, p));
        }
        board.init();

        // Mobility area does not include pawns that are on low ranks
        let white_mobility = 0xffffffffffffffff
            ^ Coordinate::C6.to_bb()
            ^ Coordinate::E6.to_bb()
            ^ Coordinate::D5.to_bb()
            ^ Coordinate::F5.to_bb()
            ^ Coordinate::E4.to_bb()
            ^ Coordinate::G4.to_bb();
        let black_mobility = 0xffffffffffffffff ^ Coordinate::D7.to_bb() ^ Coordinate::E6.to_bb();

        assert_eq!(
            calculate_mobility_area(&board),
            [white_mobility, black_mobility]
        );
    }
}

#[cfg(test)]
mod king_safety {
    use super::*;

    #[test]
    fn rook_attacks_white_king() {
        let mut board = Board::new_empty();
        let pieces = [
            (PieceType::Rook, Coordinate::F8, Color::Black),
            (PieceType::King, Coordinate::G1, Color::White),
        ];
        for (p, coord, color) in pieces {
            board.place_piece(coord, Piece::new(color, p));
        }

        let score = calculate_king_attackers_penalty(&board, Color::White);

        // 1 attacker, 2 squares
        assert_eq!(score, Score(-(2.0 * 0.5 * 40.0) as i32, 0));
    }

    #[test]
    fn knight_attacks_white_king() {
        let mut board = Board::new_empty();
        let pieces = [
            (PieceType::Knight, Coordinate::G3, Color::Black),
            (PieceType::King, Coordinate::G1, Color::White),
        ];
        for (p, coord, color) in pieces {
            board.place_piece(coord, Piece::new(color, p));
        }

        let score = calculate_king_attackers_penalty(&board, Color::White);

        // 1 attacker, 2 squares
        assert_eq!(score, Score(-(2.0 * 0.5 * 20.0) as i32, 0));
    }

    #[test]
    fn bishop_queen_attack_white_king() {
        let mut board = Board::new_empty();
        let pieces = [
            (PieceType::Queen, Coordinate::H3, Color::Black),
            (PieceType::Bishop, Coordinate::F3, Color::Black),
            (PieceType::King, Coordinate::G1, Color::White),
        ];
        for (p, coord, color) in pieces {
            board.place_piece(coord, Piece::new(color, p));
        }

        let score = calculate_king_attackers_penalty(&board, Color::White);

        // 2 attacker, Bishop hits 2 squares, Queen hits 4 squares
        assert_eq!(score, Score(-(0.75 * (2.0 * 20.0 + 4.0 * 80.0)) as i32, 0));
    }

    #[test]
    fn no_half_open_files_white_king_on_g1() {
        let mut board = Board::new_empty();
        let pieces = [
            (PieceType::Pawn, Coordinate::F2, Color::White),
            (PieceType::Pawn, Coordinate::G2, Color::White),
            (PieceType::Pawn, Coordinate::H2, Color::White),
            (PieceType::King, Coordinate::G1, Color::White),
        ];
        for (p, coord, color) in pieces {
            board.place_piece(coord, Piece::new(color, p));
        }

        assert_eq!(
            calculate_king_pawn_formation(&board, Color::White),
            Score(75 + 75 + 75, 0)
        );
    }

    #[test]
    fn missing_g_pawn_white_king_on_g1() {
        let mut board = Board::new_empty();
        let pieces = [
            (PieceType::Pawn, Coordinate::F2, Color::White),
            (PieceType::Pawn, Coordinate::H2, Color::White),
            (PieceType::King, Coordinate::G1, Color::White),
        ];
        for (p, coord, color) in pieces {
            board.place_piece(coord, Piece::new(color, p));
        }

        assert_eq!(
            calculate_king_pawn_formation(&board, Color::White),
            Score(75 - 50 + 75, 0)
        );
    }

    #[test]
    fn missing_g_pawn_white_king_on_h1() {
        let mut board = Board::new_empty();
        let pieces = [
            (PieceType::Pawn, Coordinate::F2, Color::White),
            (PieceType::Pawn, Coordinate::H2, Color::White),
            (PieceType::King, Coordinate::H1, Color::White),
        ];
        for (p, coord, color) in pieces {
            board.place_piece(coord, Piece::new(color, p));
        }

        // Same score as being on g1
        assert_eq!(
            calculate_king_pawn_formation(&board, Color::White),
            Score(75 - 50 + 75, 0)
        );
    }

    #[test]
    fn pawn_on_g4_h5_white_king_on_g1() {
        let mut board = Board::new_empty();
        let pieces = [
            (PieceType::Pawn, Coordinate::F2, Color::White),
            (PieceType::Pawn, Coordinate::G4, Color::White),
            (PieceType::Pawn, Coordinate::H5, Color::White),
            (PieceType::King, Coordinate::G1, Color::White),
        ];
        for (p, coord, color) in pieces {
            board.place_piece(coord, Piece::new(color, p));
        }

        assert_eq!(
            calculate_king_pawn_formation(&board, Color::White),
            Score(40 - 50 + 75, 0)
        );
    }

    #[test]
    fn no_half_open_files_black_king_on_g8() {
        let mut board = Board::new_empty();
        let pieces = [
            (PieceType::Pawn, Coordinate::F7, Color::Black),
            (PieceType::Pawn, Coordinate::G7, Color::Black),
            (PieceType::Pawn, Coordinate::H7, Color::Black),
            (PieceType::King, Coordinate::G8, Color::Black),
        ];
        for (p, coord, color) in pieces {
            board.place_piece(coord, Piece::new(color, p));
        }

        assert_eq!(
            calculate_king_pawn_formation(&board, Color::Black),
            Score(75 + 75 + 75, 0)
        );
    }

    #[test]
    fn missing_b_and_c_pawns_black_king_on_b8() {
        let mut board = Board::new_empty();
        let pieces = [
            (PieceType::Pawn, Coordinate::A7, Color::Black),
            (PieceType::King, Coordinate::B8, Color::Black),
        ];
        for (p, coord, color) in pieces {
            board.place_piece(coord, Piece::new(color, p));
        }

        assert_eq!(
            calculate_king_pawn_formation(&board, Color::Black),
            Score(75 - 50 - 5, 0)
        );
    }

    #[test]
    fn fianchettoed_black_king_on_b8() {
        let mut board = Board::new_empty();
        let pieces = [
            (PieceType::Pawn, Coordinate::A7, Color::Black),
            (PieceType::Pawn, Coordinate::B6, Color::Black),
            (PieceType::Pawn, Coordinate::C7, Color::Black),
            (PieceType::King, Coordinate::B8, Color::Black),
        ];
        for (p, coord, color) in pieces {
            board.place_piece(coord, Piece::new(color, p));
        }

        assert_eq!(
            calculate_king_pawn_formation(&board, Color::Black),
            Score(75 + 30 + 75, 0)
        );
    }

    #[test]
    fn fianchettoed_black_king_on_b7() {
        let mut board = Board::new_empty();
        let pieces = [
            (PieceType::Pawn, Coordinate::A7, Color::Black),
            (PieceType::Pawn, Coordinate::B6, Color::Black),
            (PieceType::Pawn, Coordinate::C7, Color::Black),
            (PieceType::King, Coordinate::B7, Color::Black),
        ];
        for (p, coord, color) in pieces {
            board.place_piece(coord, Piece::new(color, p));
        }

        assert_eq!(
            calculate_king_pawn_formation(&board, Color::Black),
            Score(75 + 30 + 75, 0)
        );
    }

    #[test]
    fn passed_pawn_bonus_white() {
        let mut board = Board::new_empty();
        let pieces = [(PieceType::Pawn, Coordinate::A5, Color::White)];

        for (p, coord, color) in pieces {
            board.place_piece(coord, Piece::new(color, p));
        }

        assert_eq!(
            calculate_passed_pawns_bonus(&board, Color::White),
            PASSED_PAWN_BONUS[5 - 1]
        );
    }

    #[test]
    fn non_passed_pawn_by_obstruction_white() {
        let mut board = Board::new_empty();
        let pieces = [
            (PieceType::Pawn, Coordinate::A5, Color::White),
            (PieceType::Pawn, Coordinate::A6, Color::Black),
        ];

        for (p, coord, color) in pieces {
            board.place_piece(coord, Piece::new(color, p));
        }

        assert_eq!(
            calculate_passed_pawns_bonus(&board, Color::White),
            Score(0, 0)
        );
        assert_eq!(
            calculate_passed_pawns_bonus(&board, Color::Black),
            Score(0, 0)
        );
    }

    #[test]
    fn non_passed_pawn_by_attacks_white() {
        let mut board = Board::new_empty();
        let pieces = [
            (PieceType::Pawn, Coordinate::A5, Color::White),
            (PieceType::Pawn, Coordinate::B7, Color::Black),
        ];

        for (p, coord, color) in pieces {
            board.place_piece(coord, Piece::new(color, p));
        }

        assert_eq!(
            calculate_passed_pawns_bonus(&board, Color::White),
            Score(0, 0)
        );
        assert_eq!(
            calculate_passed_pawns_bonus(&board, Color::Black),
            Score(0, 0)
        );
    }
}
