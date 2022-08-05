mod eval;
mod qsearch;
mod search;
#[rustfmt::skip]
mod pst;

use kimbo_state::*;
use eval::*;
use pst::get_mg_weight;

/// The extended position used by the engine.
pub struct EnginePosition {
    /// Basic position, for move generation and making moves
    pub board: Position,
    /// Incrementally updated material scores for each side
    pub material_scores: [i16; 2],
    /// Midgame piece-square table scores
    pub pst_mg: [i16; 2],
    /// Endgame piece-square table scores
    pub pst_eg: [i16; 2]
}

impl Default for EnginePosition {
    fn default() -> Self {
        Self::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
    }
}

impl EnginePosition {
    /// Initialise a new position from a fen string
    pub fn from_fen(s: &str) -> Self {
        let board = Position::from_fen(s);
        let material_scores = calculate_material_scores(&board);
        let pst_mg = calculate_pst_mg_scores(&board);
        let pst_eg = calculate_pst_eg_scores(&board);
        Self { board, material_scores, pst_mg, pst_eg }
    } 

    // TODO: Refactor to match the move flag

    /// Making move, updates engine's scores as well
    pub fn make_move(&mut self, m: u16) -> MoveContext {
        let ctx = self.board.make_move(m);
        if ctx.captured_pc != 7 {
            self.material_scores[self.board.side_to_move] -= PIECE_VALS[ctx.captured_pc as usize];
            // WARNING: overlooks en passant
            self.pst_mg[self.board.side_to_move] -= get_mg_weight(((ctx.m >> 6) & 0b111111) as usize, self.board.side_to_move, ctx.captured_pc as usize);
        }
        let opp = self.board.side_to_move ^ 1;
        self.pst_mg[opp] -= get_mg_weight((ctx.m & 0b111111) as usize, opp, ctx.moved_pc as usize);
        // WARNING overlooks promotions 
        self.pst_mg[opp] += get_mg_weight(((ctx.m >> 6) & 0b111111) as usize, opp, ctx.moved_pc as usize);
        ctx
    }

    /// Unmaking move, updates engine's scores as well
    pub fn unmake_move(&mut self, ctx: MoveContext) {
        if ctx.captured_pc != 7 {
            self.material_scores[self.board.side_to_move] += PIECE_VALS[ctx.captured_pc as usize];
            self.pst_mg[self.board.side_to_move] += get_mg_weight(((ctx.m >> 6) & 0b111111) as usize, self.board.side_to_move, ctx.captured_pc as usize);
        }
        let opp = self.board.side_to_move ^ 1;
        self.pst_mg[opp] += get_mg_weight((ctx.m & 0b111111) as usize, opp, ctx.moved_pc as usize);
        self.pst_mg[opp] -= get_mg_weight(((ctx.m >> 6) & 0b111111) as usize, opp, ctx.moved_pc as usize);
        self.board.unmake_move(ctx);
    }
}
