# Kimbo

Kimbo is a UCI compatible chess engine written in Rust. It is now succeeded by [akimbo](https://github.com/JacquesRW/akimbo).


#### Compiling
If you have cargo installed, run ```cargo build --release --bin kimbo```.

#### ELO

| Version | Release Date | CCRL Blitz | CCRL 40/15 |
| :-----: | :----------: | :--------: | :--------: |
| [0.2.1](https://github.com/JacquesRW/Kimbo/releases/tag/v0.2.1)   | 20th September 2022  | 2205 |  -   |
| [0.3.0](https://github.com/JacquesRW/Kimbo/releases/tag/v0.3.0)   | 10th October 2022    |  -   | 2476 |
| [1.0.0](https://github.com/JacquesRW/Kimbo/releases/tag/v1.0.0)   | 21st November 2022   | TBD  | TBD  |

## Features

#### Move Generation
- Bitboards
- Fully-legal
- Hyperbola quintessence sliding attacks

#### Search
- Fail-soft
- Principle variation search
- Quiescence search
- Iterative deepening
- Check extensions

#### Move Ordering
1. Hash move
2. Captures, sorted by MVV-LVA
3. Promotions
4. Killer moves
5. Counter moves
6. Castling
4. Quiets, sorted by history heuristic

#### Evaluation
- Tapered from midgame to endgame
- Material
- Piece-square tables
- Pawn structure (w/ basic king safety)

#### Pruning/Reductions
- Mate distance pruning
- Hash score pruning
- Variable late move reductions
- Reverse futility pruning
- Null move pruning
- Delta pruning
