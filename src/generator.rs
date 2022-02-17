use crate::board::{AdjacencyTable, Board, Color, Coordinate, Direction, PieceType};
use crate::r#move::Move;
use std::convert::TryFrom;

lazy_static! {
    static ref ADJACENCY_TABLE: AdjacencyTable = AdjacencyTable::new();
    static ref KNIGHT_MOVES_TABLE: KnightMovesTable = KnightMovesTable::new();
}

struct KnightMovesTable {
    table: Vec<Vec<Coordinate>>,
}

impl KnightMovesTable {
    pub fn new() -> Self {
        let mut t = KnightMovesTable {
            table: std::iter::repeat(vec![]).take(64).collect::<Vec<_>>(),
        };

        for i in 0..64 {
            let coord = Coordinate::try_from(i as u8).unwrap();
            let rank = coord.get_rank();
            let file = coord.get_file();

            if rank < 7 && file < 8 {
                //   ___>
                //   |
                //   |
                t.set(coord, Coordinate::try_from(i + 17).unwrap());
            }

            if rank < 8 && file < 7 {
                //   ____>
                //   |
                t.set(coord, Coordinate::try_from(i + 10).unwrap());
            }

            if rank < 7 && file > 1 {
                //  <___
                //     |
                //     |
                t.set(coord, Coordinate::try_from(i + 15).unwrap());
            }

            if rank < 8 && file > 2 {
                //    <|
                //     |______
                t.set(coord, Coordinate::try_from(i + 6).unwrap());
            }

            if rank > 2 && file < 8 {
                //     |
                //     |
                //  <__|
                t.set(coord, Coordinate::try_from(i - 15).unwrap());
            }

            if rank > 1 && file < 7 {
                //  |
                //  |____>
                t.set(coord, Coordinate::try_from(i - 6).unwrap());
            }

            if rank > 2 && file > 1 {
                //    |
                //    |
                // <__|
                t.set(coord, Coordinate::try_from(i - 17).unwrap());
            }

            if rank > 1 && file > 2 {
                //      |
                // <____|
                t.set(coord, Coordinate::try_from(i - 10).unwrap());
            }
        }

        t
    }

    pub fn set(&mut self, src: Coordinate, dest: Coordinate) {
        self.table[src as usize].push(dest);
    }

    pub fn get(&self, src: Coordinate) -> &Vec<Coordinate> {
        &self.table[src as usize]
    }
}

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
                    res.push(Move::new(src, front_side_square, piece, true));
                }
            }
            None => {}
        }
    }

    // Handle en passant
    for side_square in src.side_squares() {
        match board.get_from_coordinate(side_square) {
            Some(p) => {
                if p.color == piece.color.other_color() && p.piece_type == PieceType::Pawn {
                    match board.last_move {
                        Some(m) => {
                            if m.dest == side_square && m.piece == p && m.eligible_for_en_passant()
                            {
                                let mut en_passant_move = Move::new(
                                    src,
                                    side_square.vertical_offset(1, piece.color.is_white()),
                                    piece,
                                    true,
                                );
                                en_passant_move.is_en_passant = true;
                                res.push(en_passant_move);
                            }
                        }
                        None => {}
                    }
                }
            }
            None => {}
        }
    }

    // If the square in front of the pawn is occupied, it may not advance.
    if board.is_square_occupied(front_square) {
        return res;
    }

    res.push(Move::new(src, front_square, piece, false));

    // Check if the pawn is still on its starting square
    if piece_color.is_white() && src.is_in_rank(2) || !piece_color.is_white() && src.is_in_rank(7) {
        let dest_square = src.vertical_offset(2, board.is_white_turn());
        if !board.is_square_occupied(dest_square) {
            res.push(Move::new(src, dest_square, piece, false));
        }
    }

    // Handle pawn promotions
    for i in 0..res.len() {
        let sq = res[i].dest;
        if piece_color.is_white() && sq.is_in_rank(8) || !piece_color.is_white() && sq.is_in_rank(1)
        {
            res[i].is_promotion = true;
        }
    }

    res
}

fn generate_bishop_style_moves(board: &Board, src: Coordinate) -> Vec<Move> {
    let piece = board.get_from_coordinate(src).unwrap();
    let mut res = vec![];

    for dir in Direction::diagonal_iterator() {
        let mut curr_square = src;

        loop {
            if let Some(dest_square) = ADJACENCY_TABLE.get(curr_square, *dir) {
                // Check if square is occupied
                if let Some(occupying_piece) = board.get_from_coordinate(dest_square) {
                    // Add a move for the capture of an opposing color piece
                    if occupying_piece.color != piece.color {
                        res.push(Move::new(src, dest_square, piece, true));
                    }

                    // The bishop may not jump over a piece hence we stop the search
                    break;
                } else {
                    res.push(Move::new(src, dest_square, piece, false));
                    curr_square = dest_square;
                }
            } else {
                break;
            }
        }
    }

    res
}

fn generate_knight_moves(board: &Board, src: Coordinate) -> Vec<Move> {
    let piece = board.get_from_coordinate(src).unwrap();
    let mut res = vec![];

    for dest_square in KNIGHT_MOVES_TABLE.get(src) {
        match board.get_from_coordinate(*dest_square) {
            Some(occupant) => {
                // Capture an enemy piece
                if occupant.color != piece.color {
                    res.push(Move::new(src, *dest_square, piece, true));
                }
            }
            None => {
                res.push(Move::new(src, *dest_square, piece, false));
            }
        }
    }

    res
}

fn generate_rook_style_moves(board: &Board, src: Coordinate) -> Vec<Move> {
    let piece = board.get_from_coordinate(src).unwrap();
    let mut res = vec![];

    for dir in Direction::horizontal_vertical_iterator() {
        let mut curr_square = src;

        loop {
            if let Some(dest_square) = ADJACENCY_TABLE.get(curr_square, *dir) {
                // Check if square is occupied
                if let Some(occupying_piece) = board.get_from_coordinate(dest_square) {
                    // Add a move for the capture of an opposing color piece
                    if occupying_piece.color != piece.color {
                        res.push(Move::new(src, dest_square, piece, true));
                    }

                    // The rook may not jump over a piece hence we stop the search
                    break;
                } else {
                    res.push(Move::new(src, dest_square, piece, false));
                    curr_square = dest_square;
                }
            } else {
                break;
            }
        }
    }

    res
}

fn generate_king_moves(board: &Board, src: Coordinate, with_castling: bool) -> Vec<Move> {
    let piece = board.get_from_coordinate(src).unwrap();
    let mut res = vec![];

    for dir in Direction::iterator() {
        if let Some(dest_square) = ADJACENCY_TABLE.get(src, *dir) {
            // Check if square is occupied
            if let Some(occupying_piece) = board.get_from_coordinate(dest_square) {
                // Add a move for the capture of an opposing color piece
                if occupying_piece.color != piece.color {
                    res.push(Move::new(src, dest_square, piece, true));
                }
            } else {
                res.push(Move::new(src, dest_square, piece, false));
            }
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
    let enemy_controlled_squares =
        generate_players_controlled_squares(board, piece.color.other_color());

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
                    && !enemy_controlled_squares.contains(&Coordinate::F1)
                    && !enemy_controlled_squares.contains(&Coordinate::G1)
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
                    && !enemy_controlled_squares.contains(&Coordinate::F8)
                    && !enemy_controlled_squares.contains(&Coordinate::G8)
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
                    && !enemy_controlled_squares.contains(&Coordinate::C1)
                    && !enemy_controlled_squares.contains(&Coordinate::D1)
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
                    && !enemy_controlled_squares.contains(&Coordinate::C8)
                    && !enemy_controlled_squares.contains(&Coordinate::D8)
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
    let mut board_copy = board.clone();
    // Hypothetically apply the move
    board_copy.apply_move(m);
    let king_coord = board_copy.get_king_coordinate(color);

    // Check if the current player's king is in danger
    let moves = generate_moves(&board_copy);

    for m in moves {
        if m.dest == king_coord {
            return false;
        }
    }

    true
}

// Generate all moves given a certain board
pub fn generate_moves(board: &Board) -> Vec<Move> {
    let mut res = vec![];

    for (coord, piece) in board.get_player_pieces(board.get_player_color()) {
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

fn generate_pawn_controlled_squares(board: &Board, src: Coordinate) -> Vec<Coordinate> {
    let mut res = vec![];
    let piece = board.get_from_coordinate(src).unwrap();

    let directions = match piece.color {
        Color::White => [Direction::NW, Direction::NE],
        Color::Black => [Direction::SW, Direction::SE],
    };

    for direction in &directions {
        match ADJACENCY_TABLE.get(src, *direction) {
            Some(dest) => res.push(dest),
            None => {}
        }
    }
    res
}

// Generate all the squares controlled by a certain player
pub fn generate_players_controlled_squares(board: &Board, color: Color) -> Vec<Coordinate> {
    let mut res = vec![];

    for (coord, piece) in board.get_player_pieces(color) {
        if piece.piece_type == PieceType::Pawn {
            res.extend(generate_pawn_controlled_squares(board, coord));
        } else {
            let piece_moves = match piece.piece_type {
                PieceType::Knight => generate_knight_moves(board, coord),
                PieceType::Bishop => generate_bishop_style_moves(board, coord),
                PieceType::Rook => generate_rook_style_moves(board, coord),
                PieceType::King => generate_king_moves(board, coord, false),
                PieceType::Queen => generate_queen_moves(board, coord),
                _ => panic!("Unexpected piece type."),
            };

            for piece_move in piece_moves {
                let dest_square = piece_move.dest;
                res.push(dest_square)
            }
        }
    }

    // TODO: Remove duplicates
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
        let last_move = Move::new(Coordinate::F7, Coordinate::F5, capturable_piece, false);

        board.place_piece(piece_coord, pawn);
        board.place_piece(Coordinate::F5, capturable_piece);
        board.last_move = Some(last_move);

        let moves = generate_pawn_moves(&board, piece_coord);
        let mut expected_en_passant_move = Move::new(piece_coord, Coordinate::F6, pawn, true);
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
        let last_move = Move::new(Coordinate::F6, Coordinate::F5, capturable_piece, false);

        board.place_piece(piece_coord, pawn);
        board.place_piece(Coordinate::F5, capturable_piece);
        board.last_move = Some(last_move);

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
        let last_move = Move::new(Coordinate::F2, Coordinate::F4, capturable_piece, false);

        board.place_piece(piece_coord, pawn);
        board.place_piece(Coordinate::F4, capturable_piece);
        board.last_move = Some(last_move);

        let moves = generate_pawn_moves(&board, piece_coord);
        let mut expected_en_passant_move = Move::new(piece_coord, Coordinate::F3, pawn, true);
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
        let last_move = Move::new(Coordinate::F3, Coordinate::F4, capturable_piece, false);

        board.place_piece(piece_coord, pawn);
        board.place_piece(Coordinate::F4, capturable_piece);
        board.last_move = Some(last_move);

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

        let mut expected_move = Move::new(piece_coord, Coordinate::E8, pawn, false);
        expected_move.is_promotion = true;

        let moves = generate_pawn_moves(&board, piece_coord);

        assert!(moves.contains(&expected_move));
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

        let mut expected_move = Move::new(piece_coord, Coordinate::E1, pawn, false);
        expected_move.is_promotion = true;

        let moves = generate_pawn_moves(&board, piece_coord);

        assert!(moves.contains(&expected_move));
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
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D5, bishop, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::C6, bishop, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::B7, bishop, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::A8, bishop, false)));

        // Bottom left
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D3, bishop, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::C2, bishop, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::B1, bishop, false)));

        // Top right
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F5, bishop, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::G6, bishop, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::H7, bishop, false)));

        // Bottom right
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F3, bishop, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::G2, bishop, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::H1, bishop, false)));
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
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D5, bishop, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::C6, bishop, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::B7, bishop, true)));

        // Bottom left
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D3, bishop, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::C2, bishop, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::B1, bishop, false)));

        // Top right
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F5, bishop, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::G6, bishop, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::H7, bishop, false)));

        // Bottom right
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F3, bishop, true)));
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
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D5, bishop, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::C6, bishop, false)));

        // Bottom left
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D3, bishop, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::C2, bishop, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::B1, bishop, false)));

        // Top right
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F5, bishop, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::G6, bishop, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::H7, bishop, false)));
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
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E5, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E6, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E7, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E8, piece, false)));

        // Bottom
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E3, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E2, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E1, piece, false)));

        // Right
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F4, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::G4, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::H4, piece, false)));

        // Left
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D4, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::C4, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::B4, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::A4, piece, false)));
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
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E5, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E6, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E7, piece, true)));

        // Bottom
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E3, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E2, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E1, piece, false)));

        // Right
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F4, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::G4, piece, true)));

        // Left
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D4, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::C4, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::B4, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::A4, piece, false)));
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
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E5, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E6, piece, false)));

        // Bottom
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E3, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E2, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E1, piece, false)));

        // Right
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F4, piece, false)));

        // Left
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D4, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::C4, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::B4, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::A4, piece, false)));
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
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E5, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E6, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E7, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E8, piece, false)));

        // Bottom
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E3, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E2, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E1, piece, false)));

        // Right
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F4, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::G4, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::H4, piece, false)));

        // Left
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D4, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::C4, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::B4, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::A4, piece, false)));

        // Top left
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D5, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::C6, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::B7, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::A8, piece, false)));

        // Bottom left
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D3, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::C2, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::B1, piece, false)));

        // Top right
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F5, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::G6, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::H7, piece, false)));

        // Bottom right
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F3, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::G2, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::H1, piece, false)));
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
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E5, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E6, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E7, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E8, piece, false)));

        // Bottom
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E3, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E2, piece, true)));

        // Right
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F4, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::G4, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::H4, piece, false)));

        // Left
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D4, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::C4, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::B4, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::A4, piece, false)));

        // Top left
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D5, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::C6, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::B7, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::A8, piece, false)));

        // Bottom left
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D3, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::C2, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::B1, piece, false)));

        // Top right
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F5, piece, true)));

        // Bottom right
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F3, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::G2, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::H1, piece, false)));
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
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E5, piece, false)));

        // Bottom
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E3, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E2, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E1, piece, false)));

        // Right
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F4, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::G4, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::H4, piece, false)));

        // Left
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D4, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::C4, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::B4, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::A4, piece, false)));

        // Top left

        // Bottom left
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D3, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::C2, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::B1, piece, false)));

        // Top right
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F5, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::G6, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::H7, piece, false)));

        // Bottom right
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F3, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::G2, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::H1, piece, false)));
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

        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F6, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D6, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::C5, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::C3, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D2, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F2, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::G3, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::G5, piece, false)));
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

        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F6, piece, true)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D6, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::C5, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::C3, piece, true)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D2, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F2, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::G3, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::G5, piece, false)));
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

        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D6, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::C5, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D2, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F2, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::G3, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::G5, piece, false)));
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

        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D5, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E5, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F5, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D4, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F4, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D3, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E3, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F3, piece, false)));
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

        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D5, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E5, piece, true)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F5, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D4, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F4, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D3, piece, true)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E3, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F3, piece, false)));
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

        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D5, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F5, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::D4, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F4, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::E3, piece, false)));
        assert!(moves.contains(&Move::new(piece_coord, Coordinate::F3, piece, false)));
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
    fn generate_pawn_contolled_squares_white() {
        let mut board = Board::new_empty();
        let pawn = Piece {
            color: Color::White,
            piece_type: PieceType::Pawn,
        };

        board.place_piece(Coordinate::E4, pawn);
        let coords = generate_pawn_controlled_squares(&board, Coordinate::E4);

        assert_eq!(coords.len(), 2);
        assert!(coords.contains(&Coordinate::D5));
        assert!(coords.contains(&Coordinate::F5));
    }

    #[test]
    fn generate_pawn_contolled_squares_black() {
        let mut board = Board::new_empty();
        let pawn = Piece {
            color: Color::Black,
            piece_type: PieceType::Pawn,
        };

        board.place_piece(Coordinate::E5, pawn);
        let coords = generate_pawn_controlled_squares(&board, Coordinate::E5);

        assert_eq!(coords.len(), 2);
        assert!(coords.contains(&Coordinate::D4));
        assert!(coords.contains(&Coordinate::F4));
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

        // Without the enemy rook, the queen is free to move.
        let initial_dest_squares = generate_legal_moves(&board)
            .iter()
            .filter(|m| m.piece == queen)
            .map(|m| m.dest)
            .collect::<Vec<Coordinate>>();

        assert!(initial_dest_squares.len() != 0);

        board.place_piece(Coordinate::E8, enemy_rook);

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

        // Without the enemy rook, the bishop is free to move.
        let initial_dest_squares = generate_legal_moves(&board)
            .iter()
            .filter(|m| m.piece == bishop)
            .map(|m| m.dest)
            .collect::<Vec<Coordinate>>();

        assert!(initial_dest_squares.len() != 0);

        board.place_piece(Coordinate::E8, enemy_rook);

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

        let moves = generate_legal_moves(&board);
        assert_eq!(moves.len(), 6);
        // King walks away from the check
        assert!(moves.contains(&Move::new(Coordinate::E1, Coordinate::D2, king, false)));
        assert!(moves.contains(&Move::new(Coordinate::E1, Coordinate::F1, king, false)));
        assert!(moves.contains(&Move::new(Coordinate::E1, Coordinate::F2, king, false)));

        // Knight or bishop blocks
        assert!(moves.contains(&Move::new(Coordinate::D1, Coordinate::E2, bishop, false)));
        assert!(moves.contains(&Move::new(Coordinate::G1, Coordinate::E2, knight, false)));

        // Capture the attacking piece
        assert!(moves.contains(&Move::new(Coordinate::A8, Coordinate::E8, rook, true)));
    }
}
