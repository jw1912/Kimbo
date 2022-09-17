use crate::{tables::search::{Bound, HashResult}, engine::Engine};

/// Based on a hash result and given search parameters
/// returns Some(value) if pruning is appropriate, else None
pub fn tt_prune(res: HashResult, depth: u8, alpha: i16, beta: i16) -> Option<i16> {
    if res.depth > depth - (res.bound == Bound::EXACT) as u8 {
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

const LMR_MIN_IDX: usize = 2;
const LMR_MAX_SCORE: i16 = 300;
const LMR_MIN_DEPTH: u8 = 2;

impl Engine {
    /// can we safely do lmr?
    /// cannot reduce: (source: https://www.chessprogramming.org/Late_Move_Reductions)
    /// root moves
    /// extended moves (if in check before making the move)
    /// the first [LMR_MIN_IDX] moves
    /// moves sorted with score higher than [LMR_MAX_SCORE]
    /// moves near leaves - depth >= [LMR_MIN_DEPTH]
    /// moves which cause check
    pub fn can_do_lmr<const ROOT: bool>(&self, ext: u8, depth: u8, m_idx: usize, m_score: i16) -> bool {
        !ROOT
        && ext == 0
        && m_idx >= LMR_MIN_IDX
        && m_score <= LMR_MAX_SCORE
        && depth >= LMR_MIN_DEPTH
        && !self.is_in_check()
    }
}

