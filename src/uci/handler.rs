use super::ArcMutexUCIState;
use std::io::Write;

pub trait Handler {
    fn handle_uci(&mut self, state: ArcMutexUCIState, output: impl Write);
    fn handle_isready(&mut self, state: ArcMutexUCIState, output: impl Write);
}

pub struct UCIHandler {}

impl UCIHandler {
    pub fn new() -> Self {
        UCIHandler {}
    }
}

impl Handler for UCIHandler {
    fn handle_uci(&mut self, state: ArcMutexUCIState, mut output: impl Write) {
        let mut state = state.lock().unwrap();
        state.is_initialized = true;

        writeln!(output, "id name Requin v1.1.0").unwrap();
        writeln!(output, "id author James Tan").unwrap();
        writeln!(output, "uciok").unwrap();
        output.flush().unwrap();
    }

    fn handle_isready(&mut self, _state: ArcMutexUCIState, mut output: impl Write) {
        writeln!(output, "readyok").unwrap();
        output.flush().unwrap();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::uci::new_arc_mutex_uci_state;
    use std::sync::Arc;

    #[test]
    fn handle_uciok() {
        let mut output_buffer: Vec<u8> = vec![];
        let mut handler = UCIHandler::new();
        let state = new_arc_mutex_uci_state();
        handler.handle_uci(Arc::clone(&state), &mut output_buffer);

        assert!(state.lock().unwrap().is_initialized());
        assert_eq!(
            std::str::from_utf8(&output_buffer).unwrap(),
            "id name Requin v1.1.0\nid author James Tan\nuciok\n"
        );
    }

    #[test]
    fn handle_isready() {
        let mut output_buffer: Vec<u8> = vec![];
        let mut handler = UCIHandler::new();
        let state = new_arc_mutex_uci_state();
        handler.handle_isready(Arc::clone(&state), &mut output_buffer);

        assert_eq!(std::str::from_utf8(&output_buffer).unwrap(), "readyok\n");
    }
}
