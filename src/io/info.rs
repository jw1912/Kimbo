pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
pub const FEATURES: &str = "
Move Generation:
- Bitboards
- Fully-legal
- Hyperbola quintessence sliding attacks

Search:
- Fail-soft
- Principle variation search
- Quiescence search
- Iterative deepening
- Check extensions

Move Ordering:
1. Hash move
2. Captures, sorted by MVV-LVA
3. Promotions
4. Killer moves
5. Counter moves
6. Castling
4. Quiets, sorted by history heuristic

Evaluation:
- Tapered from midgame to endgame
- Material
- Piece-square tables
- Pawn structure (w/ basic king safety)

Pruning/Reductions:
- Mate distance pruning
- Hash score pruning
- Variable late move reductions
- Reverse futility pruning
- Null move pruning
- Delta pruning
";
