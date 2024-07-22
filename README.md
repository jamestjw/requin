# requin

A UCI chess program. It also comes with a chess engine (that is currently decently strong, i.e. about 2000 Lichess strength). You may play me on [lichess](https://lichess.org/@/requinBOT).

## Design
### Generation of Legal Moves
All possible piece displacements are first generated, then to check if a certain move is legal (i.e. whether it would endanger the king) we try applying the move to the board and if the king is not capturable then the move can be considered legal.

### Checks
After applying a move to the board, we check if the player that just moved has a move that captures the enemy king, if so then the last move checks the enemy king.

### Search
The engine employs a simple evaluation function that takes into account the raw value of pieces and their positional values. The positional values are calculated using a technique called [tapered eval](https://www.chessprogramming.org/Tapered_Eval) to ensure a smooth transition between the middlegame and the endgame. To evaluate the principal variation, the engine uses minimax with alpha-beta pruning. The engine also employs [quiescence search](https://www.chessprogramming.org/Quiescence_Search) to evaluate certain forcing moves (for now this only includes captures).

#### Pruning
We use [delta pruning](https://www.chessprogramming.org/Delta_Pruning) in quiescence search to avoid calculating positions that are hopeless. We also use [static exchange evaluation](https://www.chessprogramming.org/Static_Exchange_Evaluation) to skip lines that are illogical in quiescence search. [Futility pruning](https://www.chessprogramming.org/Futility_Pruning) is also used in alpha-beta search to prune lines that appear to be futile (i.e. not worth calculating) at depth 1.

#### Transposition Table
The engine employs a transition table to remember the assessment of a position that it has seen before. This occassionally helps in removing the need to reassess a position.

### Evaluation function
The engine takes into account the following factors when evaluating a position:
- Material count
- Piece activity
- King safety

## Weaknesses
* The endgame
    - The engine does not know how to win certain trivial endgames, more knowledge about endgame techniques would have to be programmed. Endgame tablebases should also be included.
* The opening
    - The engine seems to be play dubious moves in the opening, it could be useful to use an opening book.

## How to run?
You have to first install [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) (the Rust package manager). Then, you can compile the program by running

```bash
cargo build --release
```

Note: This application requires **nightly** Rust.

### Run tests
```bash
cargo test            # Run all tests
cargo test --lib      # Run unit tests
cargo test --test '*' # Run integration tests
```
Tests can be run with the `--release` flag, this is especially beneficial for integration tests.
