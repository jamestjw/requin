use requin::board::Board;
use requin::game::Game;

fn main() {
    let board = Board::new_starting_pos();
    let mut game = Game::new(board);

    game.play_game();
}
