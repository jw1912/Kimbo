use crate::tables::search::{Bound, HashResult};
use super::is_mate_score;

const LMR_MIN_DEPTH: i8 = 2;
const LMR_MIN_IDX: usize = 2;
const LMR_MAX_SCORE: i16 = 300;

const NMP_MIN_PHASE: i16 = 6;
const NMP_MIN_DEPTH: i8 = 3;

const RFP_MAX_DEPTH: i8 = 8;
pub const RFP_MARGIN_PER_DEPTH: i16 = 120;

/// can we safely prune based off hash score?
#[inline]
pub fn tt_prune<const PV: bool>(res: &HashResult, depth: i8, alpha: i16, beta: i16, halfmove_clock: u8) -> Option<i16> {
    if res.depth >= depth && halfmove_clock <= 90 {
        match res.bound {
            Bound::EXACT => {
                if !PV {
                    return Some(res.score);
                }
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

/// can we safely try pruning?
#[inline]
pub fn can_do_pruning<const PV: bool>(king_in_check: bool, beta: i16) -> bool {
    !PV
    && !king_in_check
    && !is_mate_score(beta)
}

/// can we safely do null move pruning?
#[inline]
pub fn can_do_nmp(allow_null: bool, phase: i16, depth: i8, beta: i16, lazy_eval: i16) -> bool {
    allow_null
    && phase >= NMP_MIN_PHASE
    && depth >= NMP_MIN_DEPTH
    && lazy_eval >= beta
}

/// can we safely do reverse futility pruning?
#[inline]
pub fn can_do_rfp(depth: i8, beta: i16, lazy_eval: i16) -> bool {
    depth <= RFP_MAX_DEPTH
    && lazy_eval >= beta + RFP_MARGIN_PER_DEPTH * depth as i16
}

/// can we safely do late move reductions?
#[inline]
pub fn can_do_lmr<const ROOT: bool>(king_in_check: bool, m_idx: usize, m_score: i16, check: bool, depth: i8) -> bool {
    !ROOT
    && depth >= LMR_MIN_DEPTH
    && !king_in_check
    && m_idx >= LMR_MIN_IDX
    && m_score <= LMR_MAX_SCORE
    && !check
}
