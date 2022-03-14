use super::handler::{Handler, UCIHandler};
use super::{ArcMutexUCIState, Output, UCIState};
use std::error::Error;
use std::io;
use std::sync::{Arc, Mutex};

use lazy_static::lazy_static;
use regex::Regex;

pub struct Client {
    state: ArcMutexUCIState,
    handler: UCIHandler,
}

impl Client {
    pub fn new() -> Self {
        Client {
            state: Arc::new(Mutex::new(UCIState::new())),
            handler: UCIHandler::new(),
        }
    }

    pub fn run(&mut self) {
        loop {
            self.get_and_run_command();
        }
    }

    fn get_and_run_command(&mut self) {
        match read_stdin() {
            Ok(cmd) => {
                self.handle_command(cmd);
            }
            Err(e) => {
                panic!("Unable to get command from stdin: {}", e);
            }
        }
    }

    // Implement UCI protocol base on
    // http://wbec-ridderkerk.nl/html/UCIProtocol.html
    fn handle_command(&mut self, cmd: String) {
        lazy_static! {
            static ref UCI: Regex = Regex::new(r"uci").unwrap();
            static ref ISREADY: Regex = Regex::new(r"isready").unwrap();
        }

        if let Some(_) = UCI.captures(&cmd) {
            self.handler
                .handle_uci(Arc::clone(&self.state), Output::new(std::io::stdout()));
        } else if let Some(_) = ISREADY.captures(&cmd) {
            self.handler
                .handle_isready(Arc::clone(&self.state), std::io::stdout());
        } else {
            // UCI protocol indicate that we should ignore
            // unknown commands
        }
    }
}

fn read_stdin() -> Result<String, Box<dyn Error>> {
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;
    Ok(buffer)
}
