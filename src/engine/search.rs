use super::evaluator::evaluate_board;
use crate::game::Game;
use crate::generator::generate_legal_moves;
use crate::r#move::Move;

pub struct Searcher {
    pub game: Game,
    search_depth: u32,
}

impl Searcher {
    pub fn new(game: Game, search_depth: u32) -> Self {
        Searcher { game, search_depth }
    }

    pub fn get_best_move(&mut self) -> Result<Move, &str> {
        let legal_moves = generate_legal_moves(self.game.current_board());
        let is_white_turn = self.game.current_board().is_white_turn();

        if legal_moves.len() == 0 {
            return Err("No legal moves available.");
        }

        // Initialise to a high value
        let mut best_move_eval = if is_white_turn { -9999.0 } else { 9999.0 };
        let mut best_move = legal_moves[0];

        for m in legal_moves {
            self.game.apply_move(m);
            let curr_eval = if is_white_turn {
                self.alpha_beta_min(self.search_depth - 1, -10000.0, 10000.0)
            } else {
                self.alpha_beta_max(self.search_depth - 1, -10000.0, 10000.0)
            };
            self.game.undo_move();

            if (is_white_turn && curr_eval > best_move_eval)
                || (!is_white_turn && curr_eval < best_move_eval)
            {
                best_move = m;
                best_move_eval = curr_eval;
            }
        }
        Ok(best_move)
    }

    // Inspired by https://www.chessprogramming.org/Alpha-Beta
    fn alpha_beta_max(&mut self, depth: u32, mut alpha: f32, beta: f32) -> f32 {
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

    fn alpha_beta_min(&mut self, depth: u32, alpha: f32, mut beta: f32) -> f32 {
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
