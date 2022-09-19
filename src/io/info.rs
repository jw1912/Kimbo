pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
pub const FEATURES: &str = "
\nMove Generation:\n
 - separate repo\n
 - hyperbola quintessence bitboard approach\n
 - fully-legal\n
 - move list on stack\n
Search:\n
 - negamax search\n
 - quiescence search\n
 - iterative deepening\n
 - check extensions\n
 - hash score and mate pruning\n
Move Ordering:\n
 - hash move\n
 - captures sorted by mvv-lva\n
 - promotions\n
 - killer moves\n
 - counter moves\n
 - castling\n
 - quiets\n
Evaluation:\n
 - tapered midgame to endgame\n
 - material\n
 - piece-square tables\n
 - pawn structure (w/ basic king safety)\n
";