mod eval;
#[rustfmt::skip]
mod pst;
mod moves;
/// transposition table stuff
pub mod transposition;
mod zobrist;

use eval::*;
use kimbo_state::*;
use std::sync::Arc;

use self::zobrist::{initialise_zobrist, ZobristVals};

/// The extended position used by the engine.
#[derive(Clone)]
pub struct EnginePosition {
    /// Basic position, for move generation and making moves
    pub board: Position,
    /// Midgame material scores
    pub mat_mg: [i16; 2],
    /// Midgame piece-square table scores
    pub pst_mg: [i16; 2],
    /// Endgame piece-square table scores
    pub pst_eg: [i16; 2],
    /// heuristic for game phase
    pub phase: i16,
    /// zobrist hash
    pub zobrist: u64,
    /// pointer to zobrist hash values
    pub zobrist_vals: Arc<ZobristVals>,
}
impl Default for EnginePosition {
    fn default() -> Self {
        Self::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
    }
}

/// Extended move context for incrementally updated eval fields
pub struct EngineMoveContext {
    ctx: MoveContext,
    mat_mg: [i16; 2],
    pst_mg: [i16; 2],
    pst_eg: [i16; 2],
    phase: i16,
    zobrist: u64,
}

impl EnginePosition {
    /// Initialise a new position from a fen string
    pub fn from_fen(s: &str) -> Self {
        let board = Position::from_fen(s);
        let mat_mg = calculate_mat_mg(&board);
        let pst_mg = calculate_pst_mg_scores(&board);
        let pst_eg = calculate_pst_eg_scores(&board);
        let phase = calculate_phase(&board);
        let zobrist_vals = Arc::new(ZobristVals::default());
        let zobrist = initialise_zobrist(&board, &zobrist_vals);
        Self {
            board,
            mat_mg,
            pst_mg,
            pst_eg,
            phase,
            zobrist,
            zobrist_vals,
        }
    }
}
