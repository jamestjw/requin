use super::time::max_search_time;
use super::{ArcMutexUCIState, GoArgs, UCIOption, UCIOptionType};
use crate::board::{Board, Color, Coordinate};
use crate::engine::Searcher;
use crate::game::Game;
use crate::parser::parse_fen;

use lazy_static::lazy_static;
use mockall_double::double;
use std::io::Write;

lazy_static! {
    static ref UCI_OPTIONS: [UCIOption; 1] = {
        [UCIOption::new(
            "NumThreads".into(),
            UCIOptionType::Spin,
            16,
            1,
            32,
        )]
    };
}

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

        pub fn handle_setoption<W: Write + Send + 'static>(
            &mut self,
            state: ArcMutexUCIState,
            output: W,
            arg_name: String,
            args_val: String,
        ) {
            set_option(state, output, arg_name, args_val);
        }
    }
}

#[double]
pub use mockable::UCIHandler;

fn uci<W: Write + Send + 'static>(state: ArcMutexUCIState, mut output: W) {
    let mut state = state.lock().unwrap();
    state.is_initialized = true;

    writeln!(output, "id name Requin v1.3.0").unwrap();
    writeln!(output, "id author James Tan").unwrap();

    for uci_option in UCI_OPTIONS.iter() {
        writeln!(output, "{}", format_uci_option(uci_option)).unwrap();
    }

    writeln!(output, "uciok").unwrap();
    output.flush().unwrap();
}

fn format_uci_option(uci_option: &UCIOption) -> String {
    // TODO: Handle other types
    let type_string = match uci_option.option_type {
        UCIOptionType::Spin => "spin",
        // _ => panic!("Unexpected UCIOptionType"),
    };
    format!(
        "option name {} type {} default {} min {} max {}",
        uci_option.name, type_string, uci_option.default, uci_option.min, uci_option.max
    )
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
    apply_moves_and_set_state::<W>(state, Game::new(board), moves);
}

fn position_with_startpos<W: Write + Send + 'static>(
    state: ArcMutexUCIState,
    _output: W,
    moves: Vec<String>,
) {
    apply_moves_and_set_state::<W>(state, Game::new(Board::new_starting_pos()), moves);
}

fn go<W: Write + Send + 'static>(state: ArcMutexUCIState, mut output: W, args_str: String) {
    let mut state = state.lock().unwrap();
    let go_args = GoArgs::new_from_args_str(args_str);
    let depth = go_args.depth;

    let game = match &state.game {
        Some(g) => g.clone(),
        None => {
            let board = Board::new_starting_pos();
            let game = Game::new(board);
            state.game = Some(game.clone());
            game
        }
    };

    let player_color = game.current_board().get_player_color();
    let (player_time, player_increment) = match player_color {
        Color::White => (go_args.wtime, go_args.winc),
        Color::Black => (go_args.btime, go_args.binc),
    };

    state.go_args = Some(go_args);

    let mut searcher = Searcher::new(game, depth, state.num_threads);
    let search_time = player_time.map(|t| max_search_time(t, player_increment.unwrap_or(0)));

    match searcher.get_best_move(search_time) {
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
    mut game: Game,
    moves: Vec<String>,
) {
    let mut state = state.lock().unwrap();

    for m in moves {
        let (src, dest, promotes_to) = Coordinate::new_from_long_algebraic_notation(&m);
        // We try to apply as many moves as possible
        match game.apply_move_with_src_dest(src, dest, promotes_to) {
            Ok(_) => {}
            Err(e) => {
                println!("Illegal move {}. Error: {}", m, e);
                break;
            }
        }
    }
    state.game = Some(game);
}

fn set_option<W: Write + Send + 'static>(
    state: ArcMutexUCIState,
    mut output: W,
    arg_name: String,
    args_val: String,
) {
    // Search for the option
    for uci_option in UCI_OPTIONS.iter() {
        if arg_name == uci_option.name {
            match args_val.parse::<u32>() {
                Ok(val) => {
                    if val > uci_option.max || val < uci_option.min {
                        writeln!(output, "Option not in range for {}", arg_name).unwrap();
                        output.flush().unwrap();
                        return;
                    }
                    let mut state = state.lock().unwrap();

                    // TODO: Figure out a better of doing this, this looks like a code smell
                    // Consider using a HashMap when more options come into play
                    if arg_name == "NumThreads" {
                        state.num_threads = val as usize;
                    } else {
                        writeln!(output, "Unexpected option {}", arg_name).unwrap();
                        output.flush().unwrap();
                    }
                }
                Err(_) => {
                    writeln!(output, "Expected an integer option for {}", arg_name).unwrap();
                    output.flush().unwrap();
                }
            }
            return;
        }
    }
    writeln!(output, "{} is an invalid option", arg_name).unwrap();
    output.flush().unwrap();
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
            vec![
                "id name Requin v1.3.0\n",
                "id author James Tan\n",
                "option name NumThreads type spin default 16 min 1 max 32\n",
                "uciok\n"
            ]
            .join("")
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
            *state.lock().unwrap().game.as_ref().unwrap().current_board(),
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

        let board = state
            .lock()
            .unwrap()
            .game
            .as_ref()
            .unwrap()
            .current_board()
            .clone();

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

        assert_eq!(
            state
                .lock()
                .unwrap()
                .game
                .as_ref()
                .unwrap()
                .current_board()
                .clone(),
            expected_board
        );
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
        let board = *state.lock().unwrap().game.as_ref().unwrap().current_board();

        assert_eq!(expected_board, board);
    }

    #[test]
    fn handle_set_option_invalid_option_name() {
        let output_buffer: Output<Vec<u8>> = Output::new(vec![]);
        let state = new_arc_mutex_uci_state();
        set_option(
            state,
            output_buffer.clone(),
            "InvalidOptionName".into(),
            "Some random arg".into(),
        );

        assert_eq!(
            std::str::from_utf8(&output_buffer.get_inner().lock().unwrap()).unwrap(),
            "InvalidOptionName is an invalid option\n"
        );
    }

    #[test]
    fn handle_set_option_num_threads_valid() {
        let output_buffer: Output<Vec<u8>> = Output::new(vec![]);
        let state = new_arc_mutex_uci_state();
        set_option(
            state.clone(),
            output_buffer.clone(),
            "NumThreads".into(),
            "12".into(),
        );

        assert_eq!(state.lock().unwrap().num_threads, 12);
    }

    #[test]
    fn handle_set_option_num_threads_out_of_range() {
        let output_buffer: Output<Vec<u8>> = Output::new(vec![]);
        let state = new_arc_mutex_uci_state();
        set_option(
            state.clone(),
            output_buffer.clone(),
            "NumThreads".into(),
            "10000".into(),
        );

        assert_eq!(
            std::str::from_utf8(&output_buffer.get_inner().lock().unwrap()).unwrap(),
            "Option not in range for NumThreads\n"
        );
    }

    #[test]
    fn handle_set_option_num_threads_not_number() {
        let output_buffer: Output<Vec<u8>> = Output::new(vec![]);
        let state = new_arc_mutex_uci_state();
        set_option(
            state.clone(),
            output_buffer.clone(),
            "NumThreads".into(),
            "abcd".into(),
        );

        assert_eq!(
            std::str::from_utf8(&output_buffer.get_inner().lock().unwrap()).unwrap(),
            "Expected an integer option for NumThreads\n"
        );
    }
}
