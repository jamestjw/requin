use requin::board::*;
use requin::engine::Searcher;
use requin::game::Game;
use requin::r#move::Move;

#[test]
fn test_depth_one_best_move_white() {
    let mut board = Board::new_empty();

    let white_rook = Piece {
        color: Color::White,
        piece_type: PieceType::Rook,
    };
    let white_king = Piece {
        color: Color::White,
        piece_type: PieceType::King,
    };
    let black_rook = Piece {
        color: Color::Black,
        piece_type: PieceType::Rook,
    };
    let black_king = Piece {
        color: Color::Black,
        piece_type: PieceType::King,
    };

    // The obvious move is for the rook to capture the undefended rook.
    board.place_piece(Coordinate::E1, white_king);
    board.place_piece(Coordinate::E8, black_king);
    board.place_piece(Coordinate::A1, white_rook);
    board.place_piece(Coordinate::A8, black_rook);

    let game = Game::new(board);
    let mut searcher = Searcher::new(game, 1, 1);
    let best_move = searcher.get_best_move();
    let expected_move = Move::new(Coordinate::A1, Coordinate::A8, white_rook, true);

    assert_eq!(best_move.unwrap(), expected_move);
}

#[test]
fn test_depth_one_best_move_black() {
    let mut board = Board::new_empty();
    board.set_player_color(Color::Black);

    let white_rook = Piece {
        color: Color::White,
        piece_type: PieceType::Rook,
    };
    let white_king = Piece {
        color: Color::White,
        piece_type: PieceType::King,
    };
    let black_rook = Piece {
        color: Color::Black,
        piece_type: PieceType::Rook,
    };
    let black_king = Piece {
        color: Color::Black,
        piece_type: PieceType::King,
    };

    // The obvious move is for the rook to capture the undefended rook.
    board.place_piece(Coordinate::E1, white_king);
    board.place_piece(Coordinate::E8, black_king);
    board.place_piece(Coordinate::A1, white_rook);
    board.place_piece(Coordinate::A8, black_rook);

    let game = Game::new(board);
    let mut searcher = Searcher::new(game, 1, 1);
    let best_move = searcher.get_best_move();
    let expected_move = Move::new(Coordinate::A8, Coordinate::A1, black_rook, true);

    assert_eq!(best_move.unwrap(), expected_move);
}

#[test]
fn test_depth_two_best_move_white() {
    let mut board = Board::new_empty();

    let white_rook = Piece {
        color: Color::White,
        piece_type: PieceType::Rook,
    };
    let white_king = Piece {
        color: Color::White,
        piece_type: PieceType::King,
    };
    let black_bishop_1 = Piece {
        color: Color::Black,
        piece_type: PieceType::Bishop,
    };
    let black_bishop_2 = Piece {
        color: Color::Black,
        piece_type: PieceType::Bishop,
    };
    let black_king = Piece {
        color: Color::Black,
        piece_type: PieceType::King,
    };

    // The rook should capture the undefended bishop.
    board.place_piece(Coordinate::E1, white_king);
    board.place_piece(Coordinate::B5, black_king);
    board.place_piece(Coordinate::D4, white_rook);
    board.place_piece(Coordinate::A4, black_bishop_1);
    board.place_piece(Coordinate::H4, black_bishop_2);

    let game = Game::new(board);
    let mut searcher = Searcher::new(game, 2, 1);
    let best_move = searcher.get_best_move();
    let expected_move = Move::new(Coordinate::D4, Coordinate::H4, white_rook, true);

    assert_eq!(best_move.unwrap(), expected_move);
}

#[test]
fn test_depth_two_best_move_black() {
    let mut board = Board::new_empty();
    board.set_player_color(Color::Black);

    let white_bishop_1 = Piece {
        color: Color::White,
        piece_type: PieceType::Bishop,
    };
    let white_bishop_2 = Piece {
        color: Color::White,
        piece_type: PieceType::Bishop,
    };
    let white_king = Piece {
        color: Color::White,
        piece_type: PieceType::King,
    };
    let black_king = Piece {
        color: Color::Black,
        piece_type: PieceType::King,
    };
    let black_rook = Piece {
        color: Color::Black,
        piece_type: PieceType::Rook,
    };

    // The rook should capture the undefended bishop.
    board.place_piece(Coordinate::H1, white_king);
    board.place_piece(Coordinate::B5, black_king);
    board.place_piece(Coordinate::C2, black_rook);
    board.place_piece(Coordinate::H2, white_bishop_1);
    board.place_piece(Coordinate::A2, white_bishop_2);

    let game = Game::new(board);
    let mut searcher = Searcher::new(game, 2, 1);
    let best_move = searcher.get_best_move();
    let expected_move = Move::new(Coordinate::C2, Coordinate::A2, black_rook, true);

    assert_eq!(best_move.unwrap(), expected_move);
}

#[test]
fn test_depth_three_best_move_white() {
    let mut board = Board::new_empty();

    let white_rook = Piece {
        color: Color::White,
        piece_type: PieceType::Rook,
    };
    let white_king = Piece {
        color: Color::White,
        piece_type: PieceType::King,
    };
    let white_pawn = Piece {
        color: Color::White,
        piece_type: PieceType::Pawn,
    };
    let black_king = Piece {
        color: Color::Black,
        piece_type: PieceType::King,
    };
    let black_rook = Piece {
        color: Color::Black,
        piece_type: PieceType::Rook,
    };

    // The expected move is Rd8+, followed by promotion
    board.place_piece(Coordinate::F1, white_king);
    board.place_piece(Coordinate::A8, black_king);
    board.place_piece(Coordinate::D2, white_rook);
    board.place_piece(Coordinate::E7, white_pawn);
    board.place_piece(Coordinate::E8, black_rook);

    let game = Game::new(board);
    let mut searcher = Searcher::new(game, 3, 1);
    let best_move = searcher.get_best_move();
    let expected_move = Move::new(Coordinate::D2, Coordinate::D8, white_rook, false);

    assert_eq!(best_move.unwrap(), expected_move);
}
