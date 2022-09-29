use crate::tables::search::{Bound, HashResult};
use super::{Engine, is_mate_score};

const LMR_MIN_IDX: usize = 2;
const LMR_MAX_SCORE: i16 = 300;

const NMP_MIN_PHASE: i16 = 6;
const NMP_MIN_DEPTH: i8 = 3;

const RFP_MAX_DEPTH: i8 = 8;
pub const RFP_MARGIN_PER_DEPTH: i16 = 120;

const RAZOR_MAX_DEPTH: i8 = 4;
pub const RAZOR_MARGIN_PER_DEPTH: i16 = 240;

/// can we safely prune based off hash score?
pub fn tt_prune(res: &HashResult, depth: i8, alpha: i16, beta: i16, halfmove_clock: u8) -> Option<i16> {
    if res.depth > depth - (res.bound == Bound::EXACT) as i8 && halfmove_clock <= 80 {
        match res.bound {
            Bound::EXACT => {
                return Some(res.score);
            },
            Bound::LOWER => {
                if res.score >= beta {
                    return Some(beta);
                }
            },
            Bound::UPPER => {
                if res.score <= alpha {
                    return Some(alpha);
                }
            },
            _ => ()
        }
    }
    None
}

impl Engine {
    /// can we safely do null move pruning?
    /// 
    /// source: https://www.chessprogramming.org/Null_Move_Pruning
    pub fn can_do_nmp(&self, allow_null: bool, depth: i8, beta: i16) -> bool {
        self.board.phase >= NMP_MIN_PHASE
        && depth >= NMP_MIN_DEPTH
        && allow_null
        && !is_mate_score(beta)
    }
}

/// can we safely do reverse futility pruning?
/// 
/// source: https://www.chessprogramming.org/Reverse_Futility_Pruning
pub fn can_do_rfp(depth: i8, beta: i16) -> bool {
    depth <= RFP_MAX_DEPTH && !is_mate_score(beta)
}

/// can we safely do late move reductions?
/// 
/// source: https://www.chessprogramming.org/Late_Move_Reductions
pub fn can_do_lmr<const ROOT: bool>(king_in_check: bool, m_idx: usize, m_score: i16, check: bool) -> bool {
    !ROOT
    && !king_in_check
    && m_idx >= LMR_MIN_IDX
    && m_score <= LMR_MAX_SCORE
    && !check
}

/// can we safely do razoring?
/// 
/// source: 
pub fn can_razor(depth: i8, alpha: i16) -> bool {
    depth <= RAZOR_MAX_DEPTH && !is_mate_score(alpha)
}
