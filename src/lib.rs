pub mod board;
pub mod engine;
pub mod game;
pub mod generator;
pub mod r#move;

use board::Board;
use engine::Searcher;
use game::Game;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate bitfield;

pub fn clear_screen() {
    print!("{}[2J", 27 as char);
}

pub fn play_game_ai(ai_starts: bool, depth: u32) {
    let board = Board::new_starting_pos();
    let game = Game::new(board);
    let mut searcher = Searcher::new(game, depth);

    // 0 implies that it is the AI's turn
    let turn_seq = if ai_starts { [0, 1] } else { [1, 0] };
    let mut turn_seq_iterator = turn_seq.iter().cycle();

    loop {
        clear_screen();
        searcher.game.print_current_board();

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

    loop {
        clear_screen();
        game.print_current_board();
        game.get_next_move();
    }
}
