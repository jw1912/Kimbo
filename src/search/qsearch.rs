use super::*;
use crate::engine::EngineMoveContext;
use kimbo_state::MoveType;

impl Search {
    /// Quiescence search
    pub fn quiesce(&mut self, mut alpha: i16, beta: i16) -> i16 {
        let stand_pat = self.position.static_eval();
        let mut captures = self.position.board.gen_moves::<{ MoveType::CAPTURES }>();
        // checking for mate
        if captures.is_empty() {
            // COULD test for checkmate here, but probably not worth it
            self.node_count += 1;
            return stand_pat;
        }
        // beta pruning
        // there is an argument for returning stand pat instead of beta
        if stand_pat >= beta {
            self.node_count += 1;
            return beta;
        }
        // delta pruning
        // queen worth
        if stand_pat < alpha - 850 {
            self.node_count += 1;
            return alpha;
        }
        // improving alpha bound
        if alpha < stand_pat {
            alpha = stand_pat;
        }
        captures.sort_by_key(|m| self.position.mvv_lva(m));
        // going through captures
        let mut ctx: EngineMoveContext;
        let mut score: i16;
        for m in captures {
            ctx = self.position.make_move(m);
            score = -self.quiesce(-beta, -alpha);
            self.position.unmake_move(ctx);
            // beta pruning
            if score >= beta {
                self.node_count += 1;
                return beta;
            }
            // improve alpha bound
            if score > alpha {
                alpha = score
            }
        }
        self.node_count += 1;
        alpha
    }
}
