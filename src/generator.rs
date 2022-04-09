use crate::bitboard::*;
use crate::board::{Board, Color, Coordinate, PieceType};
use crate::r#move::Move;

use std::convert::TryFrom;

// Generate all legal moves for a particular pawn
fn generate_pawn_moves(board: &Board, src: Coordinate) -> Vec<Move> {
    let piece = board.get_from_coordinate(src).unwrap();
    let piece_color = piece.color;
    let front_square = src.vertical_offset(1, piece_color.is_white());
    let mut res = vec![];

    // Handle normal captures
    for front_side_square in front_square.side_squares() {
        match board.get_from_coordinate(front_side_square) {
            Some(p) => {
                if p.color == piece.color.other_color() {
                    let mut capture_move =
                        Move::new_capture(src, front_side_square, piece, p.piece_type);

                    // Handle possible promotion by capturing
                    if piece_color.is_white() && front_square.is_in_rank(8)
                        || !piece_color.is_white() && front_square.is_in_rank(1)
                    {
                        capture_move.is_promotion = true;
                        for promotable_piece_type in PieceType::promotable_piece_types() {
                            let mut promotion_move = capture_move.clone();
                            promotion_move.promotes_to = Some(promotable_piece_type);
                            res.push(promotion_move);
                        }
                    } else {
                        res.push(capture_move);
                    }
                }
            }
            None => {}
        }

        // Handle en passant
        if let Some(en_passant_square) = board.get_en_passant_square() {
            if front_side_square == en_passant_square {
                let mut en_passant_move =
                    Move::new_capture(src, en_passant_square, piece, PieceType::Pawn);
                en_passant_move.is_en_passant = true;
                res.push(en_passant_move);
            }
        }
    }

    // If the square in front of the pawn is occupied, it may not advance.
    if board.is_square_occupied(front_square) {
        return res;
    }

    // Handle 1 square pawn advances
    let mut adv_move = Move::new(src, front_square, piece);

    // Handle possible promotion by advancing
    if piece_color.is_white() && front_square.is_in_rank(8)
        || !piece_color.is_white() && front_square.is_in_rank(1)
    {
        adv_move.is_promotion = true;
        for promotable_piece_type in PieceType::promotable_piece_types() {
            let mut promotion_move = adv_move.clone();
            promotion_move.promotes_to = Some(promotable_piece_type);
            res.push(promotion_move);
        }
    } else {
        res.push(adv_move);
    }

    // Check if the pawn is still on its starting square
    if piece_color.is_white() && src.is_in_rank(2) || !piece_color.is_white() && src.is_in_rank(7) {
        let dest_square = src.vertical_offset(2, board.is_white_turn());
        if !board.is_square_occupied(dest_square) {
            res.push(Move::new(src, dest_square, piece));
        }
    }

    res
}

fn generate_bishop_style_moves(board: &Board, src: Coordinate) -> Vec<Move> {
    generate_slider_style_moves(board, src, PieceType::Bishop)
}

fn generate_knight_moves(board: &Board, src: Coordinate) -> Vec<Move> {
    let piece = board.get_from_coordinate(src).unwrap();
    let mut res = vec![];

    let mut knight_moves_bb = get_piece_attacks_bb(PieceType::Knight, src);

    while knight_moves_bb != 0 {
        let (dest_bb, popped_bb) = pop_lsb(knight_moves_bb);
        knight_moves_bb = popped_bb;
        let dest_square = Coordinate::from_bb(dest_bb);
        match board.get_from_coordinate(dest_square) {
            Some(occupant) => {
                // Capture an enemy piece
                if occupant.color != piece.color {
                    res.push(Move::new_capture(
                        src,
                        dest_square,
                        piece,
                        occupant.piece_type,
                    ));
                }
            }
            None => {
                res.push(Move::new(src, dest_square, piece));
            }
        }
    }

    res
}

fn generate_rook_style_moves(board: &Board, src: Coordinate) -> Vec<Move> {
    generate_slider_style_moves(board, src, PieceType::Rook)
}

fn generate_slider_style_moves(board: &Board, src: Coordinate, piece_type: PieceType) -> Vec<Move> {
    let piece = board.get_from_coordinate(src).unwrap();

    let occupied = board.get_all_pieces_bb();
    let opposite_color_bb = board.get_color_bb(piece.color.other_color());
    let mut attacks = get_sliding_attacks_occupied(piece_type, src, occupied);

    let mut res = vec![];

    loop {
        let (dest, popped_attacks) = pop_lsb(attacks);
        let dest_square = Coordinate::from_bb(dest);

        // Check if the dest square is occupied
        if (occupied & dest) != 0 {
            // Check if it is occupied by an enemy piece
            if opposite_color_bb & dest != 0 {
                res.push(Move::new_capture(
                    src,
                    dest_square,
                    piece,
                    board.get_from_coordinate(dest_square).unwrap().piece_type,
                ));
            }
        } else {
            res.push(Move::new(src, dest_square, piece));
        }

        attacks = popped_attacks;
        if attacks == 0 {
            break;
        }
    }

    res
}

fn generate_king_moves(board: &Board, src: Coordinate, with_castling: bool) -> Vec<Move> {
    let piece = board.get_from_coordinate(src).unwrap();
    let mut res = vec![];
    let mut king_movement_bb = get_piece_attacks_bb(PieceType::King, src);

    while king_movement_bb != 0 {
        let (dest, popped_bb) = pop_lsb(king_movement_bb);
        king_movement_bb = popped_bb;
        let dest_square = Coordinate::from_bb(dest);
        // Check if square is occupied
        if let Some(occupying_piece) = board.get_from_coordinate(dest_square) {
            // Add a move for the capture of an opposing color piece
            if occupying_piece.color != piece.color {
                res.push(Move::new_capture(
                    src,
                    dest_square,
                    piece,
                    occupying_piece.piece_type,
                ));
            }
        } else {
            res.push(Move::new(src, dest_square, piece));
        }
    }

    if with_castling {
        res.extend(generate_castling(board, src));
    }

    res
}

fn generate_castling(board: &Board, src: Coordinate) -> Vec<Move> {
    let piece = board.get_from_coordinate(src).unwrap();
    let mut res = vec![];

    // Generate kingside castling
    if board.may_castle(piece.color, true) {
        match piece.color {
            Color::White => {
                let rook = board.get_from_coordinate(Coordinate::H1);

                // TODO: Should there be some way of removing such a guard?
                // Here we assume that if a player still has the right to castle
                // kingside, this would automatically imply that the rook is
                // currently on its starting square
                assert!(
                    src == Coordinate::E1 && rook.is_some(),
                    "Unexpected error: Invalid white kingside castling configuration"
                );

                if board.get_from_coordinate(Coordinate::F1).is_none()
                    && board.get_from_coordinate(Coordinate::G1).is_none()
                    && !are_squares_controlled_by_player(
                        board,
                        piece.color.other_color(),
                        &[Coordinate::E1, Coordinate::F1, Coordinate::G1],
                    )
                {
                    res.push(Move::new_castling(Color::White, true));
                }
            }
            Color::Black => {
                let rook = board.get_from_coordinate(Coordinate::H8);
                assert!(
                    src == Coordinate::E8 && rook.is_some(),
                    "Unexpected error: Invalid black kingside castling configuration"
                );
                if board.get_from_coordinate(Coordinate::F8).is_none()
                    && board.get_from_coordinate(Coordinate::G8).is_none()
                    && !are_squares_controlled_by_player(
                        board,
                        piece.color.other_color(),
                        &[Coordinate::E8, Coordinate::F8, Coordinate::G8],
                    )
                {
                    res.push(Move::new_castling(Color::Black, true));
                }
            }
        };
    }

    // Generate queenside castling
    if board.may_castle(piece.color, false) {
        match piece.color {
            Color::White => {
                let rook = board.get_from_coordinate(Coordinate::A1);

                assert!(
                    src == Coordinate::E1 && rook.is_some(),
                    "Unexpected error: Invalid white queenside castling configuration"
                );
                if board.get_from_coordinate(Coordinate::B1).is_none()
                    && board.get_from_coordinate(Coordinate::C1).is_none()
                    && board.get_from_coordinate(Coordinate::D1).is_none()
                    && !are_squares_controlled_by_player(
                        board,
                        piece.color.other_color(),
                        &[Coordinate::C1, Coordinate::D1, Coordinate::E1],
                    )
                {
                    res.push(Move::new_castling(Color::White, false));
                }
            }
            Color::Black => {
                let rook = board.get_from_coordinate(Coordinate::A8);
                assert!(
                    src == Coordinate::E8 && rook.is_some(),
                    "Unexpected error: Invalid black queenside castling configuration"
                );

                if board.get_from_coordinate(Coordinate::B8).is_none()
                    && board.get_from_coordinate(Coordinate::C8).is_none()
                    && board.get_from_coordinate(Coordinate::D8).is_none()
                    && !are_squares_controlled_by_player(
                        board,
                        piece.color.other_color(),
                        &[Coordinate::C8, Coordinate::D8, Coordinate::E8],
                    )
                {
                    res.push(Move::new_castling(Color::Black, false));
                }
            }
        }
    }

    res
}

fn generate_queen_moves(board: &Board, src: Coordinate) -> Vec<Move> {
    let mut res = generate_bishop_style_moves(board, src);
    res.append(&mut generate_rook_style_moves(board, src));
    res
}

// Determines whether a given move is legal
fn is_move_legal(board: &Board, color: Color, m: &Move) -> bool {
    if board.is_in_check() {
        if m.piece.piece_type != PieceType::King {
            // If in double check, the king has to move
            if more_than_one(board.get_checkers()) {
                return false;
            }
            // Interpose the check or capture the checker
            // Note: Path between returns all squares between the king and
            // the checker (including the square of the checker), hence this
            // covers both ways of defending against the check
            if (path_between(
                board.get_king_coordinate(color).expect("Missing king"),
                Coordinate::from_bb(lsb(board.get_checkers())),
            ) & m.dest.to_bb())
                == 0
            {
                return false;
            }
        } else if get_attackers_of_square_bb(
            board,
            m.dest,
            color.other_color(),
            board.get_all_pieces_bb() ^ m.src.to_bb(),
        ) != 0
        {
            // If it is a king move, we need to check that moving the king would not
            // expose an attack on the dest square, e.g. when moving along the same
            // diagonal as a bishop attack
            return false;
        }
    }

    if m.is_en_passant {
        // Check if this exposes the king to any attacks
        let king_coord = board.get_king_coordinate(color).expect("Missing king");
        // This should exist if an en passant move was generated
        let captured_square_bb = board
            .get_en_passant_square()
            .expect("Missing en passant square")
            .to_bb();
        let src_square_bb = m.src.to_bb();
        let dest_square_bb = m.dest.to_bb();
        // Calculate the bb by removing the capturing pawn's initial square and its victim, place
        // the pawn on its new square.
        let pieces =
            (board.get_all_pieces_bb() ^ src_square_bb ^ captured_square_bb) | dest_square_bb;

        // Ensure that this exposes no attacks on the king
        return (board.get_piece_types_bb_for_color(
            PieceType::Rook,
            PieceType::Queen,
            color.other_color(),
        ) & get_sliding_attacks_occupied(PieceType::Rook, king_coord, pieces)
            == 0)
            && (board.get_piece_types_bb_for_color(
                PieceType::Bishop,
                PieceType::Queen,
                color.other_color(),
            ) & get_sliding_attacks_occupied(PieceType::Bishop, king_coord, pieces)
                == 0);
    } else {
        // If it is a king move, we just have to ensure that the king is not walking into an attacked square
        if m.piece.piece_type == PieceType::King {
            return !is_square_controlled_by_player(board, color.other_color(), m.dest);
        } else {
            // If it is a non-king move, we check that the piece is not pinned. If it is, then it must not leave
            // the defense of the king.
            // To verify that the piece doesn't leave the defense of the king, we find a path between the king and
            // the piece that stretches from one edge of the board to the other, we then check that the piece's
            // destination remains in this path.
            return (board.get_king_shields(color) & m.src.to_bb() == 0)
                || (edge_to_edge_bb(
                    board.get_king_coordinate(color).expect("Missing king"),
                    m.src,
                ) & m.dest.to_bb()
                    != 0);
        }
    }
}

// Generate all moves given a certain board
pub fn generate_moves(board: &Board) -> Vec<Move> {
    let mut res = vec![];
    let color = board.get_player_color();

    for (i, piece) in board.get_pieces().iter().enumerate() {
        if let Some(piece) = piece {
            if color == piece.color {
                let coord = Coordinate::try_from(i).unwrap();
                let piece_moves = match piece.piece_type {
                    PieceType::Pawn => generate_pawn_moves(board, coord),
                    PieceType::Knight => generate_knight_moves(board, coord),
                    PieceType::Bishop => generate_bishop_style_moves(board, coord),
                    PieceType::Rook => generate_rook_style_moves(board, coord),
                    PieceType::King => generate_king_moves(board, coord, true),
                    PieceType::Queen => generate_queen_moves(board, coord),
                };
                res.extend(piece_moves);
            }
        }
    }

    res
}

pub fn generate_legal_moves(board: &Board) -> Vec<Move> {
    // Filter out illegal moves, i.e. moves that endanger the king
    let player_color = board.get_player_color();

    generate_moves(board)
        .into_iter()
        .filter(|m| is_move_legal(board, player_color, m))
        .collect()
}

pub fn generate_non_quiescent_moves(board: &Board) -> Vec<Move> {
    // Filter out illegal moves, i.e. moves that endanger the king.
    // Also filter out moves that aren't captures unless we are in check
    let player_color = board.get_player_color();
    let is_in_check = board.is_in_check();

    generate_moves(board)
        .into_iter()
        .filter(|m| is_move_legal(board, player_color, m) && (is_in_check || m.is_capture))
        .collect()
}

// Get bitboard that represents all attackers (of a particular color) of a particular square.
// This includes direct attacks only, i.e. this takes into account obstacles on the board.
pub fn get_attackers_of_square_bb(
    board: &Board,
    target: Coordinate,
    color: Color,
    occupied: Bitboard,
) -> Bitboard {
    (get_pawn_attacks_bb(color.other_color(), target)
        & board.get_piece_type_bb_for_color(PieceType::Pawn, color))
        | (get_piece_attacks_bb(PieceType::King, target)
            & board.get_piece_type_bb_for_color(PieceType::King, color))
        | (get_piece_attacks_bb(PieceType::Knight, target)
            & board.get_piece_type_bb_for_color(PieceType::Knight, color))
        | (get_sliding_attacks_occupied(PieceType::Rook, target, occupied)
            & board.get_piece_types_bb_for_color(PieceType::Rook, PieceType::Queen, color))
        | (get_sliding_attacks_occupied(PieceType::Bishop, target, occupied)
            & board.get_piece_types_bb_for_color(PieceType::Bishop, PieceType::Queen, color))
}

pub fn get_attackers_of_square_bb_for_piece_type(
    board: &Board,
    target: Coordinate,
    piece_type: PieceType,
    color: Color,
    occupied: Bitboard,
) -> Bitboard {
    match piece_type {
        PieceType::Pawn => {
            get_pawn_attacks_bb(color.other_color(), target)
                & board.get_piece_type_bb_for_color(PieceType::Pawn, color)
        }
        PieceType::King => {
            get_piece_attacks_bb(PieceType::King, target)
                & board.get_piece_type_bb_for_color(PieceType::King, color)
        }
        PieceType::Knight => {
            get_piece_attacks_bb(PieceType::Knight, target)
                & board.get_piece_type_bb_for_color(PieceType::Knight, color)
        }
        PieceType::Rook => {
            get_sliding_attacks_occupied(PieceType::Rook, target, occupied)
                & board.get_piece_type_bb_for_color(PieceType::Rook, color)
        }
        PieceType::Bishop => {
            get_sliding_attacks_occupied(PieceType::Bishop, target, occupied)
                & board.get_piece_type_bb_for_color(PieceType::Bishop, color)
        }
        PieceType::Queen => {
            (get_sliding_attacks_occupied(PieceType::Rook, target, occupied)
                & board.get_piece_type_bb_for_color(PieceType::Queen, color))
                | (get_sliding_attacks_occupied(PieceType::Bishop, target, occupied)
                    & board.get_piece_type_bb_for_color(PieceType::Queen, color))
        }
    }
}

pub fn is_square_controlled_by_player(board: &Board, color: Color, square: Coordinate) -> bool {
    get_attackers_of_square_bb(board, square, color, board.get_all_pieces_bb()) != 0
}

pub fn are_squares_controlled_by_player(
    board: &Board,
    color: Color,
    squares: &[Coordinate],
) -> bool {
    if squares.len() == 0 {
        return false;
    }

    squares
        .iter()
        .map(|square| is_square_controlled_by_player(board, color, *square))
        .reduce(|a, b| a || b)
        .unwrap()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::board::{Color, Piece};

    #[test]
    fn generate_basic_pawn_moves() {
        let mut board = Board::new_empty();
        let pawn = Piece {
            color: Color::White,
            piece_type: PieceType::Pawn,
        };
        let piece_coord = Coordinate::E2;

        board.place_piece(piece_coord, pawn);

        let moves = generate_pawn_moves(&board, piece_coord);

        assert!(moves.into_iter().eq(vec![
            Move::new(piece_coord, Coordinate::E3, pawn),
            Move::new(piece_coord, Coordinate::E4, pawn)
        ]));
    }

    #[test]
    fn generate_basic_pawn_moves_with_blockade_immediate_front() {
        let mut board = Board::new_empty();
        let pawn = Piece {
            color: Color::White,
            piece_type: PieceType::Pawn,
        };
        let blocking_pawn = Piece {
            color: Color::Black,
            piece_type: PieceType::Pawn,
        };
        let piece_coord = Coordinate::E2;

        board.place_piece(piece_coord, pawn);
        board.place_piece(Coordinate::E3, blocking_pawn);

        let moves = generate_pawn_moves(&board, piece_coord);

        assert!(moves.into_iter().eq(vec![]));
    }

    #[test]
    fn generate_basic_pawn_moves_with_blockade_two_squares_front() {
        let mut board = Board::new_empty();
        let pawn = Piece {
            color: Color::White,
            piece_type: PieceType::Pawn,
        };
        let blocking_pawn = Piece {
            color: Color::Black,
            piece_type: PieceType::Pawn,
        };

        let piece_coord = Coordinate::E2;

        board.place_piece(piece_coord, pawn);
        board.place_piece(Coordinate::E4, blocking_pawn);

        let moves = generate_pawn_moves(&board, piece_coord);

        assert!(moves
            .into_iter()
            .eq(vec![Move::new(piece_coord, Coordinate::E3, pawn),]));
    }

    #[test]
    fn generate_pawn_moves_with_captures() {
        let mut board = Board::new_empty();
        let pawn = Piece {
            color: Color::White,
            piece_type: PieceType::Pawn,
        };
        let capturable_piece_1 = Piece {
            color: Color::Black,
            piece_type: PieceType::Bishop,
        };
        let capturable_piece_2 = Piece {
            color: Color::Black,
            piece_type: PieceType::Knight,
        };
        let piece_coord = Coordinate::E4;

        board.place_piece(piece_coord, pawn);
        board.place_piece(Coordinate::D5, capturable_piece_1);
        board.place_piece(Coordinate::F5, capturable_piece_2);

        let moves = generate_pawn_moves(&board, piece_coord);

        assert!(moves.into_iter().eq(vec![
            Move::new_capture(piece_coord, Coordinate::D5, pawn, PieceType::Bishop),
            Move::new_capture(piece_coord, Coordinate::F5, pawn, PieceType::Knight),
            Move::new(piece_coord, Coordinate::E5, pawn),
        ]));
    }

    #[test]
    fn generate_white_legal_en_passant() {
        let mut board = Board::new_empty();
        let pawn = Piece {
            color: Color::White,
            piece_type: PieceType::Pawn,
        };
        let capturable_piece = Piece {
            color: Color::Black,
            piece_type: PieceType::Pawn,
        };
        let piece_coord = Coordinate::E5;
        let last_move = Move::new(Coordinate::F7, Coordinate::F5, capturable_piece);

        board.place_piece(piece_coord, pawn);
        board.place_piece(Coordinate::F7, capturable_piece);
        board.apply_move(&last_move);

        let moves = generate_pawn_moves(&board, piece_coord);
        let mut expected_en_passant_move =
            Move::new_capture(piece_coord, Coordinate::F6, pawn, PieceType::Pawn);
        expected_en_passant_move.is_en_passant = true;

        assert!(moves.contains(&expected_en_passant_move));
    }

    #[test]
    fn white_en_passant_illegal_non_double_advance() {
        let mut board = Board::new_empty();
        let pawn = Piece {
            color: Color::White,
            piece_type: PieceType::Pawn,
        };
        let capturable_piece = Piece {
            color: Color::Black,
            piece_type: PieceType::Pawn,
        };
        let piece_coord = Coordinate::E5;
        let last_move = Move::new(Coordinate::F6, Coordinate::F5, capturable_piece);

        board.place_piece(piece_coord, pawn);
        board.place_piece(Coordinate::F6, capturable_piece);
        board.apply_move(&last_move);

        let moves = generate_pawn_moves(&board, piece_coord);

        for m in moves {
            assert!(!m.is_en_passant);
        }
    }

    #[test]
    fn generate_black_legal_en_passant() {
        let mut board = Board::new_empty();
        let pawn = Piece {
            color: Color::Black,
            piece_type: PieceType::Pawn,
        };
        let capturable_piece = Piece {
            color: Color::White,
            piece_type: PieceType::Pawn,
        };
        let piece_coord = Coordinate::E4;
        let last_move = Move::new(Coordinate::F2, Coordinate::F4, capturable_piece);

        board.place_piece(piece_coord, pawn);
        board.place_piece(Coordinate::F2, capturable_piece);
        board.apply_move(&last_move);

        let moves = generate_pawn_moves(&board, piece_coord);
        let mut expected_en_passant_move =
            Move::new_capture(piece_coord, Coordinate::F3, pawn, PieceType::Pawn);
        expected_en_passant_move.is_en_passant = true;

        assert!(moves.contains(&expected_en_passant_move));
    }

    #[test]
    fn black_en_passant_illegal_non_double_advance() {
        let mut board = Board::new_empty();
        let pawn = Piece {
            color: Color::Black,
            piece_type: PieceType::Pawn,
        };
        let capturable_piece = Piece {
            color: Color::White,
            piece_type: PieceType::Pawn,
        };
        let piece_coord = Coordinate::E4;
        let last_move = Move::new(Coordinate::F3, Coordinate::F4, capturable_piece);

        board.place_piece(piece_coord, pawn);
        board.place_piece(Coordinate::F3, capturable_piece);
        board.apply_move(&last_move);

        let moves = generate_pawn_moves(&board, piece_coord);

        for m in moves {
            assert!(!m.is_en_passant);
        }
    }

    #[test]
    fn generate_white_promotion() {
        let mut board = Board::new_empty();
        let pawn = Piece {
            color: Color::White,
            piece_type: PieceType::Pawn,
        };
        let piece_coord = Coordinate::E7;

        board.place_piece(piece_coord, pawn);

        let mut expected_move = Move::new(piece_coord, Coordinate::E8, pawn);
        expected_move.is_promotion = true;

        let moves = generate_pawn_moves(&board, piece_coord);

        for pt in PieceType::promotable_piece_types() {
            expected_move.promotes_to = Some(pt);
            assert!(moves.contains(&expected_move));
        }
    }

    #[test]
    fn generate_white_capture_with_promotion() {
        let mut board = Board::new_empty();
        let pawn = Piece {
            color: Color::White,
            piece_type: PieceType::Pawn,
        };
        let enemy_rook = Piece {
            color: Color::Black,
            piece_type: PieceType::Rook,
        };
        let piece_coord = Coordinate::E7;

        board.place_piece(piece_coord, pawn);
        board.place_piece(Coordinate::F8, enemy_rook);

        let mut expected_move =
            Move::new_capture(piece_coord, Coordinate::F8, pawn, PieceType::Rook);
        expected_move.is_promotion = true;

        let moves = generate_pawn_moves(&board, piece_coord);

        for pt in PieceType::promotable_piece_types() {
            expected_move.promotes_to = Some(pt);
            assert!(moves.contains(&expected_move));
        }
    }

    #[test]
    fn generate_black_promotion() {
        let mut board = Board::new_empty();
        let pawn = Piece {
            color: Color::Black,
            piece_type: PieceType::Pawn,
        };
        let piece_coord = Coordinate::E2;

        board.place_piece(piece_coord, pawn);

        let mut expected_move = Move::new(piece_coord, Coordinate::E1, pawn);
        expected_move.is_promotion = true;

        let moves = generate_pawn_moves(&board, piece_coord);

        for pt in PieceType::promotable_piece_types() {
            expected_move.promotes_to = Some(pt);
            assert!(moves.contains(&expected_move));
        }
    }

    #[test]
    fn generate_black_capture_with_promotion() {
        let mut board = Board::new_empty();
        let pawn = Piece {
            color: Color::Black,
            piece_type: PieceType::Pawn,
        };
        let enemy_knight = Piece {
            color: Color::White,
            piece_type: PieceType::Knight,
        };
        let piece_coord = Coordinate::D2;

        board.place_piece(piece_coord, pawn);
        board.place_piece(Coordinate::C1, enemy_knight);

        let mut expected_move =
            Move::new_capture(piece_coord, Coordinate::C1, pawn, PieceType::Knight);
        expected_move.is_promotion = true;

        let moves = generate_pawn_moves(&board, piece_coord);

        for pt in PieceType::promotable_piece_types() {
            expected_move.promotes_to = Some(pt);
            assert!(moves.contains(&expected_move));
        }
    }

    #[test]
    fn generate_basic_bishop_moves() {
        let mut board = Board::new_empty();
        let bishop = Piece {
            color: Color::White,
            piece_type: PieceType::Bishop,
        };
        let piece_coord = Coordinate::E4;

        board.place_piece(piece_coord, bishop);

        let moves = generate_bishop_style_moves(&board, piece_coord);

        assert_eq!(moves.len(), 13);

        // Top left
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D5, bishop)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::C6, bishop)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::B7, bishop)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::A8, bishop)));

        // Bottom left
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D3, bishop)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::C2, bishop)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::B1, bishop)));

        // Top right
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F5, bishop)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::G6, bishop)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::H7, bishop)));

        // Bottom right
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F3, bishop)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::G2, bishop)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::H1, bishop)));
    }

    #[test]
    fn generate_bishop_style_moves_with_captures() {
        let mut board = Board::new_empty();
        let bishop = Piece {
            color: Color::White,
            piece_type: PieceType::Bishop,
        };
        let capturable_piece_1 = Piece {
            color: Color::Black,
            piece_type: PieceType::Knight,
        };
        let capturable_piece_2 = Piece {
            color: Color::Black,
            piece_type: PieceType::Pawn,
        };
        let piece_coord = Coordinate::E4;

        board.place_piece(piece_coord, bishop);
        board.place_piece(Coordinate::B7, capturable_piece_1);
        board.place_piece(Coordinate::F3, capturable_piece_2);

        let moves = generate_bishop_style_moves(&board, piece_coord);

        // Top left
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D5, bishop)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::C6, bishop)));
        assert!(moves.contains(&Move::new_capture(
            piece_coord,
            Coordinate::B7,
            bishop,
            PieceType::Knight
        )));

        // Bottom left
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D3, bishop)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::C2, bishop)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::B1, bishop)));

        // Top right
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F5, bishop)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::G6, bishop)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::H7, bishop)));

        // Bottom right
        assert!(moves.contains(&Move::new_capture(
            piece_coord,
            Coordinate::F3,
            bishop,
            PieceType::Pawn
        )));
    }

    #[test]
    fn generate_bishop_style_moves_with_ally_blockades() {
        let mut board = Board::new_empty();
        let bishop = Piece {
            color: Color::White,
            piece_type: PieceType::Bishop,
        };
        let blocking_piece_1 = Piece {
            color: Color::White,
            piece_type: PieceType::Knight,
        };
        let blocking_piece_2 = Piece {
            color: Color::White,
            piece_type: PieceType::Pawn,
        };
        let piece_coord = Coordinate::E4;

        board.place_piece(piece_coord, bishop);
        board.place_piece(Coordinate::B7, blocking_piece_1);
        board.place_piece(Coordinate::F3, blocking_piece_2);

        let moves = generate_bishop_style_moves(&board, piece_coord);

        assert_eq!(moves.len(), 8);

        // Top left
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D5, bishop)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::C6, bishop)));

        // Bottom left
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D3, bishop)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::C2, bishop)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::B1, bishop)));

        // Top right
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F5, bishop)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::G6, bishop)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::H7, bishop)));
    }

    #[test]
    fn generate_basic_rook_moves() {
        let mut board = Board::new_empty();
        let piece = Piece {
            color: Color::White,
            piece_type: PieceType::Rook,
        };
        let piece_coord = Coordinate::E4;

        board.place_piece(piece_coord, piece);

        let moves = generate_rook_style_moves(&board, piece_coord);

        assert_eq!(moves.len(), 14);

        // Top
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E5, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E6, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E7, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E8, piece)));

        // Bottom
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E3, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E2, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E1, piece)));

        // Right
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F4, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::G4, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::H4, piece)));

        // Left
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D4, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::C4, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::B4, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::A4, piece)));
    }

    #[test]
    fn generate_rook_moves_with_captures() {
        let mut board = Board::new_empty();
        let piece = Piece {
            color: Color::White,
            piece_type: PieceType::Rook,
        };
        let capturable_piece_1 = Piece {
            color: Color::Black,
            piece_type: PieceType::Bishop,
        };
        let capturable_piece_2 = Piece {
            color: Color::Black,
            piece_type: PieceType::King,
        };
        let piece_coord = Coordinate::E4;

        board.place_piece(piece_coord, piece);
        board.place_piece(Coordinate::E7, capturable_piece_1);
        board.place_piece(Coordinate::G4, capturable_piece_2);

        let moves = generate_rook_style_moves(&board, piece_coord);

        assert_eq!(moves.len(), 12);

        // Top
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E5, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E6, piece)));
        assert!(moves.contains(&Move::new_capture(
            piece_coord,
            Coordinate::E7,
            piece,
            PieceType::Bishop
        )));

        // Bottom
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E3, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E2, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E1, piece)));

        // Right
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F4, piece)));
        assert!(moves.contains(&Move::new_capture(
            piece_coord,
            Coordinate::G4,
            piece,
            PieceType::King
        )));

        // Left
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D4, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::C4, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::B4, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::A4, piece)));
    }

    #[test]
    fn generate_rook_moves_with_ally_blockades() {
        let mut board = Board::new_empty();
        let piece = Piece {
            color: Color::White,
            piece_type: PieceType::Rook,
        };
        let blocking_piece_1 = Piece {
            color: Color::White,
            piece_type: PieceType::Knight,
        };
        let blocking_piece_2 = Piece {
            color: Color::White,
            piece_type: PieceType::Pawn,
        };
        let piece_coord = Coordinate::E4;

        board.place_piece(piece_coord, piece);
        board.place_piece(Coordinate::E7, blocking_piece_1);
        board.place_piece(Coordinate::G4, blocking_piece_2);

        let moves = generate_rook_style_moves(&board, piece_coord);

        assert_eq!(moves.len(), 10);

        // Top
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E5, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E6, piece)));

        // Bottom
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E3, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E2, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E1, piece)));

        // Right
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F4, piece)));

        // Left
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D4, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::C4, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::B4, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::A4, piece)));
    }

    #[test]
    fn generate_basic_queen_moves() {
        let mut board = Board::new_empty();
        let piece = Piece {
            color: Color::White,
            piece_type: PieceType::Queen,
        };
        let piece_coord = Coordinate::E4;

        board.place_piece(piece_coord, piece);

        let moves = generate_queen_moves(&board, piece_coord);

        assert_eq!(moves.len(), 27);

        // Top
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E5, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E6, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E7, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E8, piece)));

        // Bottom
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E3, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E2, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E1, piece)));

        // Right
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F4, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::G4, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::H4, piece)));

        // Left
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D4, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::C4, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::B4, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::A4, piece)));

        // Top left
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D5, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::C6, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::B7, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::A8, piece)));

        // Bottom left
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D3, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::C2, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::B1, piece)));

        // Top right
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F5, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::G6, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::H7, piece)));

        // Bottom right
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F3, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::G2, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::H1, piece)));
    }

    #[test]
    fn generate_basic_queen_moves_with_captures() {
        let mut board = Board::new_empty();
        let piece = Piece {
            color: Color::White,
            piece_type: PieceType::Queen,
        };
        let capturable_piece_1 = Piece {
            color: Color::Black,
            piece_type: PieceType::Bishop,
        };
        let capturable_piece_2 = Piece {
            color: Color::Black,
            piece_type: PieceType::Pawn,
        };

        let piece_coord = Coordinate::E4;

        board.place_piece(piece_coord, piece);
        board.place_piece(Coordinate::F5, capturable_piece_1);
        board.place_piece(Coordinate::E2, capturable_piece_2);

        let moves = generate_queen_moves(&board, piece_coord);

        assert_eq!(moves.len(), 24);

        // Top
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E5, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E6, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E7, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E8, piece)));

        // Bottom
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E3, piece)));
        assert!(moves.contains(&Move::new_capture(
            piece_coord,
            Coordinate::E2,
            piece,
            PieceType::Pawn
        )));

        // Right
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F4, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::G4, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::H4, piece)));

        // Left
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D4, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::C4, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::B4, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::A4, piece)));

        // Top left
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D5, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::C6, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::B7, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::A8, piece)));

        // Bottom left
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D3, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::C2, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::B1, piece)));

        // Top right
        assert!(moves.contains(&Move::new_capture(
            piece_coord,
            Coordinate::F5,
            piece,
            PieceType::Bishop
        )));

        // Bottom right
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F3, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::G2, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::H1, piece)));
    }

    #[test]
    fn generate_basic_queen_moves_with_ally_blockades() {
        let mut board = Board::new_empty();
        let piece = Piece {
            color: Color::White,
            piece_type: PieceType::Queen,
        };
        let blocking_piece_1 = Piece {
            color: Color::White,
            piece_type: PieceType::Knight,
        };
        let blocking_piece_2 = Piece {
            color: Color::White,
            piece_type: PieceType::Pawn,
        };
        let piece_coord = Coordinate::E4;

        board.place_piece(piece_coord, piece);
        board.place_piece(Coordinate::E6, blocking_piece_1);
        board.place_piece(Coordinate::D5, blocking_piece_2);

        let moves = generate_queen_moves(&board, piece_coord);

        assert_eq!(moves.len(), 20);

        // Top
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E5, piece)));

        // Bottom
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E3, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E2, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E1, piece)));

        // Right
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F4, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::G4, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::H4, piece)));

        // Left
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D4, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::C4, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::B4, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::A4, piece)));

        // Top left

        // Bottom left
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D3, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::C2, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::B1, piece)));

        // Top right
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F5, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::G6, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::H7, piece)));

        // Bottom right
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F3, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::G2, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::H1, piece)));
    }

    #[test]
    fn generate_basic_knight_moves() {
        let mut board = Board::new_empty();
        let piece = Piece {
            color: Color::White,
            piece_type: PieceType::Knight,
        };
        let piece_coord = Coordinate::E4;

        board.place_piece(piece_coord, piece);

        let moves = generate_knight_moves(&board, piece_coord);

        assert_eq!(moves.len(), 8);

        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F6, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D6, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::C5, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::C3, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D2, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F2, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::G3, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::G5, piece)));
    }

    #[test]
    fn generate_basic_knight_moves_with_captures() {
        let mut board = Board::new_empty();
        let piece = Piece {
            color: Color::White,
            piece_type: PieceType::Knight,
        };
        let capturable_piece_1 = Piece {
            color: Color::Black,
            piece_type: PieceType::Knight,
        };
        let capturable_piece_2 = Piece {
            color: Color::Black,
            piece_type: PieceType::Rook,
        };

        let piece_coord = Coordinate::E4;

        board.place_piece(piece_coord, piece);
        board.place_piece(Coordinate::F6, capturable_piece_1);
        board.place_piece(Coordinate::C3, capturable_piece_2);

        let moves = generate_knight_moves(&board, piece_coord);

        assert_eq!(moves.len(), 8);

        assert!(moves.contains(&Move::new_capture(
            piece_coord,
            Coordinate::F6,
            piece,
            PieceType::Knight
        )));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D6, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::C5, piece)));
        assert!(moves.contains(&Move::new_capture(
            piece_coord,
            Coordinate::C3,
            piece,
            PieceType::Rook
        )));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D2, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F2, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::G3, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::G5, piece)));
    }

    #[test]
    fn generate_basic_knight_moves_with_ally_blockades() {
        let mut board = Board::new_empty();
        let piece = Piece {
            color: Color::White,
            piece_type: PieceType::Knight,
        };
        let blocking_piece_1 = Piece {
            color: Color::White,
            piece_type: PieceType::Knight,
        };
        let blocking_piece_2 = Piece {
            color: Color::White,
            piece_type: PieceType::Rook,
        };

        let piece_coord = Coordinate::E4;

        board.place_piece(piece_coord, piece);
        board.place_piece(Coordinate::F6, blocking_piece_1);
        board.place_piece(Coordinate::C3, blocking_piece_2);

        let moves = generate_knight_moves(&board, piece_coord);

        assert_eq!(moves.len(), 6);

        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D6, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::C5, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D2, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F2, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::G3, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::G5, piece)));
    }

    #[test]
    fn generate_basic_king_moves() {
        let mut board = Board::new_empty();
        let piece = Piece {
            color: Color::White,
            piece_type: PieceType::Knight,
        };

        let piece_coord = Coordinate::E4;

        board.place_piece(piece_coord, piece);

        let moves = generate_king_moves(&board, piece_coord, true);

        assert_eq!(moves.len(), 8);

        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D5, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E5, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F5, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D4, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F4, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D3, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E3, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F3, piece)));
    }

    #[test]
    fn generate_basic_king_moves_with_captures() {
        let mut board = Board::new_empty();
        let piece = Piece {
            color: Color::White,
            piece_type: PieceType::Knight,
        };
        let capturable_piece_1 = Piece {
            color: Color::Black,
            piece_type: PieceType::Bishop,
        };
        let capturable_piece_2 = Piece {
            color: Color::Black,
            piece_type: PieceType::Knight,
        };

        let piece_coord = Coordinate::E4;

        board.place_piece(piece_coord, piece);
        board.place_piece(Coordinate::E5, capturable_piece_1);
        board.place_piece(Coordinate::D3, capturable_piece_2);

        let moves = generate_king_moves(&board, piece_coord, true);

        assert_eq!(moves.len(), 8);

        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D5, piece)));
        assert!(moves.contains(&Move::new_capture(
            piece_coord,
            Coordinate::E5,
            piece,
            PieceType::Bishop
        )));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F5, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D4, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F4, piece)));
        assert!(moves.contains(&Move::new_capture(
            piece_coord,
            Coordinate::D3,
            piece,
            PieceType::Knight
        )));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E3, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F3, piece)));
    }

    #[test]
    fn generate_basic_king_moves_with_ally_blockades() {
        let mut board = Board::new_empty();
        let piece = Piece {
            color: Color::White,
            piece_type: PieceType::Knight,
        };
        let blocking_piece_1 = Piece {
            color: Color::White,
            piece_type: PieceType::Bishop,
        };
        let blocking_piece_2 = Piece {
            color: Color::White,
            piece_type: PieceType::Knight,
        };

        let piece_coord = Coordinate::E4;

        board.place_piece(piece_coord, piece);
        board.place_piece(Coordinate::E5, blocking_piece_1);
        board.place_piece(Coordinate::D3, blocking_piece_2);

        let moves = generate_king_moves(&board, piece_coord, true);

        assert_eq!(moves.len(), 6);

        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D5, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F5, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D4, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F4, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E3, piece)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F3, piece)));
    }

    #[test]
    fn generate_white_kingside_castling() {
        let mut board = Board::new_empty();
        board.enable_castling(Color::White, true);
        let king = Piece {
            color: Color::White,
            piece_type: PieceType::King,
        };
        let rook = Piece {
            color: Color::White,
            piece_type: PieceType::Rook,
        };

        board.place_piece(Coordinate::E1, king);
        board.place_piece(Coordinate::H1, rook);

        let moves = generate_castling(&board, Coordinate::E1);

        assert_eq!(moves.len(), 1);
        assert!(moves.contains(&Move::new_castling(Color::White, true)));
    }

    #[test]
    fn generate_white_kingside_castling_with_attacked_castling_squares() {
        let attacking_pieces = [
            (
                Piece {
                    color: Color::Black,
                    piece_type: PieceType::Bishop,
                },
                Coordinate::E3,
            ),
            (
                Piece {
                    color: Color::Black,
                    piece_type: PieceType::Queen,
                },
                Coordinate::G6,
            ),
        ];

        for (attacking_piece, attacking_square) in &attacking_pieces {
            let mut board = Board::new_empty();
            board.enable_castling(Color::White, true);
            let king = Piece {
                color: Color::White,
                piece_type: PieceType::King,
            };
            let rook = Piece {
                color: Color::White,
                piece_type: PieceType::Rook,
            };

            board.place_piece(Coordinate::E1, king);
            board.place_piece(Coordinate::H1, rook);
            board.place_piece(*attacking_square, *attacking_piece);

            let moves = generate_castling(&board, Coordinate::E1);

            assert_eq!(moves.len(), 0);
        }
    }

    #[test]
    fn generate_white_kingside_castling_while_in_check() {
        let mut board = Board::new_empty();
        board.enable_castling(Color::White, true);
        let king = Piece {
            color: Color::White,
            piece_type: PieceType::King,
        };
        let rook = Piece {
            color: Color::White,
            piece_type: PieceType::Rook,
        };
        let enemy_rook = Piece {
            color: Color::Black,
            piece_type: PieceType::Rook,
        };

        board.place_piece(Coordinate::E1, king);
        board.place_piece(Coordinate::H1, rook);
        board.place_piece(Coordinate::E4, enemy_rook);

        let moves = generate_castling(&board, Coordinate::E1);

        assert_eq!(moves.len(), 0);
    }

    #[test]
    fn generate_white_kingside_castling_with_pieces_in_the_way() {
        let obstructing_pieces = [
            (
                Piece {
                    color: Color::White,
                    piece_type: PieceType::Bishop,
                },
                Coordinate::F1,
            ),
            (
                Piece {
                    color: Color::Black,
                    piece_type: PieceType::Bishop,
                },
                Coordinate::F1,
            ),
            (
                Piece {
                    color: Color::White,
                    piece_type: PieceType::Knight,
                },
                Coordinate::G1,
            ),
            (
                Piece {
                    color: Color::Black,
                    piece_type: PieceType::Knight,
                },
                Coordinate::G1,
            ),
        ];

        for (obstructing_piece, obstructing_square) in &obstructing_pieces {
            let mut board = Board::new_empty();
            board.enable_castling(Color::White, true);
            board.disable_castling(Color::White, false);
            let king = Piece {
                color: Color::White,
                piece_type: PieceType::King,
            };
            let rook = Piece {
                color: Color::White,
                piece_type: PieceType::Rook,
            };

            board.place_piece(Coordinate::E1, king);
            board.place_piece(Coordinate::H1, rook);
            board.place_piece(*obstructing_square, *obstructing_piece);

            let moves = generate_castling(&board, Coordinate::E1);

            assert_eq!(moves.len(), 0);
        }
    }

    #[test]
    fn generate_white_queenside_castling() {
        let mut board = Board::new_empty();
        board.enable_castling(Color::White, false);
        let king = Piece {
            color: Color::White,
            piece_type: PieceType::King,
        };
        let rook = Piece {
            color: Color::White,
            piece_type: PieceType::Rook,
        };

        board.place_piece(Coordinate::E1, king);
        board.place_piece(Coordinate::A1, rook);

        let moves = generate_castling(&board, Coordinate::E1);

        assert_eq!(moves.len(), 1);
        assert!(moves.contains(&Move::new_castling(Color::White, false)));
    }

    #[test]
    fn generate_white_queenside_castling_with_pieces_in_the_way() {
        let obstructing_pieces = [
            (
                Piece {
                    color: Color::White,
                    piece_type: PieceType::Bishop,
                },
                Coordinate::C1,
            ),
            (
                Piece {
                    color: Color::Black,
                    piece_type: PieceType::Bishop,
                },
                Coordinate::C1,
            ),
            (
                Piece {
                    color: Color::White,
                    piece_type: PieceType::Knight,
                },
                Coordinate::B1,
            ),
            (
                Piece {
                    color: Color::Black,
                    piece_type: PieceType::Knight,
                },
                Coordinate::B1,
            ),
            (
                Piece {
                    color: Color::White,
                    piece_type: PieceType::Queen,
                },
                Coordinate::D1,
            ),
            (
                Piece {
                    color: Color::Black,
                    piece_type: PieceType::Queen,
                },
                Coordinate::D1,
            ),
        ];

        for (obstructing_piece, obstructing_square) in &obstructing_pieces {
            let mut board = Board::new_empty();
            board.enable_castling(Color::White, false);
            let king = Piece {
                color: Color::White,
                piece_type: PieceType::King,
            };
            let rook = Piece {
                color: Color::White,
                piece_type: PieceType::Rook,
            };

            board.place_piece(Coordinate::E1, king);
            board.place_piece(Coordinate::A1, rook);
            board.place_piece(*obstructing_square, *obstructing_piece);

            let moves = generate_castling(&board, Coordinate::E1);

            assert_eq!(moves.len(), 0);
        }
    }

    #[test]
    fn generate_white_queenside_castling_with_attacked_castling_squares() {
        let attacking_pieces = [
            (
                Piece {
                    color: Color::Black,
                    piece_type: PieceType::Bishop,
                },
                Coordinate::A3,
            ),
            (
                Piece {
                    color: Color::Black,
                    piece_type: PieceType::Queen,
                },
                Coordinate::D5,
            ),
        ];

        for (attacking_piece, attacking_square) in &attacking_pieces {
            let mut board = Board::new_empty();
            board.enable_castling(Color::White, false);
            let king = Piece {
                color: Color::White,
                piece_type: PieceType::King,
            };
            let rook = Piece {
                color: Color::White,
                piece_type: PieceType::Rook,
            };

            board.place_piece(Coordinate::E1, king);
            board.place_piece(Coordinate::A1, rook);
            board.place_piece(*attacking_square, *attacking_piece);

            let moves = generate_castling(&board, Coordinate::E1);

            assert_eq!(moves.len(), 0);
        }
    }

    #[test]
    fn generate_white_queenside_castling_with_controlled_b1_square() {
        let mut board = Board::new_empty();
        board.enable_castling(Color::White, false);
        let king = Piece {
            color: Color::White,
            piece_type: PieceType::King,
        };
        let rook = Piece {
            color: Color::White,
            piece_type: PieceType::Rook,
        };
        let enemy_rook = Piece {
            color: Color::Black,
            piece_type: PieceType::Rook,
        };

        board.place_piece(Coordinate::E1, king);
        board.place_piece(Coordinate::A1, rook);
        board.place_piece(Coordinate::B4, enemy_rook);

        let moves = generate_castling(&board, Coordinate::E1);

        assert_eq!(moves.len(), 1);
        assert!(moves.contains(&Move::new_castling(Color::White, false)));
    }

    #[test]
    fn generate_white_queenside_castling_while_in_check() {
        let mut board = Board::new_empty();
        board.enable_castling(Color::White, false);
        let king = Piece {
            color: Color::White,
            piece_type: PieceType::King,
        };
        let rook = Piece {
            color: Color::White,
            piece_type: PieceType::Rook,
        };
        let enemy_bishop = Piece {
            color: Color::Black,
            piece_type: PieceType::Bishop,
        };

        board.place_piece(Coordinate::E1, king);
        board.place_piece(Coordinate::A1, rook);
        board.place_piece(Coordinate::C3, enemy_bishop);

        let moves = generate_castling(&board, Coordinate::E1);

        assert_eq!(moves.len(), 0);
    }

    #[test]
    #[should_panic]
    fn generate_white_kingside_castling_with_missing_rook() {
        let mut board = Board::new_empty();
        board.enable_castling(Color::White, true);
        let king = Piece {
            color: Color::White,
            piece_type: PieceType::King,
        };

        board.place_piece(Coordinate::E1, king);

        generate_castling(&board, Coordinate::E1);
    }

    #[test]
    #[should_panic]
    fn generate_white_queenside_castling_with_missing_rook() {
        let mut board = Board::new_empty();
        board.enable_castling(Color::White, false);
        let king = Piece {
            color: Color::White,
            piece_type: PieceType::King,
        };

        board.place_piece(Coordinate::E1, king);

        generate_castling(&board, Coordinate::E1);
    }

    #[test]
    fn generate_black_kingside_castling() {
        let mut board = Board::new_empty();
        board.enable_castling(Color::Black, true);
        let king = Piece {
            color: Color::Black,
            piece_type: PieceType::King,
        };
        let rook = Piece {
            color: Color::Black,
            piece_type: PieceType::Rook,
        };

        board.place_piece(Coordinate::E8, king);
        board.place_piece(Coordinate::H8, rook);

        let moves = generate_castling(&board, Coordinate::E8);

        assert_eq!(moves.len(), 1);
        assert!(moves.contains(&Move::new_castling(Color::Black, true)));
    }

    #[test]
    fn generate_black_queenside_castling() {
        let mut board = Board::new_empty();
        board.enable_castling(Color::Black, false);
        let king = Piece {
            color: Color::Black,
            piece_type: PieceType::King,
        };
        let rook = Piece {
            color: Color::Black,
            piece_type: PieceType::Rook,
        };

        board.place_piece(Coordinate::E8, king);
        board.place_piece(Coordinate::A8, rook);

        let moves = generate_castling(&board, Coordinate::E8);

        assert_eq!(moves.len(), 1);
        assert!(moves.contains(&Move::new_castling(Color::Black, false)));
    }

    #[test]
    fn generate_black_kingside_castling_while_in_check() {
        let mut board = Board::new_empty();
        board.enable_castling(Color::Black, true);
        let king = Piece {
            color: Color::Black,
            piece_type: PieceType::King,
        };
        let rook = Piece {
            color: Color::Black,
            piece_type: PieceType::Rook,
        };
        let enemy_rook = Piece {
            color: Color::White,
            piece_type: PieceType::Rook,
        };

        board.place_piece(Coordinate::E8, king);
        board.place_piece(Coordinate::H8, rook);
        board.place_piece(Coordinate::A8, enemy_rook);

        let moves = generate_castling(&board, Coordinate::E8);

        assert_eq!(moves.len(), 0);
    }

    #[test]
    fn generate_black_queenside_castling_while_in_check() {
        let mut board = Board::new_empty();
        board.enable_castling(Color::Black, false);
        let king = Piece {
            color: Color::Black,
            piece_type: PieceType::King,
        };
        let rook = Piece {
            color: Color::Black,
            piece_type: PieceType::Rook,
        };
        let enemy_knight = Piece {
            color: Color::White,
            piece_type: PieceType::Knight,
        };

        board.place_piece(Coordinate::E8, king);
        board.place_piece(Coordinate::A8, rook);
        board.place_piece(Coordinate::C7, enemy_knight);

        let moves = generate_castling(&board, Coordinate::E8);

        assert_eq!(moves.len(), 0);
    }

    #[test]
    fn generate_black_kingside_castling_with_pieces_in_the_way() {
        let obstructing_pieces = [
            (
                Piece {
                    color: Color::White,
                    piece_type: PieceType::Bishop,
                },
                Coordinate::F8,
            ),
            (
                Piece {
                    color: Color::Black,
                    piece_type: PieceType::Bishop,
                },
                Coordinate::F8,
            ),
            (
                Piece {
                    color: Color::White,
                    piece_type: PieceType::Knight,
                },
                Coordinate::G8,
            ),
            (
                Piece {
                    color: Color::Black,
                    piece_type: PieceType::Knight,
                },
                Coordinate::G8,
            ),
        ];

        for (obstructing_piece, obstructing_square) in &obstructing_pieces {
            let mut board = Board::new_empty();
            board.enable_castling(Color::Black, true);
            let king = Piece {
                color: Color::Black,
                piece_type: PieceType::King,
            };
            let rook = Piece {
                color: Color::Black,
                piece_type: PieceType::Rook,
            };

            board.place_piece(Coordinate::E8, king);
            board.place_piece(Coordinate::H8, rook);
            board.place_piece(*obstructing_square, *obstructing_piece);

            let moves = generate_castling(&board, Coordinate::E8);

            assert_eq!(moves.len(), 0);
        }
    }

    #[test]
    fn generate_black_queenside_castling_with_pieces_in_the_way() {
        let obstructing_pieces = [
            (
                Piece {
                    color: Color::White,
                    piece_type: PieceType::Bishop,
                },
                Coordinate::C8,
            ),
            (
                Piece {
                    color: Color::Black,
                    piece_type: PieceType::Bishop,
                },
                Coordinate::C8,
            ),
            (
                Piece {
                    color: Color::White,
                    piece_type: PieceType::Knight,
                },
                Coordinate::B8,
            ),
            (
                Piece {
                    color: Color::Black,
                    piece_type: PieceType::Knight,
                },
                Coordinate::B8,
            ),
            (
                Piece {
                    color: Color::White,
                    piece_type: PieceType::Queen,
                },
                Coordinate::D8,
            ),
            (
                Piece {
                    color: Color::Black,
                    piece_type: PieceType::Queen,
                },
                Coordinate::D8,
            ),
        ];

        for (obstructing_piece, obstructing_square) in &obstructing_pieces {
            let mut board = Board::new_empty();
            board.enable_castling(Color::Black, false);
            let king = Piece {
                color: Color::Black,
                piece_type: PieceType::King,
            };
            let rook = Piece {
                color: Color::Black,
                piece_type: PieceType::Rook,
            };

            board.place_piece(Coordinate::E8, king);
            board.place_piece(Coordinate::A8, rook);
            board.place_piece(*obstructing_square, *obstructing_piece);

            let moves = generate_castling(&board, Coordinate::E8);

            assert_eq!(moves.len(), 0);
        }
    }

    #[test]
    fn generate_black_kingside_castling_with_attacked_castling_squares() {
        let attacking_pieces = [
            (
                Piece {
                    color: Color::White,
                    piece_type: PieceType::Bishop,
                },
                Coordinate::E6,
            ),
            (
                Piece {
                    color: Color::White,
                    piece_type: PieceType::Queen,
                },
                Coordinate::G3,
            ),
        ];

        for (attacking_piece, attacking_square) in &attacking_pieces {
            let mut board = Board::new_empty();
            board.enable_castling(Color::Black, true);
            let king = Piece {
                color: Color::Black,
                piece_type: PieceType::King,
            };
            let rook = Piece {
                color: Color::Black,
                piece_type: PieceType::Rook,
            };

            board.place_piece(Coordinate::E8, king);
            board.place_piece(Coordinate::H8, rook);
            board.place_piece(*attacking_square, *attacking_piece);

            let moves = generate_castling(&board, Coordinate::E8);

            assert_eq!(moves.len(), 0);
        }
    }

    #[test]
    fn generate_black_queenside_castling_with_attacked_castling_squares() {
        let attacking_pieces = [
            (
                Piece {
                    color: Color::White,
                    piece_type: PieceType::Bishop,
                },
                Coordinate::A6,
            ),
            (
                Piece {
                    color: Color::White,
                    piece_type: PieceType::Queen,
                },
                Coordinate::D4,
            ),
        ];

        for (attacking_piece, attacking_square) in &attacking_pieces {
            let mut board = Board::new_empty();
            board.enable_castling(Color::Black, false);
            let king = Piece {
                color: Color::Black,
                piece_type: PieceType::King,
            };
            let rook = Piece {
                color: Color::Black,
                piece_type: PieceType::Rook,
            };

            board.place_piece(Coordinate::E8, king);
            board.place_piece(Coordinate::A8, rook);
            board.place_piece(*attacking_square, *attacking_piece);

            let moves = generate_castling(&board, Coordinate::E8);

            assert_eq!(moves.len(), 0);
        }
    }

    #[test]
    fn generate_black_queenside_castling_with_controlled_b8_square() {
        let mut board = Board::new_empty();
        board.enable_castling(Color::Black, false);
        let king = Piece {
            color: Color::Black,
            piece_type: PieceType::King,
        };
        let rook = Piece {
            color: Color::Black,
            piece_type: PieceType::Rook,
        };
        let enemy_rook = Piece {
            color: Color::White,
            piece_type: PieceType::Rook,
        };

        board.place_piece(Coordinate::E8, king);
        board.place_piece(Coordinate::A8, rook);
        board.place_piece(Coordinate::B5, enemy_rook);

        let moves = generate_castling(&board, Coordinate::E8);

        assert_eq!(moves.len(), 1);
        assert!(moves.contains(&Move::new_castling(Color::Black, false)));
    }

    #[test]
    #[should_panic]
    fn generate_black_kingside_castling_with_missing_rook() {
        let mut board = Board::new_empty();
        board.enable_castling(Color::Black, true);
        let king = Piece {
            color: Color::Black,
            piece_type: PieceType::King,
        };

        board.place_piece(Coordinate::E8, king);

        generate_castling(&board, Coordinate::E8);
    }

    #[test]
    #[should_panic]
    fn generate_black_queenside_castling_with_missing_rook() {
        let mut board = Board::new_empty();
        board.enable_castling(Color::Black, false);
        let king = Piece {
            color: Color::Black,
            piece_type: PieceType::King,
        };

        board.place_piece(Coordinate::E8, king);

        generate_castling(&board, Coordinate::E8);
    }

    #[test]
    fn king_cannot_move_to_attacked_square() {
        let mut board = Board::new_empty();
        let king = Piece {
            color: Color::White,
            piece_type: PieceType::King,
        };
        let enemy_rook = Piece {
            color: Color::Black,
            piece_type: PieceType::Rook,
        };

        board.place_piece(Coordinate::E4, king);

        // Without the obstruction of enemy pieces, the king can go to D5, E5 and F5
        let initial_dest_squares = generate_legal_moves(&board)
            .iter()
            .map(|m| m.dest)
            .collect::<Vec<Coordinate>>();

        assert!(initial_dest_squares.contains(&Coordinate::D5));
        assert!(initial_dest_squares.contains(&Coordinate::E5));
        assert!(initial_dest_squares.contains(&Coordinate::F5));

        // The rook controls D5, E5 and F5 from A5
        board.place_piece(Coordinate::A5, enemy_rook);

        // The king can no longer go to those three squares
        let final_dest_squares = generate_legal_moves(&board)
            .iter()
            .map(|m| m.dest)
            .collect::<Vec<Coordinate>>();

        assert!(!final_dest_squares.contains(&Coordinate::D5));
        assert!(!final_dest_squares.contains(&Coordinate::E5));
        assert!(!final_dest_squares.contains(&Coordinate::F5));
    }

    #[test]
    fn queen_cannot_move_away_if_it_covers_the_king() {
        let mut board = Board::new_empty();
        let king = Piece {
            color: Color::White,
            piece_type: PieceType::King,
        };
        let queen = Piece {
            color: Color::White,
            piece_type: PieceType::Queen,
        };
        let enemy_rook = Piece {
            color: Color::Black,
            piece_type: PieceType::Rook,
        };

        board.place_piece(Coordinate::E4, king);
        board.place_piece(Coordinate::E5, queen);
        board.update_board_state();

        // Without the enemy rook, the queen is free to move.
        let initial_dest_squares = generate_legal_moves(&board)
            .iter()
            .filter(|m| m.piece == queen)
            .map(|m| m.dest)
            .collect::<Vec<Coordinate>>();

        assert!(initial_dest_squares.len() != 0);

        board.place_piece(Coordinate::E8, enemy_rook);
        board.update_board_state();

        let final_dest_squares = generate_legal_moves(&board)
            .iter()
            .filter(|m| m.piece == queen)
            .map(|m| m.dest)
            .collect::<Vec<Coordinate>>();

        assert_eq!(final_dest_squares.len(), 3);
        // The queen can move while still covering the king or
        // capture the attacking piece
        assert!(initial_dest_squares.contains(&Coordinate::E6));
        assert!(initial_dest_squares.contains(&Coordinate::E7));
        assert!(initial_dest_squares.contains(&Coordinate::E8));
    }

    #[test]
    fn bishop_cannot_move_away_if_it_covers_the_king() {
        let mut board = Board::new_empty();
        let king = Piece {
            color: Color::White,
            piece_type: PieceType::King,
        };
        let bishop = Piece {
            color: Color::White,
            piece_type: PieceType::Bishop,
        };
        let enemy_rook = Piece {
            color: Color::Black,
            piece_type: PieceType::Rook,
        };

        board.place_piece(Coordinate::E4, king);
        board.place_piece(Coordinate::E5, bishop);
        board.update_board_state();

        // Without the enemy rook, the bishop is free to move.
        let initial_dest_squares = generate_legal_moves(&board)
            .iter()
            .filter(|m| m.piece == bishop)
            .map(|m| m.dest)
            .collect::<Vec<Coordinate>>();

        assert!(initial_dest_squares.len() != 0);

        board.place_piece(Coordinate::E8, enemy_rook);
        board.update_board_state();

        let final_dest_squares = generate_legal_moves(&board)
            .iter()
            .filter(|m| m.piece == bishop)
            .map(|m| m.dest)
            .collect::<Vec<Coordinate>>();

        // The bishop cannot move at all
        assert_eq!(final_dest_squares.len(), 0);
    }

    #[test]
    fn forced_to_deal_with_checks() {
        let mut board = Board::new_empty();
        let king = Piece {
            color: Color::White,
            piece_type: PieceType::King,
        };
        let bishop = Piece {
            color: Color::White,
            piece_type: PieceType::Bishop,
        };
        let knight = Piece {
            color: Color::White,
            piece_type: PieceType::Knight,
        };
        let rook = Piece {
            color: Color::White,
            piece_type: PieceType::Rook,
        };
        let enemy_rook = Piece {
            color: Color::Black,
            piece_type: PieceType::Rook,
        };

        board.place_piece(Coordinate::E1, king);
        board.place_piece(Coordinate::D1, bishop);
        board.place_piece(Coordinate::G1, knight);
        board.place_piece(Coordinate::A8, rook);
        board.place_piece(Coordinate::E8, enemy_rook);
        board.update_board_state();

        assert!(board.is_in_check());

        let moves = generate_legal_moves(&board);
        assert_eq!(moves.len(), 6);
        // King walks away from the check
        assert!(moves.contains(&Move::new(Coordinate::E1, Coordinate::D2, king)));
        assert!(moves.contains(&Move::new(Coordinate::E1, Coordinate::F1, king)));
        assert!(moves.contains(&Move::new(Coordinate::E1, Coordinate::F2, king)));

        // Knight or bishop blocks
        assert!(moves.contains(&Move::new(Coordinate::D1, Coordinate::E2, bishop)));
        assert!(moves.contains(&Move::new(Coordinate::G1, Coordinate::E2, knight)));

        // Capture the attacking piece
        assert!(moves.contains(&Move::new_capture(
            Coordinate::A8,
            Coordinate::E8,
            rook,
            PieceType::Rook
        )));
    }

    #[test]
    fn get_square_attackers() {
        let mut board = Board::new_empty();
        let pieces = [
            (PieceType::Queen, Color::Black, Coordinate::B7),
            (PieceType::Rook, Color::Black, Coordinate::E8),
            (PieceType::Pawn, Color::Black, Coordinate::E7),
            (PieceType::Pawn, Color::White, Coordinate::C6),
            (PieceType::Knight, Color::Black, Coordinate::F6),
            (PieceType::Pawn, Color::Black, Coordinate::F5),
            (PieceType::Queen, Color::Black, Coordinate::A4),
            (PieceType::King, Color::Black, Coordinate::E5),
            (PieceType::Pawn, Color::Black, Coordinate::C2),
            (PieceType::Bishop, Color::Black, Coordinate::B1),
            (PieceType::Rook, Color::Black, Coordinate::E1),
            (PieceType::Bishop, Color::Black, Coordinate::G2),
        ];

        for (pt, color, coord) in pieces {
            board.place_piece(coord, Piece::new(color, pt));
        }

        assert_eq!(
            get_attackers_of_square_bb(
                &board,
                Coordinate::E4,
                Color::Black,
                board.get_all_pieces_bb()
            ),
            Coordinate::A4.to_bb()
                | Coordinate::E5.to_bb()
                | Coordinate::G2.to_bb()
                | Coordinate::E1.to_bb()
                | Coordinate::F5.to_bb()
                | Coordinate::F6.to_bb()
        );
    }
}
