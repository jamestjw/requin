use crate::board::{file_to_index, Board, Color, Coordinate, PieceType};
use crate::{generator::generate_legal_moves, r#move::Move};
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use std::io::{self, Write};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameState {
    InProgress,
    WhiteWon,
    BlackWon,
    Stalemate,
}

impl GameState {
    pub fn to_text(&self) -> &str {
        match self {
            GameState::InProgress => "Game is in progress",
            GameState::WhiteWon => "White has won by checkmate",
            GameState::BlackWon => "Black has won by checkmate",
            GameState::Stalemate => "Stalemate",
        }
    }
}

#[derive(Clone)]
pub struct Game {
    board_history: Vec<Board>,
    current_legal_moves: Option<Vec<Move>>, // Legal moves for the current board
    pub state: GameState,
    plies_from_last_irreversible_move: u32, // When this is zero, it implies that the last played move was irreversible
}

impl Game {
    pub fn new(starting_board: Board) -> Game {
        Game {
            board_history: vec![starting_board],
            current_legal_moves: Some(generate_legal_moves(&starting_board)),
            state: GameState::InProgress,
            plies_from_last_irreversible_move: 0,
        }
    }

    pub fn current_board(&self) -> &Board {
        // Board history should never be empty
        self.board_history.last().unwrap()
    }

    pub fn print_current_board(&self) {
        self.current_board().print();
    }

    fn find_legal_move(
        &self,
        piece_type: PieceType,
        dest_coord: Coordinate,
        src_rank: Option<usize>,
        src_file: Option<usize>,
        is_capture: bool,
        promotion_piece_type: Option<PieceType>,
    ) -> Result<Move, &'static str> {
        let mut candidate_moves = vec![];

        for m in self.current_legal_moves() {
            if m.piece.piece_type == piece_type
                && m.dest == dest_coord
                && m.is_capture == is_capture
            {
                candidate_moves.push(m.clone());
            }
        }

        match candidate_moves.len() {
            0 => Err("Illegal move"),
            1 => {
                let m = candidate_moves[0];
                match src_file {
                    Some(f) => {
                        if f != m.src.get_file() {
                            // Since the intended source file does not match that
                            // of the legal move.
                            return Err("Illegal move.");
                        }
                    }
                    None => {}
                }

                match src_rank {
                    Some(f) => {
                        if f != m.src.get_rank() {
                            // Since the intended source rank does not match that
                            // of the legal move.
                            return Err("Illegal move.");
                        }
                    }
                    None => {}
                }

                Ok(m)
            }
            _ => {
                // When there is ambiguity, use the src_rank, src_file
                // and promotion_pieces to identify the move
                if src_file.is_none() && src_rank.is_none() && promotion_piece_type.is_none() {
                    return Err("The move is ambiguous.");
                }

                let src_files: Vec<usize> =
                    candidate_moves.iter().map(|m| m.src.get_file()).collect();
                let src_ranks: Vec<usize> =
                    candidate_moves.iter().map(|m| m.src.get_rank()).collect();

                // Identify the source of the ambiguity, is it from the rank or file?
                let file_is_ambiguous = src_files.iter().unique().into_iter().count() != 1;
                let rank_is_ambiguous = src_ranks.iter().unique().into_iter().count() != 1;
                // TODO: Is there a better way of doing this?
                let promotion_piece_is_ambiguous = candidate_moves[0].is_promotion;

                // If there is ambiguity in both source file and source rank, we expect
                // the file to be used to disambiguate.
                if file_is_ambiguous {
                    match src_file {
                        Some(src_file) => {
                            // Identify the move that has the right file
                            match src_files.into_iter().position(|f| f == src_file) {
                                Some(idx) => Ok(candidate_moves[idx]),
                                None => Err("Invalid move: Illegal source square"),
                            }
                        }
                        None => {
                            return Err("Ambiguity in file of source square");
                        }
                    }
                } else if rank_is_ambiguous {
                    match src_rank {
                        Some(src_rank) => {
                            // Identify the move that has the right rank
                            match src_ranks.iter().position(|r| *r == src_rank) {
                                Some(idx) => Ok(candidate_moves[idx]),
                                None => Err("Invalid move: Illegal source square"),
                            }
                        }
                        None => {
                            return Err("Ambiguity in rank of source square");
                        }
                    }
                } else if promotion_piece_is_ambiguous {
                    match promotion_piece_type {
                        Some(promotion_piece_type) => {
                            // Identify the move that has the right promotion_piece_type
                            match candidate_moves
                                .iter()
                                .position(|r| r.promotes_to == Some(promotion_piece_type))
                            {
                                Some(idx) => Ok(candidate_moves[idx]),
                                None => Err("Invalid move: Illegal promotion piece"),
                            }
                        }
                        None => {
                            return Err("Ambiguity in promotion piece type");
                        }
                    }
                } else {
                    panic!("Unexpected error: No ambiguity found with multiple candidate moves.");
                }
            }
        }
    }

    // Prompts the user for a next move
    // When this method is called, we assume that there
    // are indeed legal moves in the position.
    pub fn get_next_move(&mut self) {
        loop {
            let mut move_string = String::new();

            print!("Next move:");

            io::stdout().flush().unwrap();

            match io::stdin().read_line(&mut move_string) {
                Ok(_) => {
                    println!();

                    // Remove whitespace
                    move_string.retain(|c| !c.is_whitespace());

                    let board = self.current_board();
                    let player_color = board.get_player_color();

                    let m = match move_string.as_ref() {
                        "O-O" | "0-0" => {
                            let tmp_m = Move::new_castling(player_color, true);

                            if self.current_legal_moves().contains(&tmp_m) {
                                tmp_m
                            } else {
                                println!("Error: Illegal move.");
                                continue;
                            }
                        }
                        "O-O-O" | "0-0-0" => {
                            let tmp_m = Move::new_castling(player_color, false);

                            if self.current_legal_moves().contains(&tmp_m) {
                                tmp_m
                            } else {
                                println!("Error: Illegal move.");
                                continue;
                            }
                        }
                        _ => {
                            let (
                                p_type,
                                src_rank,
                                src_file,
                                dest_coord,
                                is_capture,
                                promotion_piece_type,
                            ) = match parse_move_string(&move_string) {
                                Ok(res) => res,
                                Err(_) => {
                                    println!("Error: {}", move_string);
                                    continue;
                                }
                            };
                            match self.find_legal_move(
                                p_type,
                                dest_coord,
                                src_rank,
                                src_file,
                                is_capture,
                                promotion_piece_type,
                            ) {
                                Ok(mut m) => {
                                    if m.is_promotion {
                                        match promotion_piece_type {
                                            Some(ppt) => {
                                                m.promotes_to = Some(ppt);
                                                m
                                            }
                                            None => {
                                                println!("Error: Promotion piece type unspecified");
                                                continue;
                                            }
                                        }
                                    } else {
                                        m
                                    }
                                }
                                Err(e) => {
                                    println!("Error: {}", e);
                                    continue;
                                }
                            }
                        }
                    };

                    self.apply_move(&m);
                    break;
                }
                Err(_) => println!("Invalid move, please try again."),
            }
        }
    }

    fn generate_legal_moves(&mut self) {
        self.current_legal_moves = Some(generate_legal_moves(self.current_board()));
    }

    pub fn current_legal_moves(&self) -> &Vec<Move> {
        self.current_legal_moves
            .as_ref()
            .expect("Attempted to access legal moves before move generation")
    }

    // This function should be called before running a game
    pub fn init_game_board(&mut self) {
        self.generate_legal_moves();
    }

    // This must not be called if the side to move is in check
    pub fn apply_null_move(&mut self) {
        let mut new_board = self.current_board().clone();
        new_board.set_player_color(new_board.get_opposing_player_color());
        self.board_history.push(new_board);
        // Null moves are reversible
        self.plies_from_last_irreversible_move += 1;
        self.generate_legal_moves();
        if self.current_legal_moves().len() == 0 {
            if self.current_board().is_in_check() {
                match self.current_board().get_player_color() {
                    Color::White => {
                        self.state = GameState::BlackWon;
                    }
                    Color::Black => {
                        self.state = GameState::WhiteWon;
                    }
                }
            } else {
                self.state = GameState::Stalemate;
            }
        }
    }

    pub fn apply_move(&mut self, m: &Move) {
        let mut new_board = self.current_board().clone();
        new_board.apply_move(&m);
        self.board_history.push(new_board);

        // Check if the move is irreversible
        // TODO: Moves that change castling rights should also be included
        if m.is_capture || m.piece.piece_type == PieceType::Pawn || m.is_castling() {
            self.plies_from_last_irreversible_move = 0;
        } else {
            self.plies_from_last_irreversible_move += 1;
        }

        self.generate_legal_moves();

        if self.current_legal_moves().len() == 0 {
            if self.current_board().is_in_check() {
                match self.current_board().get_player_color() {
                    Color::White => {
                        self.state = GameState::BlackWon;
                    }
                    Color::Black => {
                        self.state = GameState::WhiteWon;
                    }
                }
            } else {
                self.state = GameState::Stalemate;
            }
        }
    }

    pub fn undo_move(&mut self) {
        self.board_history.pop();
        self.current_legal_moves = None;
        self.state = GameState::InProgress;
        self.plies_from_last_irreversible_move =
            self.plies_from_last_irreversible_move.saturating_sub(1);
    }

    pub fn is_game_over(&self) -> bool {
        self.state != GameState::InProgress
    }

    pub fn is_in_check(&self) -> bool {
        self.current_board().is_in_check()
    }

    pub fn is_threefold_repetition(&self) -> bool {
        if self.plies_from_last_irreversible_move < 8 {
            return false;
        }

        let current_zobrist = self.current_board().get_zobrist();
        let mut num_matches = 0;

        for ply_offset in (4..=self.plies_from_last_irreversible_move).step_by(2) {
            let index = self.board_history.len() - 1 - ply_offset as usize;

            match self.board_history.get(index) {
                Some(b) => {
                    if current_zobrist == b.get_zobrist() {
                        num_matches += 1;
                    }
                }
                None => {
                    // This should never occur
                    panic!("Missing board at index {}", index);
                }
            }

            if num_matches == 2 {
                return true;
            }
        }

        false
    }
}

// Returns the PieceType, an optional source rank, an optional source file,
// the destination coordinate, a boolean if the move is a capture and
// an optional promotion piece type.
// This function does not handle castling.
fn parse_move_string(
    move_string: &str,
) -> Result<
    (
        PieceType,
        Option<usize>,
        Option<usize>,
        Coordinate,
        bool,
        Option<PieceType>,
    ),
    &'static str,
> {
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r"^([NBRQK])?([a-h])?([1-8])?(x)?([a-h])([1-8])(=[NBRQ])?(\+|#)?$").unwrap();
    }

    match RE.captures(move_string) {
        Some(caps) => {
            let piece_type =
                PieceType::new_from_string(caps.get(1).map_or_else(|| "", |s| s.as_str())).unwrap();
            let source_file = caps.get(2).map(|f| file_to_index(f.as_str()));
            let source_rank = caps.get(3).map(|r| r.as_str().parse::<usize>().unwrap());
            let dest_file = file_to_index(caps.get(5).unwrap().as_str());
            let dest_rank = caps.get(6).unwrap().as_str().parse::<usize>().unwrap();
            let dest_coord = Coordinate::new_from_rank_file(dest_rank, dest_file);
            let is_capture = caps.get(4).is_some();
            let promotion_piece_type = match caps.get(7) {
                Some(s) => Some(PieceType::new_from_string(&s.as_str()[1..]).unwrap()),
                None => None,
            };

            // TODO: Handle checks and checkmate

            // Pawn captures must specify source file
            if is_capture && piece_type == PieceType::Pawn && source_file.is_none() {
                return Err("Invalid move!");
            }

            Ok((
                piece_type,
                source_rank,
                source_file,
                dest_coord,
                is_capture,
                promotion_piece_type,
            ))
        }
        None => Err("Invalid move!"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_move_strings() {
        let test_cases = [
            (
                "Nc4",
                (PieceType::Knight, None, None, Coordinate::C4, false, None),
            ),
            (
                "Nxc4",
                (PieceType::Knight, None, None, Coordinate::C4, true, None),
            ),
            (
                "Nbd7",
                (
                    PieceType::Knight,
                    None,
                    Some(2),
                    Coordinate::D7,
                    false,
                    None,
                ),
            ),
            (
                "N5d6",
                (
                    PieceType::Knight,
                    Some(5),
                    None,
                    Coordinate::D6,
                    false,
                    None,
                ),
            ),
            (
                "N5xd6",
                (PieceType::Knight, Some(5), None, Coordinate::D6, true, None),
            ),
            (
                "bxc5",
                (PieceType::Pawn, None, Some(2), Coordinate::C5, true, None),
            ),
            (
                "Bxc5",
                (PieceType::Bishop, None, None, Coordinate::C5, true, None),
            ),
            (
                "Qxc5",
                (PieceType::Queen, None, None, Coordinate::C5, true, None),
            ),
            (
                "Kxc5",
                (PieceType::King, None, None, Coordinate::C5, true, None),
            ),
            (
                "e8=Q",
                (
                    PieceType::Pawn,
                    None,
                    None,
                    Coordinate::E8,
                    false,
                    Some(PieceType::Queen),
                ),
            ),
            (
                "exf8=Q",
                (
                    PieceType::Pawn,
                    None,
                    Some(5),
                    Coordinate::F8,
                    true,
                    Some(PieceType::Queen),
                ),
            ),
        ];

        for (move_str, expected_res) in &test_cases {
            match parse_move_string(move_str) {
                Ok(res) => {
                    assert_eq!(*expected_res, res);
                }
                Err(_) => panic!("Unable to parse move"),
            }
        }
    }

    #[test]
    fn parse_invalid_move_strings() {
        let test_strings = ["Kxf0", "Lxc5", "K6f", "O-O", "O-O-O"];

        for test_string in &test_strings {
            assert!(parse_move_string(test_string).is_err());
        }
    }
}

#[cfg(test)]
mod game_tests {
    use super::*;
    use crate::board::*;

    #[test]
    fn finding_basic_legal_move() {
        let board = Board::new_empty();
        let mut game = Game::new(board);

        let piece = Piece {
            color: Color::White,
            piece_type: PieceType::Bishop,
        };
        let expected_move = Move::new(Coordinate::E2, Coordinate::F3, piece);
        let alternative_move = Move::new(Coordinate::E2, Coordinate::G4, piece);

        game.current_legal_moves = Some(vec![expected_move, alternative_move]);

        let found_move = game
            .find_legal_move(PieceType::Bishop, Coordinate::F3, None, None, false, None)
            .unwrap();
        assert_eq!(found_move, expected_move);
    }

    #[test]
    fn finding_illegal_move() {
        // Verify that if a move is not in the legal move list, we get an error
        let board = Board::new_empty();
        let mut game = Game::new(board);

        let piece = Piece {
            color: Color::White,
            piece_type: PieceType::Bishop,
        };
        let move1 = Move::new(Coordinate::E2, Coordinate::F3, piece);
        let move2 = Move::new(Coordinate::E2, Coordinate::G4, piece);

        game.current_legal_moves = Some(vec![move1, move2]);

        match game.find_legal_move(PieceType::Bishop, Coordinate::E3, None, None, false, None) {
            Ok(_) => panic!("Expected an error"),
            Err(e) => {
                assert_eq!(e.to_string(), "Illegal move");
            }
        }
    }

    #[test]
    fn finding_move_with_ambiguous_source_knights_on_different_ranks_and_files() {
        let board = Board::new_empty();
        let mut game = Game::new(board);

        let knight1 = Piece {
            color: Color::White,
            piece_type: PieceType::Knight,
        };
        let knight2 = Piece {
            color: Color::White,
            piece_type: PieceType::Knight,
        };
        let move1 = Move::new(Coordinate::B1, Coordinate::D2, knight1);
        let move2 = Move::new(Coordinate::F3, Coordinate::D2, knight2);

        game.current_legal_moves = Some(vec![move1, move2]);

        // Without specifying which knight
        match game.find_legal_move(PieceType::Knight, Coordinate::D2, None, None, false, None) {
            Ok(_) => panic!("Expected an error"),
            Err(e) => {
                assert_eq!(e.to_string(), "The move is ambiguous.");
            }
        }

        // Specifying the rank instead of the file
        match game.find_legal_move(
            PieceType::Knight,
            Coordinate::D2,
            Some(1),
            None,
            false,
            None,
        ) {
            Ok(_) => panic!("Expected an error"),
            Err(e) => {
                assert_eq!(e.to_string(), "Ambiguity in file of source square");
            }
        }

        // Specifying the file to disambiguate the move (Nbd2)
        match game.find_legal_move(
            PieceType::Knight,
            Coordinate::D2,
            None,
            Some(2),
            false,
            None,
        ) {
            Ok(m) => {
                assert_eq!(move1, m);
            }
            Err(_) => {
                panic!("Did not expect an error");
            }
        }

        // Specifying the file to disambiguate the move (Nfd2)
        match game.find_legal_move(
            PieceType::Knight,
            Coordinate::D2,
            None,
            Some(6),
            false,
            None,
        ) {
            Ok(m) => {
                assert_eq!(move2, m);
            }
            Err(_) => {
                panic!("Did not expect an error");
            }
        }
    }

    #[test]
    fn finding_move_with_ambiguous_source_rooks_on_the_same_rank() {
        let board = Board::new_empty();
        let mut game = Game::new(board);

        let rook1 = Piece {
            color: Color::White,
            piece_type: PieceType::Rook,
        };
        let rook2 = Piece {
            color: Color::White,
            piece_type: PieceType::Rook,
        };
        let move1 = Move::new(Coordinate::B2, Coordinate::C2, rook1);
        let move2 = Move::new(Coordinate::D2, Coordinate::C2, rook2);

        game.current_legal_moves = Some(vec![move1, move2]);

        // Without specifying which rook
        match game.find_legal_move(PieceType::Rook, Coordinate::C2, None, None, false, None) {
            Ok(_) => panic!("Expected an error"),
            Err(e) => {
                assert_eq!(e.to_string(), "The move is ambiguous.");
            }
        }

        // Specifying rank to disambiguate move
        match game.find_legal_move(PieceType::Rook, Coordinate::C2, Some(2), None, false, None) {
            Ok(_) => panic!("Expected an error"),
            Err(e) => {
                assert_eq!(e.to_string(), "Ambiguity in file of source square");
            }
        }

        // Specifying the file to disambiguate the move (Rbc2)
        match game.find_legal_move(PieceType::Rook, Coordinate::C2, None, Some(2), false, None) {
            Ok(m) => {
                assert_eq!(move1, m);
            }
            Err(_) => {
                panic!("Did not expect an error");
            }
        }

        // Specifying the file to disambiguate the move (Rdc2)
        match game.find_legal_move(PieceType::Rook, Coordinate::C2, None, Some(4), false, None) {
            Ok(m) => {
                assert_eq!(move2, m);
            }
            Err(_) => {
                panic!("Did not expect an error");
            }
        }
    }

    #[test]
    fn finding_move_with_ambiguous_source_rooks_on_the_same_file() {
        let board = Board::new_empty();
        let mut game = Game::new(board);

        let rook1 = Piece {
            color: Color::White,
            piece_type: PieceType::Rook,
        };
        let rook2 = Piece {
            color: Color::White,
            piece_type: PieceType::Rook,
        };
        let move1 = Move::new(Coordinate::B2, Coordinate::B4, rook1);
        let move2 = Move::new(Coordinate::B5, Coordinate::B4, rook2);

        game.current_legal_moves = Some(vec![move1, move2]);

        // Without specifying which rook
        match game.find_legal_move(PieceType::Rook, Coordinate::B4, None, None, false, None) {
            Ok(_) => panic!("Expected an error"),
            Err(e) => {
                assert_eq!(e.to_string(), "The move is ambiguous.");
            }
        }

        // Specifying file to disambiguate move
        match game.find_legal_move(PieceType::Rook, Coordinate::B4, None, Some(2), false, None) {
            Ok(_) => panic!("Expected an error"),
            Err(e) => {
                assert_eq!(e.to_string(), "Ambiguity in rank of source square");
            }
        }

        // Specifying the rank to disambiguate the move (R2b4)
        match game.find_legal_move(PieceType::Rook, Coordinate::B4, Some(2), None, false, None) {
            Ok(m) => {
                assert_eq!(move1, m);
            }
            Err(_) => {
                panic!("Did not expect an error");
            }
        }

        // Specifying the rank to disambiguate the move (R5b4)
        match game.find_legal_move(PieceType::Rook, Coordinate::B4, Some(5), None, false, None) {
            Ok(m) => {
                assert_eq!(move2, m);
            }
            Err(_) => {
                panic!("Did not expect an error");
            }
        }
    }

    #[test]
    fn finding_promotion_move() {
        let board = Board::new_empty();
        let mut game = Game::new(board);

        let piece = Piece {
            color: Color::White,
            piece_type: PieceType::Pawn,
        };
        let mut expected_move = Move::new(Coordinate::E7, Coordinate::E8, piece);
        expected_move.is_promotion = true;
        expected_move.promotes_to = Some(PieceType::Rook);
        let mut alternative_move = expected_move.clone();
        alternative_move.promotes_to = Some(PieceType::Queen);

        game.current_legal_moves = Some(vec![expected_move, alternative_move]);

        let found_move = game
            .find_legal_move(
                PieceType::Pawn,
                Coordinate::E8,
                None,
                None,
                false,
                Some(PieceType::Rook),
            )
            .unwrap();
        assert_eq!(found_move, expected_move);
    }

    #[test]
    fn finding_promotion_move_with_captures() {
        let board = Board::new_empty();
        let mut game = Game::new(board);

        let piece = Piece {
            color: Color::White,
            piece_type: PieceType::Pawn,
        };
        let mut expected_move = Move::new(Coordinate::F7, Coordinate::E8, piece);
        expected_move.is_promotion = true;
        expected_move.promotes_to = Some(PieceType::Rook);
        let mut alternative_move = expected_move.clone();
        alternative_move.promotes_to = Some(PieceType::Queen);

        game.current_legal_moves = Some(vec![expected_move, alternative_move]);

        let found_move = game
            .find_legal_move(
                PieceType::Pawn,
                Coordinate::E8,
                None,
                Some(6),
                false,
                Some(PieceType::Rook),
            )
            .unwrap();
        assert_eq!(found_move, expected_move);
    }

    #[test]
    fn finding_promotion_move_with_two_possible_captures() {
        let board = Board::new_empty();
        let mut game = Game::new(board);

        let piece = Piece {
            color: Color::White,
            piece_type: PieceType::Pawn,
        };
        let mut expected_move = Move::new(Coordinate::F7, Coordinate::E8, piece);
        expected_move.is_promotion = true;
        expected_move.promotes_to = Some(PieceType::Rook);
        let mut alternative_move_1 = expected_move.clone();
        alternative_move_1.promotes_to = Some(PieceType::Queen);
        let mut alternative_move_2 = expected_move.clone();
        alternative_move_2.src = Coordinate::D7;
        let mut alternative_move_3 = alternative_move_1.clone();
        alternative_move_3.src = Coordinate::D7;

        game.current_legal_moves = Some(vec![
            expected_move,
            alternative_move_1,
            alternative_move_2,
            alternative_move_3,
        ]);

        let found_move = game
            .find_legal_move(
                PieceType::Pawn,
                Coordinate::E8,
                None,
                Some(6),
                false,
                Some(PieceType::Rook),
            )
            .unwrap();
        assert_eq!(found_move, expected_move);
    }

    #[test]
    fn threefold_repetition_bongcloud() {
        let board = Board::new_starting_pos();
        let mut game = Game::new(board);
        let white_pawn_advance = Move::new(
            Coordinate::E2,
            Coordinate::E4,
            Piece::new(Color::White, PieceType::Pawn),
        );
        let black_pawn_advance = Move::new(
            Coordinate::E7,
            Coordinate::E5,
            Piece::new(Color::Black, PieceType::Pawn),
        );
        let white_king_advance = Move::new(
            Coordinate::E1,
            Coordinate::E2,
            Piece::new(Color::White, PieceType::King),
        );
        let black_king_advance = Move::new(
            Coordinate::E8,
            Coordinate::E7,
            Piece::new(Color::Black, PieceType::King),
        );
        let white_king_retreat = Move::new(
            Coordinate::E2,
            Coordinate::E1,
            Piece::new(Color::White, PieceType::King),
        );
        let black_king_retreat = Move::new(
            Coordinate::E7,
            Coordinate::E8,
            Piece::new(Color::Black, PieceType::King),
        );

        game.apply_move(&white_pawn_advance);
        game.apply_move(&black_pawn_advance);
        assert!(!game.is_threefold_repetition());

        game.apply_move(&white_king_advance);
        game.apply_move(&black_king_advance);
        assert!(!game.is_threefold_repetition());

        game.apply_move(&white_king_retreat);
        game.apply_move(&black_king_retreat);
        assert!(!game.is_threefold_repetition());

        game.apply_move(&white_king_advance);
        game.apply_move(&black_king_advance);
        assert!(!game.is_threefold_repetition());

        game.apply_move(&white_king_retreat);
        game.apply_move(&black_king_retreat);
        assert!(!game.is_threefold_repetition());

        game.apply_move(&white_king_advance);
        game.apply_move(&black_king_advance);

        assert!(game.is_threefold_repetition());
    }
}
