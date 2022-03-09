# requin

A chess program that runs in the CLI. It also comes with a chess engine (that is currently not too strong).

## Design
### Generation of Legal Moves
All possible piece displacements are first generated, then to check if a certain move is legal (i.e. whether it would endanger the king) we try applying the move to the board and if the king is not capturable then the move can be considered legal.

### Checks
After applying a move to the board, we check if the player that just moved has a move that captures the enemy king, if so then the last move checks the enemy king.

### Search
The engine employs a simple evaluation function that takes into account the raw value of pieces and their positional values. To evaluate the principal variation, the engine uses minimax with alpha-beta pruning.

## How to run?
You have to first install [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) (the Rust package manager). Then, you can compile the program by running

```bash
cargo build --release
```