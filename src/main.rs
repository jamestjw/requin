use requin::board::Board;
use requin::generator::generate_moves;

fn main() {
    let board = Board::new_starting_pos();
    board.print();

    println!("Legal moves in this position:");

    for candidate_move in generate_moves(&board) {
        println!("{}", candidate_move.to_algebraic_notation());
    }
}
