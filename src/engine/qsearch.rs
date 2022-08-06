use super::*;

// atomic count of quiescence calls
use std::sync::atomic::{AtomicUsize, Ordering};
pub static QS_CALLS: AtomicUsize = AtomicUsize::new(0);
pub fn count_qs_plus() {
    QS_CALLS.fetch_add(1, Ordering::SeqCst);
}

impl EnginePosition {
    /// Quiescence search
    pub fn quiesce(&mut self, mut alpha: i16, beta: i16, max_depth: u8) -> i16 {
        let stand_pat = self.static_eval();
        if max_depth == 0 {
            count_qs_plus();
            return stand_pat;
        }
        let mut captures = self.board.gen_moves::<{ MoveType::CAPTURES }>();
        // checking for mate
        if captures.is_empty() {
            count_qs_plus();
            let quiets = self.board.gen_moves::<{ MoveType::QUIETS }>();
            if quiets.is_empty() {
                let side = self.board.side_to_move;
                let idx = ls1b_scan(self.board.pieces[side][5]) as usize;
                // checkmate
                if self
                    .board
                    .is_square_attacked(idx, side, self.board.occupied)
                {
                    return -MAX;
                }
                // stalemate
                return 0;
            }
            return stand_pat;
        }
        // beta pruning
        // there is an argument for returning stand pat instead of beta
        if stand_pat >= beta {
            count_qs_plus();
            return beta;
        }
        // delta pruning
        // queen worth
        if stand_pat < alpha - 900 {
            count_qs_plus();
            return alpha;
        }
        // improving alpha bound
        if alpha < stand_pat {
            alpha = stand_pat;
        }
        captures.sort_unstable_by_key(|m| self.mvv_lva(m));
        // going through captures
        let mut ctx: EngineMoveContext;
        let mut score: i16;
        for m in captures {
            ctx = self.make_move(m);
            score = -self.quiesce(-beta, -alpha, max_depth - 1);
            self.unmake_move(ctx);
            // beta pruning
            if score >= beta {
                count_qs_plus();
                return beta;
            }
            // improve alpha bound
            if score > alpha {
                alpha = score
            }
        }
        count_qs_plus();
        alpha
    }
}
