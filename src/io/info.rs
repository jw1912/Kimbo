pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
pub const FEATURES: &str = "
Move Generation:
 - fully-legal
 - bitboard based
 - classical/hyperbola quintessence sliding attacks
 - move list on stack\n
Search:
 - negamax search
 - quiescence search
 - iterative deepening
 - check extensions
 - null move pruning
 - late move reductions (searched by PVS)
 - hash score and mate pruning\n
Move Ordering:
 - hash move
 - captures sorted by mvv-lva
 - promotions
 - killer moves
 - counter moves
 - castling
 - quiets\n
Evaluation:
 - tapered midgame to endgame
 - material
 - piece-square tables
 - pawn structure (w/ basic king safety)\n
";