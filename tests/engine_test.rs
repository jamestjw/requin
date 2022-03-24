use requin::board::*;
use requin::engine::Searcher;
use requin::game::Game;
use requin::parser::parse_fen;
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
    let expected_move =
        Move::new_capture(Coordinate::A1, Coordinate::A8, white_rook, PieceType::Rook);

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
    let expected_move =
        Move::new_capture(Coordinate::A8, Coordinate::A1, black_rook, PieceType::Rook);

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
    let expected_move = Move::new_capture(
        Coordinate::D4,
        Coordinate::H4,
        white_rook,
        PieceType::Bishop,
    );

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
    let expected_move = Move::new_capture(
        Coordinate::C2,
        Coordinate::A2,
        black_rook,
        PieceType::Bishop,
    );

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
    let expected_move = Move::new(Coordinate::D2, Coordinate::D8, white_rook);

    assert_eq!(best_move.unwrap(), expected_move);
}

#[test]
fn white_mate_in_one() {
    let board = parse_fen(String::from(
        "r2qkbnr/1b5p/p1n2p2/1pN1pNp1/1P1pP3/1Q4P1/PBPPBP1P/R3K2R w KQkq - 0 1",
    ))
    .unwrap();
    let game = Game::new(board);
    let mut searcher = Searcher::new(game, 1, 32);
    let best_move = searcher.get_best_move();
    let expected_move = Move::new(
        Coordinate::E2,
        Coordinate::H5,
        Piece::new(Color::White, PieceType::Bishop),
    );

    assert_eq!(best_move.unwrap(), expected_move);
}

#[test]
fn black_mate_in_one() {
    let board = parse_fen(String::from(
        "5rk1/NQp1q1pp/2B1p3/4P1p1/B1p2P2/N1P2R1K/B7/Q4nr1 b - - 0 1",
    ))
    .unwrap();
    let game = Game::new(board);
    let mut searcher = Searcher::new(game, 1, 32);
    let best_move = searcher.get_best_move();
    let expected_move = Move::new(
        Coordinate::G5,
        Coordinate::G4,
        Piece::new(Color::Black, PieceType::Pawn),
    );

    assert_eq!(best_move.unwrap(), expected_move);
}

#[test]
fn white_mate_in_two() {
    let board = parse_fen(String::from(
        "r1bq3r/ppp1nQ2/2kp1N2/2b3n1/4P3/8/P2N1PPP/1RR3K1 w - - 0 1",
    ))
    .unwrap();
    let game = Game::new(board);
    let mut searcher = Searcher::new(game, 3, 32);
    let best_move = searcher.get_best_move();
    let expected_move = Move::new(
        Coordinate::F7,
        Coordinate::D5,
        Piece::new(Color::White, PieceType::Queen),
    );

    assert_eq!(best_move.unwrap(), expected_move);
}

#[test]
fn white_mate_in_two_v2() {
    let board = parse_fen(String::from(
        "8/2p2p2/1nR4p/4k2K/1N2p2N/3p1n2/3Q2b1/b4RB1 w - - 0 1",
    ))
    .unwrap();
    let game = Game::new(board);
    let mut searcher = Searcher::new(game, 3, 32);
    let best_move = searcher.get_best_move();
    let expected_move = Move::new(
        Coordinate::D2,
        Coordinate::E1,
        Piece::new(Color::White, PieceType::Queen),
    );

    assert_eq!(best_move.unwrap(), expected_move);
}

#[test]
fn black_mate_in_two() {
    let board = parse_fen(String::from(
        "3r2k1/2P2p1p/4p1p1/8/3b1Q2/8/3p1RPP/3q1BK1 b - - 0 1",
    ))
    .unwrap();
    let game = Game::new(board);
    let mut searcher = Searcher::new(game, 3, 32);
    let best_move = searcher.get_best_move();
    let expected_move = Move::new_capture(
        Coordinate::D1,
        Coordinate::F1,
        Piece::new(Color::Black, PieceType::Queen),
        PieceType::Bishop,
    );

    assert_eq!(best_move.unwrap(), expected_move);
}

#[test]
fn black_mate_in_three() {
    let board = parse_fen(String::from(
        "8/2b2r1P/2P4k/1pK3n1/1N1R1N2/nqp5/8/8 b - - 0 1",
    ))
    .unwrap();
    let game = Game::new(board);
    let mut searcher = Searcher::new(game, 5, 32);
    let best_move = searcher.get_best_move();
    let expected_move = Move::new(
        Coordinate::F7,
        Coordinate::F5,
        Piece::new(Color::Black, PieceType::Rook),
    );

    assert_eq!(best_move.unwrap(), expected_move);
}
