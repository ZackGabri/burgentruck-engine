# Bergentrück Chess Engine
## Overview
Burgentruck (yes, we spelt it wrong) is a rust chess engine developed by ZackGabri and Tamplite Siphron Kents, the latter of whom developed the Dragonrose chess engine.

## Specifications
### Strength 
| Version | CCRL 2+1 |
|:-------:|:--------:|
|  v0.1   |  ~1879   |

### Board representation
Burgentruck uses [shakmaty crate](https://crates.io/crates/Shakmaty) for board representation and movegen, a library developed by Niklas Fiekas (niklasf) et al. 

### Search
- Negamax with Alpha-beta Pruning (fail-soft)
- Quiescence search (fail-soft)
- Transposition table
- Principal variation search (PVS)
- Reverse futility pruning (RFP)
- Null move pruning (NMP)
- Move ordering: Hash move, MVV-LVA, quiet history moves, quiet moves

### Evaluation
Currently using basic material + piece square tables (PST / PSQT) + small random value, inspired by Clockwork's older evaluation. In the future this will be used as a base to iteratively train neural networks, eventually allowing Burgentruck to use NNUE.

## Etymology
Burgentruck (Bergentrück) is a reference to Undertale / Deltarune, both games developed by Toby Fox et al. In Undertale, the soundtrack for Asgore's boss theme is Bergentrücking (German, lit. "King in the Mountain"), while in Deltarune there is a [popular meme](https://knowyourmeme.com/memes/asgore-running-over-dess) (animation + song) about Asgore running over December (Dess) Holiday with his truck. It is the combination of both of these references that lead to the culmination of the engine's name.

## Changelogs
v0.1: First release. Search progression up to NMP except aspiration windows, which failed to gain. Tested against Stash v19 (120W 295D 585L, Book: UHO_2022_8mvs_+130_+139).
