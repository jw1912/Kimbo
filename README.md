# Kimbo

Kimbo is a uci-compatible chess engine. 

#### Move Generation
- handled in [kimbo_state](https://github.com/JacquesRW/kimbo_state)
- fully legal
- classical bitboard approach

#### Search
- alpha-beta (negamax) search
- quiescence search
- MVV-LVA move ordering
- hash move
- iterative deepening
- check extensions
- mate pruning

#### Evaluation
- tapered midgame to endgame
- material
- piece-square tables

#### Features currently being tested
- hash score pruning
- late move reductions and pruning
- razoring
