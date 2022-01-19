use requin::board::Board;
use requin::generator::generate_legal_moves;

fn main() {
    let board = Board::new_starting_pos();
    board.print();

    println!("Legal moves in this position:");

    for candidate_move in generate_legal_moves(&board) {
        println!("{}", candidate_move.to_algebraic_notation());
    }
}
