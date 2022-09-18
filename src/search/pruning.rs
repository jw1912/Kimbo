use crate::tables::search::{Bound, HashResult};
use super::Engine;

const LMR_MIN_IDX: usize = 2;
const LMR_MAX_SCORE: i16 = 300;
const LMR_MIN_DEPTH: i8 = 2;

/// Based on a hash result and given search parameters
/// returns Some(value) if pruning is appropriate, else None
pub fn tt_prune(res: &HashResult, depth: i8, alpha: i16, beta: i16) -> Option<i16> {
    if res.depth > depth - (res.bound == Bound::EXACT) as i8 {
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
    /// SOURCE: https://www.chessprogramming.org/Late_Move_Reductions
    /// 
    /// cannot reduce:
    ///  - root moves,
    ///  - extended moves (if in check before making the move),
    ///  - the first [LMR_MIN_IDX] moves,
    ///  - moves sorted with score higher than [LMR_MAX_SCORE],
    ///  - depth >= [LMR_MIN_DEPTH],
    ///  - moves which cause check
    pub fn can_do_lmr<const ROOT: bool>(&self, ext: i8, depth: i8, m_idx: usize, m_score: i16, check: bool) -> bool {
        !ROOT
        && ext == 0
        && m_idx >= LMR_MIN_IDX
        && m_score <= LMR_MAX_SCORE
        && depth >= LMR_MIN_DEPTH
        && !check
    }
}
