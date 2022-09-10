use super::*;
use crate::engine::EngineMoveContext;
use kimbo_state::{MoveType, Check};

impl Search {
    /// Quiescence search
    pub fn quiesce<const STATS: bool>(&mut self, mut alpha: i16, beta: i16, ply: u8) -> i16 {
        let stand_pat = self.position.static_eval();

        // beta pruning
        // there is an argument for returning stand pat instead of beta
        if stand_pat >= beta {
            self.stats.node_count += 1;
            if STATS { self.stats.qnode_count += 1; }
            return beta;
        }
        // delta pruning
        // queen worth
        if stand_pat < alpha - 850 {
            self.stats.node_count += 1;
            if STATS { self.stats.qnode_count += 1; }
            return alpha;
        }
        // improving alpha bound
        if alpha < stand_pat {
            alpha = stand_pat;
        }

        // generating and sorting captures
        let mut _king_checked = Check::None;
        let mut captures = self.position.board.gen_moves::<{ MoveType::CAPTURES }>(&mut _king_checked);
        //let king_in_check = _king_checked != Check::None;
        //if king_in_check {
        //    captures.append(&mut self.position.board.gen_moves::<{ MoveType::QUIETS }>(&mut _king_checked))
        //}

        captures.sort_by_key(|m| self.position.mvv_lva(m));

        // going through captures
        let mut ctx: EngineMoveContext;
        let mut score: i16;
        for m in captures {
            // making move, getting score, unmaking move
            ctx = self.position.make_move(m);
            //if king_in_check {
            //    score = -self.negamax::<false, false>(-beta, -alpha, 1, ply + 1, &mut Vec::new())
            //} else {
            //    score = -self.quiesce(-beta, -alpha, ply + 1);
            //}
            score = -self.quiesce::<STATS>(-beta, -alpha, ply + 1);
            self.position.unmake_move(ctx);

            // beta pruning
            if score >= beta {
                self.stats.node_count += 1;
                if STATS { self.stats.qnode_count += 1; }
                return beta;
            }
            // improve alpha bound
            if score > alpha {
                alpha = score
            }
        }
        self.stats.node_count += 1;
        if STATS { self.stats.qnode_count += 1; }
        alpha
    }
}
