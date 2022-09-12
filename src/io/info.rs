pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
pub const FEATURES: &str = "
\nMove Generation:\n
 - separate repo\n
 - classical bitboard approach\n
 - fully-legal\n
 - move list on stack\n
Search:\n
 - negamax search\n
 - quiescence search\n
 - mvv-lva move ordering\n
 - hash moves\n
 - iterative deepening\n
 - check extensions\n
 - mate pruning\n
Evaluation:\n
 - tapered midgame to endgame\n
 - material\n
 - piece-square tables\n
";