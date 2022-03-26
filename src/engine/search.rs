use super::evaluator::{evaluate_board, static_exchange_evaluation_capture};
use crate::generator::generate_legal_moves;
use crate::r#move::Move;
use crate::{
    game::{Game, GameState},
    generator::generate_non_quiescent_moves,
};

use std::sync::mpsc::channel;
use threadpool::ThreadPool;

static CHECKMATE_SCORE: i32 = 320000;
static STALEMATE_SCORE: i32 = 0;
static INITIAL_ALPHA: i32 = -CHECKMATE_SCORE - 1;
static INITIAL_BETA: i32 = CHECKMATE_SCORE + 1;
static FUTILITY_MARGIN: i32 = 300; // Equal to the value of a minor piece

#[derive(Clone)]
pub struct Searcher {
    pub game: Game,
    search_depth: u32,
    num_threads: usize,
    nodes_searched: u32,
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
            nodes_searched: 0,
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
                // Whether a move can be pruned depends on whether it is a capture
                let curr_eval = -searcher.alpha_beta(
                    search_depth,
                    INITIAL_ALPHA,
                    INITIAL_BETA,
                    is_white_turn,
                    !m.is_capture,
                );
                tx.send((m, curr_eval))
                    .expect("Unexpected error: Main thread is not receiving.");
            });
        }

        // Assuming that all moves are evaluated successfully without fail
        let mut move_evals: Vec<(Move, i32)> = rx.iter().take(num_legal_moves).collect();

        move_evals.sort_by(|(_, e1), (_, e2)| e2.cmp(e1));

        Ok(move_evals[0].0)
    }

    // Inspired by https://www.chessprogramming.org/Alpha-Beta
    // Alpha-beta pruning in the negamax framework
    pub fn alpha_beta(
        &mut self,
        depth: u32,
        mut alpha: i32,
        beta: i32,
        is_white: bool,
        can_prune: bool,
    ) -> i32 {
        match self.game.state {
            GameState::InProgress => {}
            GameState::WhiteWon | GameState::BlackWon => return -CHECKMATE_SCORE,
            GameState::Stalemate => return STALEMATE_SCORE,
        }

        if depth == 0 {
            return self.quiesce(alpha, beta, is_white);
        } else if depth == 1 {
            // Futility pruning
            let offset = if is_white { -1 } else { 1 };
            let eval = offset * evaluate_board(self.game.current_board());
            // If a move proves to be futile, we just return alpha since
            // further continuations are unlikely to raise alpha
            if eval + FUTILITY_MARGIN < alpha && can_prune {
                return alpha;
            }
        }

        let legal_moves = generate_legal_moves(self.game.current_board());

        for m in legal_moves {
            self.nodes_searched += 1;

            self.game.apply_move(m);
            // Whether or not a node can be pruned depends on whether
            // the move was a 'peaceful' move
            let score = -self.alpha_beta(depth - 1, -beta, -alpha, !is_white, !m.is_capture);
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

    pub fn quiesce(&mut self, mut alpha: i32, beta: i32, is_white: bool) -> i32 {
        let offset = if is_white { -1 } else { 1 };
        let stand_pat = offset * evaluate_board(self.game.current_board());

        if stand_pat >= beta {
            return beta;
        }

        if alpha < stand_pat {
            alpha = stand_pat;
        }

        // Sort moves based on SEE
        let mut non_quiescent_moves = generate_non_quiescent_moves(self.game.current_board())
            .into_iter()
            .map(|m| {
                (
                    m,
                    static_exchange_evaluation_capture(self.game.current_board().clone(), &m),
                )
            })
            .collect::<Vec<(Move, i32)>>();
        non_quiescent_moves.sort_by(|(_, see1), (_, see2)| see2.cmp(see1));

        for (m, see) in non_quiescent_moves {
            // Prune captures with SEE < 0
            if see < 0 {
                break;
            }
            self.nodes_searched += 1;

            self.game.apply_move(m);
            let score = -self.quiesce(-beta, -alpha, !is_white);
            self.game.undo_move();

            if score >= beta {
                return beta;
            }

            if score > alpha {
                alpha = score;
            }
        }

        return alpha;
    }

    pub fn apply_best_move(&mut self) {
        match self.get_best_move() {
            Ok(m) => {
                self.game.apply_move(m);
            }
            Err(e) => panic!("Unable to apply best move. Error: {}", e),
        }
    }

    pub fn get_nodes_searched(&self) -> u32 {
        self.nodes_searched
    }
}
