mod client;
mod go_args;
mod handler;
mod output;
mod time;

pub use client::Client;
use go_args::GoArgs;
pub use output::Output;

use crate::board::Board;
use std::sync::{Arc, Mutex};

pub struct UCIState {
    is_initialized: bool,
    position: Option<Board>,
    go_args: Option<GoArgs>,
    num_threads: usize,
}

impl UCIState {
    pub fn new() -> Self {
        UCIState {
            is_initialized: false,
            position: None,
            go_args: None,
            num_threads: 16,
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

// TODO: Handle string types
struct UCIOption {
    name: String,
    option_type: UCIOptionType,
    default: u32,
    min: u32,
    max: u32,
}

impl UCIOption {
    pub fn new(
        name: String,
        option_type: UCIOptionType,
        default: u32,
        min: u32,
        max: u32,
    ) -> UCIOption {
        if option_type != UCIOptionType::Spin {
            panic!("Unsupported UCIOptionType");
        }
        UCIOption {
            name,
            option_type,
            default,
            min,
            max,
        }
    }
}

#[derive(PartialEq)]
enum UCIOptionType {
    // TODO: Implement the rest when they become necessary
    Spin,
    // Check,
    // Combo,
    // Button,
    // String,
}
