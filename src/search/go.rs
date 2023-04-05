use super::*;
use crate::io::outputs::uci_info;
use crate::position::{MoveList, MoveType};
use std::sync::atomic::Ordering;
use std::time::Instant;

impl Engine {
    /// iterative deepening search
    /// CLI: command line output of info needed?
    /// STATS: debug stats needed?
    pub fn go<const CLI: bool>(&mut self) -> u16 {
        // if only one legal move, make it immediately
        let mut moves = MoveList::default();
        self.board.gen_moves::<{ MoveType::ALL }>(&mut moves);
        if moves.len() == 1 {
            return moves[0];
        }

        // loop of iterative deepening, up to preset max depth
        self.stats.start_time = Instant::now();
        let mut best_move = 0;
        let mut prev_m = 0;
        if !self.board.state_stack.is_empty() {
            prev_m = self.board.state_stack.last().unwrap().m;
        }
        for d in 0..self.max_depth {
            self.stats.seldepth = 0;
            let mut pv = Vec::new();
            let check = self.board.is_in_check();
            let score = self.negamax::<true, true>(
                -MAX_SCORE,
                MAX_SCORE,
                d + 1,
                0,
                &mut pv,
                prev_m,
                check,
                false,
            );

            if self.stop.load(Ordering::Relaxed) || self.stats.node_count > self.max_nodes {
                break;
            }
            if !pv.is_empty() {
                best_move = pv[0];
            }
            let time = self.stats.start_time.elapsed().as_millis();
            if CLI {
                uci_info(
                    d + 1,
                    self.stats.seldepth,
                    self.stats.node_count,
                    time,
                    pv,
                    score,
                    self.ttable.hashfull(),
                );
            }

            if is_mate_score(score) {
                break;
            }
        }
        // resetting counts
        best_move
    }
}
