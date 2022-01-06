use crate::board::{Board, Coordinate, PieceType};
use crate::r#move::Move;

// Generate all legal moves for a particular pawn
fn generate_pawn_moves(board: &Board, src: Coordinate) -> Vec<Move> {
    let piece = board.get_from_coordinate(src).unwrap();
    let front_square = src.vertical_offset(1, board.is_white_turn());
    let mut res = vec![];

    // TODO: Handle illegal board positions, e.g. when pawns are on the last rank

    // Handle captures
    for side_square in front_square.side_squares() {
        match board.get_from_coordinate(side_square) {
            Some(p) => {
                if p.color == piece.color.other_color() {
                    res.push(Move::new(src, side_square, piece, true));
                }
            }
            None => {}
        }
    }

    // TODO: Handle en passant

    // Handle pawn advances

    // If the square in front of the pawn is occupied, it may not advance.
    if board.is_square_occupied(front_square) {
        return res;
    }

    res.push(Move::new(src, front_square, piece, false));

    // Check if the pawn is still on its starting square
    if board.is_white_turn() && src.is_in_rank(2) || !board.is_white_turn() && src.is_in_rank(7) {
        let dest_square = src.vertical_offset(2, board.is_white_turn());
        if !board.is_square_occupied(dest_square) {
            res.push(Move::new(src, dest_square, piece, false));
        }
    }

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
            Move::new(piece_coord, Coordinate::E3, pawn, false),
            Move::new(piece_coord, Coordinate::E4, pawn, false)
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
            .eq(vec![Move::new(piece_coord, Coordinate::E3, pawn, false),]));
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
            Move::new(piece_coord, Coordinate::D5, pawn, true),
            Move::new(piece_coord, Coordinate::F5, pawn, true),
            Move::new(piece_coord, Coordinate::E5, pawn, false),
        ]));
    }
}
