use super::ArcMutexUCIState;
use crate::board::Board;
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
            thread::spawn(move || {
                position_with_fen(state, output, fen);
            });
        }

        pub fn handle_position_startpos<W: Write + Send + 'static>(
            &mut self,
            state: ArcMutexUCIState,
            output: W,
            moves: Vec<String>,
        ) {
            thread::spawn(move || {
                position_with_startpos(state, output);
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

fn position_with_fen<W: Write + Send + 'static>(state: ArcMutexUCIState, _output: W, fen: String) {
    let board = match parse_fen(fen) {
        Ok(board) => board,
        Err(_) => return,
    };

    let mut state = state.lock().unwrap();
    state.position = Some(board);
}

fn position_with_startpos<W: Write + Send + 'static>(state: ArcMutexUCIState, _output: W) {
    let board = Board::new_starting_pos();
    let mut state = state.lock().unwrap();
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
}
