use crate::hash::search::{Bound, HashResult};

/// Based on a hash result and given search parameters
/// returns Some(value) if pruning is appropriate, else None
pub fn tt_prune(res: HashResult, depth: u8, mut alpha: i16, mut beta: i16) -> Option<i16> {
    if res.depth >= depth - (res.bound == Bound::EXACT) as u8 {
        match res.bound {
            Bound::EXACT => {
                return Some(res.score);
            },
            Bound::LOWER => {
                if res.score > alpha {
                    alpha = res.score;
                }
            },
            Bound::UPPER => {
                if res.score < beta {
                    beta = res.score;
                }
            },
            _ => ()
        }
        if alpha >= beta {
            return Some(alpha)
        }
    }
    None
}
