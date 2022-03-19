use super::{ArcMutexUCIState, GoArgs};
use crate::board::{Board, Coordinate};
use crate::engine::Searcher;
use crate::game::Game;
use crate::parser::parse_fen;

use mockall_double::double;
use std::io::Write;

#[allow(dead_code)]
mod mockable {
    use super::*;
    #[cfg(test)]
    use mockall::automock;
    use std::thread;

    pub struct UCIHandler {}

    #[cfg_attr(test, automock)]
    impl UCIHandler {
        pub fn new() -> Self {
            UCIHandler {}
        }

        pub fn handle_uci<W: Write + Send + 'static>(
            &mut self,
            state: ArcMutexUCIState,
            output: W,
        ) {
            thread::spawn(move || {
                uci(state, output);
            });
        }

        pub fn handle_isready<W: Write + Send + 'static>(
            &mut self,
            _state: ArcMutexUCIState,
            output: W,
        ) {
            thread::spawn(move || {
                isready(output);
            });
        }

        pub fn handle_ucinewgame<W: Write + Send + 'static>(
            &mut self,
            _state: ArcMutexUCIState,
            _output: W,
        ) {
        }

        pub fn handle_position_fen<W: Write + Send + 'static>(
            &mut self,
            state: ArcMutexUCIState,
            output: W,
            fen: String,
            moves: Vec<String>,
        ) {
            // This cannot be done asynchronously since
            // users expect this to be instant
            position_with_fen(state, output, fen, moves);
        }

        pub fn handle_position_startpos<W: Write + Send + 'static>(
            &mut self,
            state: ArcMutexUCIState,
            output: W,
            moves: Vec<String>,
        ) {
            // This cannot be done asynchronously since
            // users expect this to be instant
            position_with_startpos(state, output, moves);
        }

        pub fn handle_go<W: Write + Send + 'static>(
            &mut self,
            state: ArcMutexUCIState,
            output: W,
            args_str: String,
        ) {
            thread::spawn(move || {
                go(state, output, args_str);
            });
        }

        pub fn handle_stop<W: Write + Send + 'static>(
            &mut self,
            state: ArcMutexUCIState,
            output: W,
        ) {
            thread::spawn(move || {
                stop(state, output);
            });
        }

        pub fn handle_ponderhit<W: Write + Send + 'static>(
            &mut self,
            state: ArcMutexUCIState,
            output: W,
        ) {
            thread::spawn(move || {
                ponderhit(state, output);
            });
        }
    }
}

#[double]
pub use mockable::UCIHandler;

fn uci<W: Write + Send + 'static>(state: ArcMutexUCIState, mut output: W) {
    let mut state = state.lock().unwrap();
    state.is_initialized = true;

    writeln!(output, "id name Requin v1.1.0").unwrap();
    writeln!(output, "id author James Tan").unwrap();
    writeln!(output, "uciok").unwrap();
    output.flush().unwrap();
}

fn isready<W: Write + Send + 'static>(mut output: W) {
    writeln!(output, "readyok").unwrap();
    output.flush().unwrap();
}

fn position_with_fen<W: Write + Send + 'static>(
    state: ArcMutexUCIState,
    _output: W,
    fen: String,
    moves: Vec<String>,
) {
    let board = match parse_fen(fen) {
        Ok(board) => board,
        Err(_) => return,
    };
    apply_moves_and_set_state::<W>(state, board, moves);
}

fn position_with_startpos<W: Write + Send + 'static>(
    state: ArcMutexUCIState,
    _output: W,
    moves: Vec<String>,
) {
    let board = Board::new_starting_pos();
    apply_moves_and_set_state::<W>(state, board, moves);
}

fn go<W: Write + Send + 'static>(state: ArcMutexUCIState, mut output: W, args_str: String) {
    let mut state = state.lock().unwrap();
    state.go_args = Some(GoArgs::new_from_args_str(args_str));

    let pos = match state.position {
        Some(pos) => pos,
        None => {
            let board = Board::new_starting_pos();
            state.position = Some(board);
            board
        }
    };

    // TODO: Read from go args when the engine is
    // capable of early stopping
    let mut searcher = Searcher::new(Game::new(pos), 5, 16);

    match searcher.get_best_move() {
        Ok(best_move) => {
            writeln!(
                output,
                "bestmove {}",
                best_move.to_long_algebraic_notation()
            )
            .unwrap();
            output.flush().unwrap();
        }
        Err(e) => panic!("Unexpected error during move search: {}", e),
    }
}

fn stop<W: Write + Send + 'static>(_state: ArcMutexUCIState, _output: W) {
    // TODO: Implement early stopping
}

fn ponderhit<W: Write + Send + 'static>(_state: ArcMutexUCIState, _output: W) {
    // TODO: Implement early stopping
}

fn apply_moves_and_set_state<W: Write + Send + 'static>(
    state: ArcMutexUCIState,
    mut board: Board,
    moves: Vec<String>,
) {
    let mut state = state.lock().unwrap();

    for m in moves {
        let (src, dest) = Coordinate::new_from_long_algebraic_notation(&m);
        // We try to apply as many moves as possible
        match board.apply_move_with_src_dest(src, dest) {
            Ok(_) => {}
            Err(_) => {
                break;
            }
        }
    }
    state.position = Some(board);
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::uci::new_arc_mutex_uci_state;
    use crate::uci::Output;
    use std::sync::Arc;

    #[test]
    fn handle_uci() {
        let output_buffer: Output<Vec<u8>> = Output::new(vec![]);
        let state = new_arc_mutex_uci_state();
        uci(Arc::clone(&state), output_buffer.clone());

        assert!(state.lock().unwrap().is_initialized());

        assert_eq!(
            std::str::from_utf8(&output_buffer.get_inner().lock().unwrap()).unwrap(),
            "id name Requin v1.1.0\nid author James Tan\nuciok\n"
        );
    }

    #[test]
    fn handle_isready() {
        let output_buffer: Output<Vec<u8>> = Output::new(vec![]);
        isready(output_buffer.clone());

        assert_eq!(
            std::str::from_utf8(&output_buffer.get_inner().lock().unwrap()).unwrap(),
            "readyok\n"
        );
    }

    #[test]
    fn handle_position_startpos_with_no_moves() {
        let output_buffer: Output<Vec<u8>> = Output::new(vec![]);
        let state = new_arc_mutex_uci_state();
        position_with_startpos(state.clone(), output_buffer.clone(), vec![]);

        assert_eq!(
            state.lock().unwrap().position.unwrap(),
            Board::new_starting_pos()
        );
    }

    #[test]
    fn handle_position_startpos_with_moves() {
        let output_buffer: Output<Vec<u8>> = Output::new(vec![]);
        let state = new_arc_mutex_uci_state();
        position_with_startpos(
            state.clone(),
            output_buffer.clone(),
            vec![String::from("e2e4"), String::from("e7e5")],
        );

        let board = state.lock().unwrap().position.unwrap();

        assert!(board.get_from_coordinate(Coordinate::E2).is_none());
        assert!(board.get_from_coordinate(Coordinate::E7).is_none());
        assert!(board.get_from_coordinate(Coordinate::E4).is_some());
        assert!(board.get_from_coordinate(Coordinate::E5).is_some());
    }

    #[test]
    fn handle_position_fen_with_no_moves() {
        let output_buffer: Output<Vec<u8>> = Output::new(vec![]);
        let state = new_arc_mutex_uci_state();
        position_with_fen(
            state.clone(),
            output_buffer.clone(),
            String::from("8/8/4Rp2/5P2/1PP1pkP1/7P/1P1r4/7K b - - 0 40"),
            vec![],
        );

        let expected_board =
            crate::parser::parse_fen(String::from("8/8/4Rp2/5P2/1PP1pkP1/7P/1P1r4/7K b - - 0 40"))
                .unwrap();

        assert_eq!(state.lock().unwrap().position.unwrap(), expected_board);
    }

    #[test]
    fn handle_position_fen_with_moves() {
        let output_buffer: Output<Vec<u8>> = Output::new(vec![]);
        let state = new_arc_mutex_uci_state();
        position_with_fen(
            state.clone(),
            output_buffer.clone(),
            String::from("8/8/4Rp2/5P2/1PP1pkP1/7P/1P1r4/7K b - - 0 40"),
            vec![String::from("f4g3"), String::from("e6e4")],
        );

        let expected_board =
            crate::parser::parse_fen(String::from("8/8/5p2/5P2/1PP1R1P1/6kP/1P1r4/7K b - - 0 41"))
                .unwrap();
        let board = state.lock().unwrap().position.unwrap();

        assert_eq!(expected_board, board);
    }
}
