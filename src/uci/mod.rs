mod client;
mod handler;
mod output;

pub use client::Client;
pub use output::Output;

use crate::board::Board;
use std::sync::{Arc, Mutex};

pub struct UCIState {
    is_initialized: bool,
    position: Option<Board>,
}

impl UCIState {
    pub fn new() -> Self {
        UCIState {
            is_initialized: false,
            position: None,
        }
    }

    pub fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

type ArcMutexUCIState = Arc<Mutex<UCIState>>;

pub fn new_arc_mutex_uci_state() -> ArcMutexUCIState {
    Arc::new(Mutex::new(UCIState::new()))
}
