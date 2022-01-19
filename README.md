# requin

A chess program that runs in the CLI. Current plans include coding up all the rules of the game and then building an AI that plays it.


## Design
### Generation of Legal Moves
All possible piece displacements are first generated, then to check if a certain move is legal (i.e. whether it would endanger the king) we try applying the move to the board and if the king is not capturable then the move can be considered legal.


### Checks
After applying a move to the board, we check if the player that just moved has a move that captures the enemy king, if so then the last move checks the enemy king.
