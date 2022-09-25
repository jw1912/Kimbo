use crate::tables::search::{Bound, HashResult};
use super::{Engine, is_mate_score};

const LMR_MIN_IDX: usize = 2;
const LMR_MAX_SCORE: i16 = 300;
const LMR_MIN_DEPTH: i8 = 2;

const NMP_MIN_PHASE: i16 = 6;
const NMP_MIN_DEPTH: i8 = 3;

const RFP_MAX_DEPTH: i8 = 8;
pub const RFP_MARGIN_PER_DEPTH: i16 = 120;

const RAZOR_MAX_DEPTH: i8 = 4;
pub const RAZOR_MARGIN_PER_DEPTH: i16 = 120;

const HLP_MAX_DEPTH: i8 = 4;
const HLP_MAX_SCORE: i16 = 0;

/// Based on a hash result and given search parameters
/// returns Some(value) if pruning is appropriate, else None
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
    /// can we safely do late move reductions?
    /// 
    /// source: https://www.chessprogramming.org/Late_Move_Reductions
    /// 
    /// cannot reduce:
    ///  - root moves
    ///  - when friendly king is in check before the move,
    ///  - the first [LMR_MIN_IDX] moves,
    ///  - moves sorted with score higher than [LMR_MAX_SCORE],
    ///  - depth <= [LMR_MIN_DEPTH],
    ///  - moves which cause check
    pub fn can_do_lmr<const ROOT: bool>(&self, king_in_check: bool, depth: i8, m_idx: usize, m_score: i16, check: bool) -> bool {
        !ROOT
        && !king_in_check
        && m_idx >= LMR_MIN_IDX
        && m_score <= LMR_MAX_SCORE
        && depth >= LMR_MIN_DEPTH
        && !check
    }

    /// can we safely do null move pruning?
    /// 
    /// source: https://www.chessprogramming.org/Null_Move_Pruning
    /// 
    /// cannot prune:
    ///  - in the endgame
    ///  - phase > [NMP_MIN_PHASE]
    ///  - depth < [NMP_MIN_DEPTH]
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
/// 
/// cannot prune:
///  - nodes not near frontier
///  - when beta is near mate score
pub fn can_do_rfp(depth: i8, beta: i16) -> bool {
    depth <= RFP_MAX_DEPTH && !is_mate_score(beta)
}

/// can we safely do razoring?
/// 
/// source: 
/// 
/// cannot prune: 
///  - nodes not near frontier
///  - when alpha is a mate score
pub fn can_razor(depth: i8, alpha: i16) -> bool {
    depth <= RAZOR_MAX_DEPTH && !is_mate_score(alpha)
}

/// can we safely do history leaf pruning?
/// 
/// source: 
/// 
/// cannot prune:
///  - PV nodes
///  - when in check or giving check
///  - non-quiet, highest sorted moves
pub fn can_do_hlp<const PV: bool>(king_in_check: bool, depth: i8, m_idx: usize, m_score: i16, check: bool) -> bool {
    !PV
    && !king_in_check
    && m_idx >= 2 + depth as usize * 4
    && depth <= HLP_MAX_DEPTH
    && m_score <= HLP_MAX_SCORE
    && !check
}