use super::handler::UCIHandler;
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

    pub fn new_with_handler(handler: UCIHandler) -> Self {
        Client {
            state: Arc::new(Mutex::new(UCIState::new())),
            handler,
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
            static ref UCI: Regex = Regex::new(r"^uci$").unwrap();
            static ref ISREADY: Regex = Regex::new(r"^isready$").unwrap();
            static ref UCINEWGAME: Regex = Regex::new(r"^ucinewgame$").unwrap();
        }

        if let Some(_) = UCI.captures(&cmd) {
            self.handler
                .handle_uci(Arc::clone(&self.state), Output::new(std::io::stdout()));
        } else if let Some(_) = ISREADY.captures(&cmd) {
            self.handler
                .handle_isready(Arc::clone(&self.state), Output::new(std::io::stdout()));
        } else if let Some(_) = UCINEWGAME.captures(&cmd) {
            self.handler
                .handle_ucinewgame(Arc::clone(&self.state), Output::new(std::io::stdout()));
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_uci() {
        let mut mock_handler = UCIHandler::default();
        mock_handler
            .expect_handle_uci::<Output<std::io::Stdout>>()
            .times(1)
            .returning(|_, _| ());
        let mut client = Client::new_with_handler(mock_handler);
        client.handle_command(String::from("uci"));
    }

    #[test]
    fn test_handle_isready() {
        let mut mock_handler = UCIHandler::default();
        mock_handler
            .expect_handle_isready::<Output<std::io::Stdout>>()
            .times(1)
            .returning(|_, _| ());
        let mut client = Client::new_with_handler(mock_handler);
        client.handle_command(String::from("isready"));
    }

    #[test]
    fn test_handle_ucinewgame() {
        let mut mock_handler = UCIHandler::default();
        mock_handler
            .expect_handle_ucinewgame::<Output<std::io::Stdout>>()
            .times(1)
            .returning(|_, _| ());
        let mut client = Client::new_with_handler(mock_handler);
        client.handle_command(String::from("ucinewgame"));
    }
}
