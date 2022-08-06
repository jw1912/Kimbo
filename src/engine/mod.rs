mod eval;
mod qsearch;
mod search;
#[rustfmt::skip]
mod pst;

use eval::*;
use kimbo_state::*;
use pst::*;

/// The extended position used by the engine.
pub struct EnginePosition {
    /// Basic position, for move generation and making moves
    pub board: Position,
    /// Incrementally updated material scores for each side
    pub material_scores: [i16; 2],
    /// Midgame piece-square table scores
    pub pst_mg: [i16; 2],
    /// Endgame piece-square table scores
    pub pst_eg: [i16; 2],
    /// heuristic for game phase
    pub phase: i16,
}
impl Default for EnginePosition {
    fn default() -> Self {
        Self::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
    }
}

/// Extended move context for incrementally updated eval fields
pub struct EngineMoveContext {
    ctx: MoveContext,
    scores: [[i16; 2]; 3],
    phase: i16,
}

impl EnginePosition {
    /// Initialise a new position from a fen string
    pub fn from_fen(s: &str) -> Self {
        let board = Position::from_fen(s);
        let material_scores = calculate_material_scores(&board);
        let pst_mg = calculate_pst_mg_scores(&board);
        let pst_eg = calculate_pst_eg_scores(&board);
        let phase = calculate_phase(&board);
        Self {
            board,
            material_scores,
            pst_mg,
            pst_eg,
            phase,
        }
    }

    /// Making move, updates engine's scores as well
    pub fn make_move(&mut self, m: u16) -> EngineMoveContext {
        let ctx = self.board.make_move(m);
        let ext_ctx = EngineMoveContext {
            ctx,
            scores: [self.material_scores, self.pst_mg, self.pst_eg],
            phase: self.phase,
        };
        let from_idx = (ctx.m & 63) as usize;
        let to_idx = ((ctx.m >> 6) & 63) as usize;
        let flag = ctx.m & MoveFlags::ALL;
        let moved_pc = ctx.moved_pc as usize;
        // side to move after this
        let opp = self.board.side_to_move;
        // side that just moved above
        let side = opp ^ 1;
        // removing original pst bonus
        self.pst_mg[side] -= get_mg_weight(from_idx, side, moved_pc);
        self.pst_eg[side] -= get_eg_weight(from_idx, side, moved_pc);
        match flag {
            MoveFlags::QUIET => {
                self.pst_mg[side] += get_mg_weight(to_idx, side, moved_pc);
                self.pst_eg[side] += get_eg_weight(to_idx, side, moved_pc);
            }
            MoveFlags::CAPTURE => {
                self.pst_mg[side] += get_mg_weight(to_idx, side, moved_pc);
                self.pst_eg[side] += get_eg_weight(to_idx, side, moved_pc);
                // updated captured piece psts
                let cap_pc = ctx.captured_pc as usize;
                self.material_scores[opp] -= PIECE_VALS[cap_pc];
                self.pst_mg[opp] -= get_mg_weight(to_idx, opp, cap_pc);
                self.pst_eg[opp] -= get_eg_weight(to_idx, opp, cap_pc);
                self.phase -= PHASE_VALS[cap_pc];
            }
            MoveFlags::EN_PASSANT => {
                self.pst_mg[side] += get_mg_weight(to_idx, side, moved_pc);
                self.pst_eg[side] += get_eg_weight(to_idx, side, moved_pc);
                let pwn_idx = match opp {
                    Side::WHITE => to_idx + 8,
                    Side::BLACK => to_idx - 8,
                    _ => panic!("Invalid side!"),
                };
                self.material_scores[opp] -= PIECE_VALS[0];
                self.pst_mg[opp] -= get_mg_weight(pwn_idx, opp, 0);
                self.pst_eg[opp] -= get_eg_weight(pwn_idx, opp, 0);
                self.phase -= PHASE_VALS[0];
            }
            MoveFlags::DBL_PUSH => {
                self.pst_mg[side] += get_mg_weight(to_idx, side, moved_pc);
                self.pst_eg[side] += get_eg_weight(to_idx, side, moved_pc);
            }
            MoveFlags::QS_CASTLE => {
                self.pst_mg[side] += get_mg_weight(to_idx, side, moved_pc);
                self.pst_eg[side] += get_eg_weight(to_idx, side, moved_pc);
                let (idx1, idx2) = match side {
                    Side::WHITE => (0, 3),
                    Side::BLACK => (56, 59),
                    _ => panic!("Invalid side!"),
                };
                self.pst_mg[side] -= get_mg_weight(idx1, side, 3);
                self.pst_eg[side] -= get_eg_weight(idx1, side, 3);
                self.pst_mg[side] += get_mg_weight(idx2, side, 3);
                self.pst_eg[side] += get_eg_weight(idx2, side, 3);
            }
            MoveFlags::KS_CASTLE => {
                self.pst_mg[side] += get_mg_weight(to_idx, side, moved_pc);
                self.pst_eg[side] += get_eg_weight(to_idx, side, moved_pc);
                let (idx1, idx2) = match side {
                    Side::WHITE => (7, 5),
                    Side::BLACK => (63, 61),
                    _ => panic!("Invalid side!"),
                };
                self.pst_mg[side] -= get_mg_weight(idx1, side, 3);
                self.pst_eg[side] -= get_eg_weight(idx1, side, 3);
                self.pst_mg[side] += get_mg_weight(idx2, side, 3);
                self.pst_eg[side] += get_eg_weight(idx2, side, 3);
            }
            _ => {
                if flag < MoveFlags::KNIGHT_PROMO_CAPTURE {
                    let promo_pc: usize = match flag {
                        MoveFlags::KNIGHT_PROMO => 1,
                        MoveFlags::BISHOP_PROMO => 2,
                        MoveFlags::ROOK_PROMO => 3,
                        MoveFlags::QUEEN_PROMO => 4,
                        _ => panic!("Invalid push promotion!"),
                    };
                    self.pst_mg[side] += get_mg_weight(to_idx, side, promo_pc);
                    self.pst_eg[side] += get_eg_weight(to_idx, side, promo_pc);
                    self.phase += PHASE_VALS[promo_pc];
                } else {
                    let promo_pc: usize = match flag {
                        MoveFlags::KNIGHT_PROMO_CAPTURE => 1,
                        MoveFlags::BISHOP_PROMO_CAPTURE => 2,
                        MoveFlags::ROOK_PROMO_CAPTURE => 3,
                        MoveFlags::QUEEN_PROMO_CAPTURE => 4,
                        _ => panic!("Invalid capture promotion!"),
                    };
                    self.pst_mg[side] += get_mg_weight(to_idx, side, promo_pc);
                    self.pst_eg[side] += get_eg_weight(to_idx, side, promo_pc);
                    let cap_pc = ctx.captured_pc as usize;
                    self.material_scores[opp] -= PIECE_VALS[cap_pc];
                    self.pst_mg[opp] -= get_mg_weight(to_idx, opp, cap_pc);
                    self.pst_eg[opp] -= get_eg_weight(to_idx, opp, cap_pc);
                    self.phase += PHASE_VALS[promo_pc];
                    self.phase -= PHASE_VALS[cap_pc];
                }
            }
        }
        ext_ctx
    }

    /// Unmaking move, updates engine's scores as well
    pub fn unmake_move(&mut self, ext_ctx: EngineMoveContext) {
        let ctx = ext_ctx.ctx;
        self.material_scores = ext_ctx.scores[0];
        self.pst_mg = ext_ctx.scores[1];
        self.pst_eg = ext_ctx.scores[2];
        self.phase = ext_ctx.phase;
        self.board.unmake_move(ctx);
    }
}
