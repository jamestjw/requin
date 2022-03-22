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
                self.handle_command(cmd.trim_end());
            }
            Err(e) => {
                panic!("Unable to get command from stdin: {}", e);
            }
        }
    }

    // Implement UCI protocol base on
    // http://wbec-ridderkerk.nl/html/UCIProtocol.html
    fn handle_command(&mut self, cmd: &str) {
        lazy_static! {
            static ref UCI: Regex = Regex::new(r"^uci$").unwrap();
            static ref ISREADY: Regex = Regex::new(r"^isready").unwrap();
            static ref UCINEWGAME: Regex = Regex::new(r"^ucinewgame").unwrap();
            static ref POSITION_WITH_FEN: Regex = Regex::new(
                r"^position fen (([prnbqkPRNBQK12345678]{1,8}(?:/[prnbqkPRNBQK12345678]{1,8}){7})\s+(w|b)\s+([KQkq]{1,4}|-)\s+(-|[a-h][1-8])\s(\d+\s\d+))(\s+moves ([a-h][1-8][a-h][1-8][nbrq]?(\s[a-h][1-8][a-h][1-8][nbrq]?)*))?"
            ).unwrap();
            static ref POSITION_WITH_STARTPOS: Regex = Regex::new(r"position startpos(\s+moves ([a-h][1-8][a-h][1-8][nbrq]?(\s[a-h][1-8][a-h][1-8][nbrq]?)*))?").unwrap();
            static ref GO: Regex = Regex::new(r"^go((\s+(ponder|infinite|searchmoves(\s+[a-h][1-8][a-h][1-8])+|(wtime|btime|winc|binc|depth|movestogo|nodes|mate|movetime)\s+(\d+)))*)?").unwrap();
            static ref STOP: Regex = Regex::new(r"^stop").unwrap();
            static ref PONDERHIT: Regex = Regex::new(r"^ponderhit").unwrap();
            static ref QUIT: Regex = Regex::new(r"^quit").unwrap();
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
        } else if let Some(m) = POSITION_WITH_FEN.captures(&cmd) {
            let moves = match m.get(8) {
                Some(move_match) => move_match
                    .as_str()
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>(),
                None => vec![],
            };
            self.handler.handle_position_fen(
                Arc::clone(&self.state),
                Output::new(std::io::stdout()),
                m[1].to_string(),
                moves,
            );
        } else if let Some(m) = POSITION_WITH_STARTPOS.captures(&cmd) {
            let moves = match m.get(2) {
                Some(move_match) => move_match
                    .as_str()
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>(),
                None => vec![],
            };
            self.handler.handle_position_startpos(
                Arc::clone(&self.state),
                Output::new(std::io::stdout()),
                moves,
            );
        } else if let Some(m) = GO.captures(&cmd) {
            let args = m.get(1).map_or("", |v| v.as_str()).to_string();
            self.handler.handle_go(
                Arc::clone(&self.state),
                Output::new(std::io::stdout()),
                args,
            );
        } else if let Some(_) = STOP.captures(&cmd) {
            self.handler
                .handle_stop(Arc::clone(&self.state), Output::new(std::io::stdout()));
        } else if let Some(_) = PONDERHIT.captures(&cmd) {
            self.handler
                .handle_ponderhit(Arc::clone(&self.state), Output::new(std::io::stdout()));
        } else if let Some(_) = QUIT.captures(&cmd) {
            std::process::exit(exitcode::OK);
        } else {
            // UCI protocol indicate that we should ignore
            // unknown commands
            println!("Unknown command: {}", cmd);
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
    use mockall::predicate;

    #[test]
    fn test_uci() {
        let mut mock_handler = UCIHandler::default();
        mock_handler
            .expect_handle_uci::<Output<std::io::Stdout>>()
            .times(1)
            .returning(|_, _| ());
        let mut client = Client::new_with_handler(mock_handler);
        client.handle_command("uci");
    }

    #[test]
    fn test_handle_isready() {
        let mut mock_handler = UCIHandler::default();
        mock_handler
            .expect_handle_isready::<Output<std::io::Stdout>>()
            .times(1)
            .returning(|_, _| ());
        let mut client = Client::new_with_handler(mock_handler);
        client.handle_command("isready");
    }

    #[test]
    fn test_handle_ucinewgame() {
        let mut mock_handler = UCIHandler::default();
        mock_handler
            .expect_handle_ucinewgame::<Output<std::io::Stdout>>()
            .times(1)
            .returning(|_, _| ());
        let mut client = Client::new_with_handler(mock_handler);
        client.handle_command("ucinewgame");
    }

    #[test]
    fn test_handle_position_with_full_fen() {
        let mut mock_handler = UCIHandler::default();
        mock_handler
            .expect_handle_position_fen::<Output<std::io::Stdout>>()
            .with(
                predicate::always(),
                predicate::always(),
                predicate::eq(String::from("8/8/4Rp2/5P2/1PP1pkP1/7P/1P1r4/7K b - - 0 40")),
                predicate::eq(vec![]),
            )
            .times(1)
            .returning(|_, _, _, _| ());

        let mut client = Client::new_with_handler(mock_handler);
        client.handle_command("position fen 8/8/4Rp2/5P2/1PP1pkP1/7P/1P1r4/7K b - - 0 40");
    }

    #[test]
    fn test_handle_position_with_full_fen_and_moves() {
        let mut mock_handler = UCIHandler::default();
        mock_handler
            .expect_handle_position_fen::<Output<std::io::Stdout>>()
            .with(
                predicate::always(),
                predicate::always(),
                predicate::eq(String::from(
                    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
                )),
                predicate::eq(vec![String::from("e2e4"), String::from("e7e6")]),
            )
            .times(1)
            .returning(|_, _, _, _| ());

        let mut client = Client::new_with_handler(mock_handler);
        client.handle_command(
            "position fen rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 moves e2e4 e7e6",
        );
    }

    #[test]
    fn test_handle_position_with_full_fen_and_moves_with_promotion() {
        let mut mock_handler = UCIHandler::default();
        mock_handler
            .expect_handle_position_fen::<Output<std::io::Stdout>>()
            .with(
                predicate::always(),
                predicate::always(),
                predicate::eq(String::from(
                    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
                )),
                predicate::eq(vec![
                    String::from("e2e4"),
                    String::from("e7e6"),
                    String::from("g7g8q"),
                    String::from("c2c1r"),
                ]),
            )
            .times(1)
            .returning(|_, _, _, _| ());

        let mut client = Client::new_with_handler(mock_handler);
        client.handle_command(
            "position fen rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 moves e2e4 e7e6 g7g8q c2c1r",
        );
    }

    #[test]
    fn test_handle_position_with_full_fen_and_empty_moves() {
        let mut mock_handler = UCIHandler::default();
        mock_handler
            .expect_handle_position_fen::<Output<std::io::Stdout>>()
            .with(
                predicate::always(),
                predicate::always(),
                predicate::eq(String::from(
                    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
                )),
                predicate::eq(vec![]),
            )
            .times(1)
            .returning(|_, _, _, _| ());

        let mut client = Client::new_with_handler(mock_handler);
        client.handle_command(
            "position fen rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 moves",
        );
    }

    #[test]
    fn test_handle_position_with_startpos_and_moves() {
        let mut mock_handler = UCIHandler::default();
        mock_handler
            .expect_handle_position_startpos::<Output<std::io::Stdout>>()
            .with(
                predicate::always(),
                predicate::always(),
                predicate::eq(vec![String::from("e2e4"), String::from("e7e6")]),
            )
            .times(1)
            .returning(|_, _, _| ());

        let mut client = Client::new_with_handler(mock_handler);
        client.handle_command("position startpos moves e2e4 e7e6");
    }

    #[test]
    fn test_handle_position_with_startpos_and_moves_with_promotion() {
        let mut mock_handler = UCIHandler::default();
        mock_handler
            .expect_handle_position_startpos::<Output<std::io::Stdout>>()
            .with(
                predicate::always(),
                predicate::always(),
                predicate::eq(vec![
                    String::from("e2e4"),
                    String::from("e7e6"),
                    String::from("g7g8q"),
                    String::from("g7f8r"),
                ]),
            )
            .times(1)
            .returning(|_, _, _| ());

        let mut client = Client::new_with_handler(mock_handler);
        client.handle_command("position startpos moves e2e4 e7e6 g7g8q g7f8r");
    }

    #[test]
    fn test_handle_position_with_startpos_and_empty_moves() {
        let mut mock_handler = UCIHandler::default();
        mock_handler
            .expect_handle_position_startpos::<Output<std::io::Stdout>>()
            .with(
                predicate::always(),
                predicate::always(),
                predicate::eq(vec![]),
            )
            .times(1)
            .returning(|_, _, _| ());

        let mut client = Client::new_with_handler(mock_handler);
        client.handle_command("position startpos moves");
    }

    #[test]
    fn test_handle_go_no_args() {
        let mut mock_handler = UCIHandler::default();
        mock_handler
            .expect_handle_go::<Output<std::io::Stdout>>()
            .with(
                predicate::always(),
                predicate::always(),
                predicate::eq(String::from("")),
            )
            .times(1)
            .returning(|_, _, _| ());

        let mut client = Client::new_with_handler(mock_handler);
        client.handle_command("go");
    }

    #[test]
    fn test_handle_go_infinite() {
        let mut mock_handler = UCIHandler::default();
        mock_handler
            .expect_handle_go::<Output<std::io::Stdout>>()
            .with(
                predicate::always(),
                predicate::always(),
                predicate::eq(String::from(" infinite")),
            )
            .times(1)
            .returning(|_, _, _| ());

        let mut client = Client::new_with_handler(mock_handler);
        client.handle_command("go infinite");
    }

    #[test]
    fn test_handle_go_multiples_args() {
        let mut mock_handler = UCIHandler::default();
        mock_handler
            .expect_handle_go::<Output<std::io::Stdout>>()
            .with(
                predicate::always(),
                predicate::always(),
                predicate::eq(String::from(" depth 5 movestogo 5 nodes 1")),
            )
            .times(1)
            .returning(|_, _, _| ());

        let mut client = Client::new_with_handler(mock_handler);
        client.handle_command("go depth 5 movestogo 5 nodes 1");
    }

    #[test]
    fn test_handle_go_multiples_args_with_moves() {
        let mut mock_handler = UCIHandler::default();
        mock_handler
            .expect_handle_go::<Output<std::io::Stdout>>()
            .with(
                predicate::always(),
                predicate::always(),
                predicate::eq(String::from(
                    " depth 5 movestogo 5 searchmoves e2e4 d2d4 nodes 1",
                )),
            )
            .times(1)
            .returning(|_, _, _| ());

        let mut client = Client::new_with_handler(mock_handler);
        client.handle_command("go depth 5 movestogo 5 searchmoves e2e4 d2d4 nodes 1");
    }

    #[test]
    fn test_handle_stop() {
        let mut mock_handler = UCIHandler::default();
        mock_handler
            .expect_handle_stop::<Output<std::io::Stdout>>()
            .with(predicate::always(), predicate::always())
            .times(1)
            .returning(|_, _| ());

        let mut client = Client::new_with_handler(mock_handler);
        client.handle_command("stop");
    }

    #[test]
    fn test_handle_ponderhit() {
        let mut mock_handler = UCIHandler::default();
        mock_handler
            .expect_handle_ponderhit::<Output<std::io::Stdout>>()
            .with(predicate::always(), predicate::always())
            .times(1)
            .returning(|_, _| ());

        let mut client = Client::new_with_handler(mock_handler);
        client.handle_command("ponderhit");
    }
}
