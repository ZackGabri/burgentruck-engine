# Bergentrück Chess Engine
## Overview
Burgentruck (yes, we spelt it wrong) is a rust chess engine developed by ZackGabri and Tamplite Siphron Kents, the latter of whom developed the Dragonrose chess engine.

## Specifications
### Board representation
Burgentruck uses [shakmaty crate](https://crates.io/crates/Shakmaty) for board representation and movegen, a library developed by Niklas Fiekas (niklasf) et al. 

### Search
- Negamax with Alpha-beta Pruning (fail-soft)
- Quiescence search (fail-soft)
- Transposition table
- Move ordering: Hash move, MVV-LVA, quiet moves

### Evaluation
Currently using basic material + piece square tables (PST / PSQT) + small random value, inspired by Clockwork's older evaluation. In the future this will be used as a base to iteratively train neural networks, eventually allowing Burgentruck to use NNUE.

## Etymology
Burgentruck (Bergentrück) is a reference to Undertale / Deltarune, both games developed by Toby Fox et al. In Undertale, the soundtrack for Asgore's boss theme is Bergentrücking (German, lit. "King in the Mountain"), while in Deltarune there is a [popular meme](https://knowyourmeme.com/memes/asgore-running-over-dess) (animation + song) about Asgore running over December (Dess) Holiday with his truck. It is the combination of both of these references that lead to the culmination of the engine's name.
