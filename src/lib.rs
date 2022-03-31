pub mod bitboard;
pub mod board;
pub mod engine;
pub mod game;
pub mod generator;
pub mod r#move;
pub mod parser;
pub mod uci;

pub use bitboard::init_tables;
use board::Board;
use engine::Searcher;
use game::Game;
use uci::Client;

use std::process::exit;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate bitfield;

extern crate exitcode;

pub fn clear_screen() {
    print!("{}[2J", 27 as char);
}

pub fn play_game_ai(ai_starts: bool, depth: u32, num_threads: usize) {
    let board = Board::new_starting_pos();
    let mut game = Game::new(board);
    game.init_game_board();
    let mut searcher = Searcher::new(game, depth, num_threads);

    init_tables();

    // 0 implies that it is the AI's turn
    let turn_seq = if ai_starts { [0, 1] } else { [1, 0] };
    let mut turn_seq_iterator = turn_seq.iter().cycle();

    loop {
        clear_screen();
        searcher.game.print_current_board();

        if searcher.game.is_game_over() {
            println!("Game over. Result: {}", searcher.game.state.to_text());
            exit(exitcode::OK);
        }

        if *turn_seq_iterator.next().unwrap() == 0 {
            searcher.apply_best_move();
        } else {
            // Get next move from user
            searcher.game.get_next_move();
        }
    }
}

pub fn play_game_pvp() {
    let board = Board::new_starting_pos();
    let mut game = Game::new(board);
    game.init_game_board();

    init_tables();

    loop {
        clear_screen();
        game.print_current_board();

        if game.is_game_over() {
            println!("Game over. Result: {}", game.state.to_text());
            exit(exitcode::OK);
        }

        game.get_next_move();
    }
}

pub fn run_uci() {
    let mut uci_client = Client::new();
    init_tables();
    uci_client.run();
}

const fn num_bits<T>() -> usize {
    std::mem::size_of::<T>() * 8
}

fn log_2(x: u64) -> usize {
    assert!(x > 0);
    num_bits::<u64>() as usize - x.leading_zeros() as usize - 1
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_log_2() {
        assert_eq!(log_2(1), 0);
        assert_eq!(log_2(2), 1);
        assert_eq!(log_2(3), 1);
        assert_eq!(log_2(4), 2);
        assert_eq!(log_2(8), 3);
    }
}
