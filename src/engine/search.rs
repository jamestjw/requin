use super::evaluator::evaluate_board;
use crate::game::{Game, GameState};
use crate::generator::generate_legal_moves;
use crate::r#move::Move;

use std::sync::mpsc::channel;
use threadpool::ThreadPool;

#[derive(Clone)]
pub struct Searcher {
    pub game: Game,
    search_depth: u32,
    num_threads: usize,
}

impl Searcher {
    pub fn new(game: Game, search_depth: u32, num_threads: usize) -> Self {
        if num_threads <= 0 {
            panic!("The engine requires at least one thread to run searches.")
        }
        Searcher {
            game,
            search_depth,
            num_threads,
        }
    }

    pub fn get_best_move(&mut self) -> Result<Move, &str> {
        let legal_moves = generate_legal_moves(self.game.current_board());
        let num_legal_moves = legal_moves.len();
        let is_white_turn = self.game.current_board().is_white_turn();

        if num_legal_moves == 0 {
            return Err("No legal moves available.");
        }

        let pool = ThreadPool::new(self.num_threads);

        // Workers will send results via tx, main thread
        // receives results via tx
        let (tx, rx) = channel();

        for m in legal_moves {
            let tx = tx.clone();
            let mut searcher = self.clone();
            let search_depth = self.search_depth - 1;
            searcher.game.apply_move(m);
            pool.execute(move || {
                let curr_eval = if is_white_turn {
                    searcher.alpha_beta_min(search_depth, -10000.0, 10000.0)
                } else {
                    searcher.alpha_beta_max(search_depth, -10000.0, 10000.0)
                };
                tx.send((m, curr_eval))
                    .expect("Unexpected error: Main thread is not receiving.");
            });
        }

        // Assuming that all moves are evaluated successfully without fail
        let mut move_evals: Vec<(Move, f32)> = rx.iter().take(num_legal_moves).collect();

        // Sort by descending order if it is white's turn
        if is_white_turn {
            move_evals.sort_by(|(_, e1), (_, e2)| e2.partial_cmp(e1).unwrap());
        } else {
            move_evals.sort_by(|(_, e1), (_, e2)| e1.partial_cmp(e2).unwrap());
        }

        Ok(move_evals[0].0)
    }

    // Inspired by https://www.chessprogramming.org/Alpha-Beta
    pub fn alpha_beta_max(&mut self, depth: u32, mut alpha: f32, beta: f32) -> f32 {
        match self.game.state {
            GameState::WhiteWon => return 9998.0,
            GameState::BlackWon => return -9998.0,
            GameState::Stalemate => return 0.0,
            _ => {}
        }

        if depth == 0 {
            return evaluate_board(self.game.current_board());
        }

        let legal_moves = generate_legal_moves(self.game.current_board());

        for m in legal_moves {
            self.game.apply_move(m);
            let score = self.alpha_beta_min(depth - 1, alpha, beta);
            self.game.undo_move();

            if score >= beta {
                return beta;
            }

            if score > alpha {
                alpha = score;
            }
        }

        alpha
    }

    pub fn alpha_beta_min(&mut self, depth: u32, alpha: f32, mut beta: f32) -> f32 {
        match self.game.state {
            GameState::WhiteWon => return 9998.0,
            GameState::BlackWon => return -9998.0,
            GameState::Stalemate => return 0.0,
            _ => {}
        }

        if depth == 0 {
            return evaluate_board(self.game.current_board());
        }

        let legal_moves = generate_legal_moves(self.game.current_board());

        for m in legal_moves {
            self.game.apply_move(m);
            let score = self.alpha_beta_max(depth - 1, alpha, beta);
            self.game.undo_move();

            if score <= alpha {
                return alpha;
            }

            if score < beta {
                beta = score;
            }
        }

        beta
    }

    pub fn apply_best_move(&mut self) {
        match self.get_best_move() {
            Ok(m) => {
                self.game.apply_move(m);
            }
            Err(e) => panic!("Unable to apply best move. Error: {}", e),
        }
    }
}
