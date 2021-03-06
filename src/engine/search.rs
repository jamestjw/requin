use super::evaluator::{evaluate_board, static_exchange_evaluation_capture};
use super::tt::{
    build_new_tt, NodeType, TranspositionTable, TranspositionTableEntry,
    TranspositionTableEntryMoveData, TranspositionTableEntrySearchData,
};
use crate::board::Color;
use crate::r#move::Move;
use crate::{
    game::{Game, GameState},
    generator::generate_non_quiescent_moves,
};

use std::sync::mpsc::channel;
use std::time::{Duration, Instant};
use threadpool::ThreadPool;

static CHECKMATE_SCORE: i32 = 320000;
static DRAW_SCORE: i32 = 0;
static INITIAL_ALPHA: i32 = -CHECKMATE_SCORE - 1;
static INITIAL_BETA: i32 = CHECKMATE_SCORE + 1;
static FUTILITY_MARGIN_1: i32 = 800; // Approximately equal to the value of a minor piece
static FUTILITY_MARGIN_2: i32 = 1300; // Approximately equal to the value of a rook
static DELTA_PRUNING_THRESHOLD: i32 = 2538; // Value of a queen
static NULL_MOVE_PRUNING_R: u8 = 2;

#[derive(Clone)]
pub struct Searcher {
    pub game: Game,
    search_depth: u8,
    num_threads: usize,
    nodes_searched: u32,
    tt: TranspositionTable,
}

impl Searcher {
    pub fn new(game: Game, search_depth: u8, num_threads: usize) -> Self {
        if num_threads <= 0 {
            panic!("The engine requires at least one thread to run searches.")
        }
        Searcher {
            game,
            search_depth,
            num_threads,
            nodes_searched: 0,
            tt: build_new_tt(),
        }
    }

    /// # Arguments
    ///
    /// * `time_limit` - Maximum search time in milliseconds
    pub fn get_best_move(&mut self, time_limit: Option<u32>) -> Result<Move, &str> {
        let mut legal_moves = self.game.current_legal_moves().clone();
        let num_legal_moves = legal_moves.len();
        let is_white_turn = self.game.current_board().is_white_turn();
        // Save zobrist to fill up the TT
        let zobrist = self.game.get_current_zobrist();

        if num_legal_moves == 0 {
            return Err("No legal moves available.");
        }

        // Workers will send results via tx, main thread
        // receives results via tx
        let (tx, rx) = channel();

        let max_search_depth = self.search_depth;
        let mut best_move: Option<Move> = None;
        let start_time = Instant::now();
        let time_limit = time_limit.map(|l| Duration::from_millis(l as u64));
        let deadline = time_limit.map(|l| start_time + l);

        // Iterative deepening
        for current_search_depth in 1..=max_search_depth {
            let elapsed_time = Instant::now().duration_since(start_time);

            // Consider skipping the current iteration if the time situation is not good
            if time_limit.is_some() && best_move.is_some() {
                // Check if 50% of allocated time has been used
                if elapsed_time.div_duration_f32(time_limit.unwrap()) > 0.5 {
                    return Ok(best_move.unwrap());
                }
            }

            let pool = ThreadPool::with_name("requin_searchers".to_string(), self.num_threads);
            // Search the best move first, this is useful when the num of available threads is low.
            legal_moves.sort_by_key(|m| if Some(m) == best_move.as_ref() { 0 } else { 1 });
            for m in &legal_moves {
                let tx = tx.clone();
                let mut searcher = self.clone();
                let search_depth = current_search_depth - 1;
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

            // We only apply the deadline if we already have a best move, since we need
            // to forcefully return the best move if we breach the deadline
            let mut move_evals: Vec<(Move, i32)> = if deadline.is_some() && best_move.is_some() {
                let mut res = Vec::with_capacity(num_legal_moves);
                for _ in 0..num_legal_moves {
                    match rx.recv_deadline(deadline.unwrap()) {
                        Ok(m) => {
                            res.push(m);
                        }
                        Err(_) => {
                            // If we timeout, then return the best move that currently have
                            return Ok(best_move.unwrap());
                        }
                    }
                }
                res
            } else {
                // Assuming that all moves are evaluated successfully without fail
                rx.iter().take(num_legal_moves).collect()
            };

            move_evals.sort_by(|(_, e1), (_, e2)| e2.cmp(e1));

            let candidate_move = move_evals[0].0;
            let candidate_move_score = move_evals[0].1;

            // Insert into TT
            self.tt.set_entry(
                zobrist,
                build_tt_entry(
                    Some(&candidate_move),
                    zobrist,
                    current_search_depth,
                    candidate_move_score,
                    NodeType::PV,
                ),
            );
            best_move = Some(candidate_move);
        }

        Ok(best_move.unwrap())
    }

    // Inspired by https://www.chessprogramming.org/Alpha-Beta
    // Alpha-beta pruning in the negamax framework
    pub fn alpha_beta(
        &mut self,
        remaining_depth: u8,
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
            if eval + FUTILITY_MARGIN_1 < alpha && can_prune {
                return alpha;
            }
        } else if remaining_depth == 2 {
            // Extended futility pruning
            let offset = if is_white { -1 } else { 1 };
            let eval = offset * evaluate_board(self.game.current_board());
            // If a move proves to be futile, we just return alpha since
            // further continuations are unlikely to raise alpha
            if eval + FUTILITY_MARGIN_2 < alpha && can_prune {
                return alpha;
            }
        }

        // To be used to read and write to TT
        let zobrist = self.game.get_current_zobrist();
        let tt_entry = self.tt.get_entry(zobrist);
        let hash_move = if tt_entry.is_valid(zobrist) {
            let tt_search_data = tt_entry.get_search_data();
            // If this move has already been searched before, just return the score
            if tt_search_data.depth() >= remaining_depth
                && tt_search_data.node_type() == NodeType::PV
            {
                return tt_search_data.score();
            }

            if tt_search_data.node_type() == NodeType::PV
                || tt_search_data.node_type() == NodeType::Cut
            {
                let tt_move_data = tt_entry.get_move_data();
                let ppt = if tt_move_data.best_move_is_promotion() {
                    Some(tt_move_data.best_move_ppt())
                } else {
                    None
                };
                self.game
                    .current_board()
                    .build_move_with_src_dest(
                        tt_move_data.best_move_src(),
                        tt_move_data.best_move_dest(),
                        ppt,
                    )
                    .ok()
            } else {
                None
            }
        } else {
            None
        };

        // Move ordering
        // 1. Hash move
        // 2. Good captures
        // 3. Bad captures
        // 4. Non-captures
        let mut legal_moves = self
            .game
            .current_legal_moves()
            .clone()
            .into_iter()
            .map(|m| {
                (
                    m,
                    if hash_move.is_some() && hash_move.unwrap() == m {
                        // Test the hash move first
                        // TODO: Test the hash move without generating other moves
                        i32::MAX
                    } else if m.is_capture {
                        static_exchange_evaluation_capture(self.game.current_board(), &m)
                    } else {
                        // Give non-captures a low score for them to be evaluated last
                        i32::MIN
                    },
                )
            })
            .collect::<Vec<(Move, i32)>>();

        legal_moves.sort_by(|(_, score1), (_, score2)| score2.cmp(score1));

        // Maybe do null move pruning
        if self.may_do_null_move_pruning(remaining_depth - 1, is_white) {
            self.game.apply_null_move();
            // Do an alpha beta search with reduced depth
            let score = -self.alpha_beta(
                remaining_depth - 1 - NULL_MOVE_PRUNING_R,
                -beta,
                -alpha,
                !is_white,
                false,
                searched_depth - 1,
            );
            self.game.undo_move();
            if score >= beta {
                return beta;
            }
        }

        let mut best_move: Option<Move> = None;

        for (m, _) in legal_moves {
            // TODO: Fix how null move pruning makes this value more than
            // what it should be
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
                // TODO: Figure out the score to save here
                self.tt.set_entry(
                    zobrist,
                    build_tt_entry(
                        Some(&m),
                        zobrist,
                        remaining_depth as u8,
                        score,
                        NodeType::Cut,
                    ),
                );
                return beta;
            }

            if score > alpha {
                best_move = Some(m);
                alpha = score;
            }
        }

        self.tt.set_entry(
            zobrist,
            build_tt_entry(
                best_move.as_ref(),
                zobrist,
                remaining_depth as u8,
                alpha,
                if best_move.is_some() {
                    NodeType::PV
                } else {
                    NodeType::All
                },
            ),
        );

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
                    static_exchange_evaluation_capture(&self.game.current_board(), &m),
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
        match self.get_best_move(None) {
            Ok(m) => {
                self.game.apply_move(&m);
            }
            Err(e) => panic!("Unable to apply best move. Error: {}", e),
        }
    }

    pub fn get_nodes_searched(&self) -> u32 {
        self.nodes_searched
    }

    pub fn may_do_null_move_pruning(&self, remaining_depth: u8, is_white: bool) -> bool {
        // Ensure that there is still a sufficient amount of material on the board
        // Ensure that the side to move is not in check
        // Ensure that the remaining depth is more than the pruning window
        let color = if is_white { Color::White } else { Color::Black };
        self.game
            .current_board()
            .get_non_king_pawn_bb_for_color(color)
            != 0
            && !self.game.is_in_check()
            && remaining_depth >= NULL_MOVE_PRUNING_R
    }
}

fn build_tt_entry(
    candidate_move: Option<&Move>,
    key: u64,
    depth: u8,
    score: i32,
    node_type: NodeType,
) -> TranspositionTableEntry {
    let tt_move_data = match candidate_move {
        Some(candidate_move) => {
            let (move_src, move_dest, move_ppt) = candidate_move.to_src_dest();
            TranspositionTableEntryMoveData::new(move_src, move_dest, move_ppt)
        }
        None => TranspositionTableEntryMoveData(0),
    };

    let mut tt_search_data = TranspositionTableEntrySearchData(0);
    tt_search_data.set_depth(depth);
    tt_search_data.set_score(score);
    tt_search_data.set_node_type(node_type as u8);
    TranspositionTableEntry::new(key, tt_move_data, tt_search_data)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::board::Board;

    #[test]
    fn test_tt_filled_at_root_depth() {
        let board = Board::new_starting_pos();
        let zobrist_to_inspect = board.get_zobrist();
        let game = Game::new(board);
        let mut searcher = Searcher::new(game, 1, 32);

        let best_move = searcher.get_best_move(None).unwrap();

        let tt_entry = searcher.tt.get_entry(zobrist_to_inspect);
        let tt_move_data = tt_entry.get_move_data();
        let tt_search_data = tt_entry.get_search_data();

        assert!(tt_entry.is_valid(zobrist_to_inspect));
        assert_eq!(tt_entry.get_key(), zobrist_to_inspect);
        assert_eq!(tt_search_data.depth(), 1);
        assert_ne!(tt_search_data.score(), 0);
        assert_eq!(tt_search_data.node_type(), NodeType::PV);
        assert_eq!(tt_move_data.best_move_src(), best_move.src);
        assert_eq!(tt_move_data.best_move_dest(), best_move.dest);
    }
}
