# Kimbo

Kimbo is a uci-compatible chess engine. 

#### Working Features
- legal move generation handled in [kimbo_state](https://github.com/JacquesRW/kimbo_state)
- negamax framework
- MVV-LVA move ordering
- quiescence search
- transposition table
- iterative deepening
- check extensions

#### Features currently being tested
- late move reductions and pruning
- razoring

#### TODO

- add a field "active_side" to Search
- use "active_side" to determine friendly and enemy king in check during search