use super::evaluator::{evaluate_board, static_exchange_evaluation_capture};
use crate::r#move::Move;
use crate::{
    game::{Game, GameState},
    generator::generate_non_quiescent_moves,
};

use std::sync::mpsc::channel;
use threadpool::ThreadPool;

static CHECKMATE_SCORE: i32 = 320000;
static DRAW_SCORE: i32 = 0;
static INITIAL_ALPHA: i32 = -CHECKMATE_SCORE - 1;
static INITIAL_BETA: i32 = CHECKMATE_SCORE + 1;
static FUTILITY_MARGIN: i32 = 800; // Equal to the value of a minor piece
static DELTA_PRUNING_THRESHOLD: i32 = 2538; // Value of a queen

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
        let legal_moves = self.game.current_legal_moves();
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
            let m = m.clone();
            searcher.game.apply_move(&m);
            pool.execute(move || {
                // Whether a move can be pruned depends on whether it is a capture
                let curr_eval = -searcher.alpha_beta(
                    search_depth,
                    INITIAL_ALPHA,
                    INITIAL_BETA,
                    is_white_turn,
                    !m.is_capture,
                    1, // Start with search depth 1
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
        remaining_depth: u32,
        mut alpha: i32,
        beta: i32,
        is_white: bool,
        can_prune: bool,
        mut searched_depth: u32,
    ) -> i32 {
        match self.game.state {
            GameState::InProgress => {}
            GameState::WhiteWon | GameState::BlackWon => {
                return -(CHECKMATE_SCORE - searched_depth as i32)
            }
            GameState::Stalemate => return DRAW_SCORE,
        }
        if self.game.is_threefold_repetition() {
            return DRAW_SCORE;
        }

        searched_depth += 1;

        if remaining_depth == 0 {
            return self.quiesce(alpha, beta, is_white, searched_depth);
        } else if remaining_depth == 1 {
            // Futility pruning
            let offset = if is_white { -1 } else { 1 };
            let eval = offset * evaluate_board(self.game.current_board());
            // If a move proves to be futile, we just return alpha since
            // further continuations are unlikely to raise alpha
            if eval + FUTILITY_MARGIN < alpha && can_prune {
                return alpha;
            }
        }

        // Move ordering
        // 1. Good captures
        // 2. Bad captures
        // 3. Non-captures
        let mut legal_moves = self
            .game
            .current_legal_moves()
            .clone()
            .into_iter()
            .map(|m| {
                (
                    m,
                    if m.is_capture {
                        static_exchange_evaluation_capture(self.game.current_board().clone(), &m)
                    } else {
                        // Give non-captures a low score for them to be evaluated last
                        i32::MIN
                    },
                )
            })
            .collect::<Vec<(Move, i32)>>();

        legal_moves.sort_by(|(_, score1), (_, score2)| score2.cmp(score1));

        for (m, _) in legal_moves {
            self.nodes_searched += 1;
            self.game.apply_move(&m);
            // Whether or not a node can be pruned depends on whether
            // the move was a 'peaceful' move
            let score = -self.alpha_beta(
                remaining_depth - 1,
                -beta,
                -alpha,
                !is_white,
                !m.is_capture,
                searched_depth,
            );
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

    pub fn quiesce(
        &mut self,
        mut alpha: i32,
        beta: i32,
        is_white: bool,
        mut searched_depth: u32,
    ) -> i32 {
        match self.game.state {
            GameState::InProgress => {}
            GameState::WhiteWon | GameState::BlackWon => {
                return -(CHECKMATE_SCORE - searched_depth as i32)
            }
            GameState::Stalemate => return DRAW_SCORE,
        }
        if self.game.is_threefold_repetition() {
            return DRAW_SCORE;
        }

        searched_depth += 1;

        let offset = if is_white { -1 } else { 1 };
        let stand_pat = offset * evaluate_board(self.game.current_board());

        // Do not return stand-pat if in check
        if stand_pat >= beta && !self.game.current_board().is_in_check() {
            return beta;
        }

        // Delta pruning
        if stand_pat < alpha - DELTA_PRUNING_THRESHOLD {
            // If giving a side a queen is not good enough,
            // then we conclude that further searches are futile
            return alpha;
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
            if see < 0 && !self.game.current_board().is_in_check() {
                break;
            }
            self.nodes_searched += 1;

            self.game.apply_move(&m);
            let score = -self.quiesce(-beta, -alpha, !is_white, searched_depth);
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
                self.game.apply_move(&m);
            }
            Err(e) => panic!("Unable to apply best move. Error: {}", e),
        }
    }

    pub fn get_nodes_searched(&self) -> u32 {
        self.nodes_searched
    }
}
