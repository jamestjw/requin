use crate::board::{Board, Coordinate, PieceType};
use crate::r#move::Move;

// Generate all legal moves for a particular pawn
fn generate_pawn_moves(board: &Board, src: Coordinate) -> Vec<Move> {
    let piece = board.get_from_coordinate(src).unwrap();
    let front_square = src.vertical_offset(1, board.is_white_turn());
    let mut res = vec![];

    // TODO: Check if squares in front are occupied
    res.push(Move::new(src, front_square, piece, false));

    // Check if the pawn is still on its starting square
    if board.is_white_turn() && src.is_in_rank(2) || !board.is_white_turn() && src.is_in_rank(7) {
        res.push(Move::new(
            src,
            src.vertical_offset(2, board.is_white_turn()),
            piece,
            false,
        ));
    }

    // TODO: Captures and en passant

    res
}

fn generate_bishop_moves(board: &Board, src: Coordinate) -> Vec<Move> {
    let res = vec![];
    res
}

fn generate_knight_moves(board: &Board, src: Coordinate) -> Vec<Move> {
    let res = vec![];
    res
}

fn generate_rook_moves(board: &Board, src: Coordinate) -> Vec<Move> {
    let res = vec![];
    res
}

fn generate_king_moves(board: &Board, src: Coordinate) -> Vec<Move> {
    let res = vec![];
    res
}

fn generate_queen_moves(board: &Board, src: Coordinate) -> Vec<Move> {
    let res = vec![];
    res
}

// Generate all legal moves given a certain board
pub fn generate_moves(board: &Board) -> Vec<Move> {
    let mut res = vec![];

    for (coord, piece) in board.get_player_pieces(board.get_player_color()) {
        let piece_moves = match piece.piece_type {
            PieceType::Pawn => generate_pawn_moves(board, coord),
            PieceType::Knight => generate_knight_moves(board, coord),
            PieceType::Bishop => generate_bishop_moves(board, coord),
            PieceType::Rook => generate_rook_moves(board, coord),
            PieceType::King => generate_king_moves(board, coord),
            PieceType::Queen => generate_queen_moves(board, coord),
        };
        res.extend(piece_moves);
    }
    res
}
