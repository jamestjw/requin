use crate::board::*;
use lazy_static::lazy_static;
use regex::Regex;

pub fn parse_fen(fen_str: String) -> Result<Board, &'static str> {
    lazy_static! {
        static ref FEN_REGEX: Regex =
            Regex::new(r"\s*([prnbqkPRNBQK12345678]{1,8}(?:/[prnbqkPRNBQK12345678]{1,8}){7})\s+(w|b)\s+([KQkq]{1,4}|-)\s+(-|[a-h][1-8])\s(\d+\s\d+)").unwrap();
    }

    match FEN_REGEX.captures(&fen_str) {
        Some(matches) => {
            let mut board = Board::new_empty();
            // Handle piece positions
            // FEN notation describes the board from the
            // 8th rank to the 1st rank
            let mut rank = 8;
            for row in matches[1].split("/") {
                let mut file = 1;
                let mut last_c_was_digit = false;

                for c in row.chars() {
                    match c.to_string().parse::<usize>() {
                        Ok(digit) => {
                            // Ensure that there aren't two consecutive digits in a rank
                            if last_c_was_digit {
                                return Err("Two consecutive digits");
                            }
                            last_c_was_digit = true;
                            file += digit;
                        }
                        Err(_) => {
                            last_c_was_digit = false;

                            // Verify that the notation does not go out of bounds
                            if file > 8 {
                                return Err("Too many pieces in a rank");
                            }

                            let piece_type = fen_piece_name_to_piece_type(&c.to_string());
                            let piece_color = fen_piece_name_to_piece_color(&c.to_string());

                            let coord = Coordinate::new_from_rank_file(rank, file);
                            let piece = Piece {
                                color: piece_color,
                                piece_type,
                            };
                            board.place_piece(coord, piece);

                            file += 1;
                        }
                    }
                }
                rank -= 1;
            }

            // Handle player turn
            board.set_player_color(match &matches[2] {
                "w" => Color::White,
                "b" => Color::Black,
                _ => panic!("Invalid player color"),
            });

            // Handle castling rights
            for c in matches[3].chars() {
                match c {
                    'K' => board.enable_castling(Color::White, true),
                    'Q' => board.enable_castling(Color::White, false),
                    'k' => board.enable_castling(Color::Black, true),
                    'q' => board.enable_castling(Color::Black, false),
                    '-' => {
                        board.disable_castling(Color::White, true);
                        board.disable_castling(Color::White, false);
                        board.disable_castling(Color::Black, true);
                        board.disable_castling(Color::Black, false);
                    }
                    _ => panic!("Invalid castling rights"),
                }
            }

            // Handle en passant target square
            if &matches[4] != "-" {
                board.set_en_passant_square(Coordinate::new_from_algebraic_notation(&matches[4]));
            }

            board.init();

            Ok(board)
        }
        None => Err("Syntax error"),
    }
}

fn fen_piece_name_to_piece_type(name: &str) -> PieceType {
    match name {
        "P" | "p" => PieceType::Pawn,
        "K" | "k" => PieceType::King,
        "Q" | "q" => PieceType::Queen,
        "N" | "n" => PieceType::Knight,
        "R" | "r" => PieceType::Rook,
        "B" | "b" => PieceType::Bishop,
        _ => panic!("Invalid FEN piece type"),
    }
}

fn fen_piece_name_to_piece_color(name: &str) -> Color {
    match name {
        "P" | "K" | "Q" | "N" | "R" | "B" => Color::White,
        "p" | "k" | "q" | "n" | "r" | "b" => Color::Black,
        _ => panic!("Invalid FEN piece type"),
    }
}

#[cfg(test)]
mod fen_parser_tests {
    use super::*;
    use crate::r#move::Move;

    #[test]
    fn parse_invalid_fens() {
        let faulty_fens = [
            (
                "rnbqkbnr/pppppppp/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", // Missing a row
                "Syntax error",
            ),
            (
                "rnbqkbnr/pppppppp/44/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", // Two conseutive digits on the 6th rank
                "Two consecutive digits",
            ),
            (
                "rnbqkbnr/ppppp2pp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", // 9 pieces on the 7th rank
                "Too many pieces in a rank",
            ),
        ];
        for (fen, err_msg) in faulty_fens {
            match parse_fen(fen.to_string()) {
                Ok(_) => panic!("Should have failed"),
                Err(e) => assert_eq!(err_msg, e),
            }
        }
    }

    #[test]
    fn parse_starting_position() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        match parse_fen(fen.to_string()) {
            Ok(res) => {
                assert_eq!(res, Board::new_starting_pos());
            }
            Err(_) => {
                panic!("Should have successfully parsed base position");
            }
        }
    }

    #[test]
    fn parse_sicilian_first_moves() {
        let mut board = Board::new_starting_pos();
        let moves_fens = [
            (
                Move::new(
                    Coordinate::E2,
                    Coordinate::E4,
                    Piece::new(Color::White, PieceType::Pawn),
                ),
                "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1",
            ),
            (
                Move::new(
                    Coordinate::C7,
                    Coordinate::C5,
                    Piece::new(Color::Black, PieceType::Pawn),
                ),
                "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2",
            ),
            (
                Move::new(
                    Coordinate::G1,
                    Coordinate::F3,
                    Piece::new(Color::White, PieceType::Knight),
                ),
                "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2",
            ),
            (
                Move::new(
                    Coordinate::D7,
                    Coordinate::D6,
                    Piece::new(Color::Black, PieceType::Pawn),
                ),
                "rnbqkbnr/pp2pppp/3p4/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 0 3",
            ),
            (
                Move::new(
                    Coordinate::D2,
                    Coordinate::D4,
                    Piece::new(Color::White, PieceType::Pawn),
                ),
                "rnbqkbnr/pp2pppp/3p4/2p5/3PP3/5N2/PPP2PPP/RNBQKB1R b KQkq d3 0 3",
            ),
            (
                Move::new_capture(
                    Coordinate::C5,
                    Coordinate::D4,
                    Piece::new(Color::Black, PieceType::Pawn),
                    PieceType::Pawn,
                ),
                "rnbqkbnr/pp2pppp/3p4/8/3pP3/5N2/PPP2PPP/RNBQKB1R w KQkq - 0 4",
            ),
            (
                Move::new_capture(
                    Coordinate::F3,
                    Coordinate::D4,
                    Piece::new(Color::White, PieceType::Knight),
                    PieceType::Pawn,
                ),
                "rnbqkbnr/pp2pppp/3p4/8/3NP3/8/PPP2PPP/RNBQKB1R b KQkq - 0 4",
            ),
        ];

        for (m, fen) in moves_fens {
            board.apply_move(&m);
            assert_eq!(board, parse_fen(fen.to_string()).unwrap());
        }
    }
}
