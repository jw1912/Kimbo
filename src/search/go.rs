use super::*;
use crate::engine::sorting::is_score_near_mate;
use crate::io::outputs::uci_info;
use std::sync::atomic::Ordering;
use std::time::Instant;

impl Search {
    /// iterative deepening search
    pub fn go<const TEST: bool>(&mut self) -> u16 {
        // loop of iterative deepening, up to preset max depth
        self.stats.start_time = Instant::now();
        self.searching_side = self.position.board.side_to_move;
        for d in 0..self.max_depth {
            let mut pv = Vec::new();
            let score = self.negamax::<true>( -MAX, MAX, d + 1, 1, &mut pv);

            if self.stop.load(Ordering::Relaxed) || self.stats.node_count > self.max_nodes {
                break;
            }
            self.best_move = pv[0];
            uci_info(
                d + 1,
                self.stats.node_count,
                self.stats.start_time.elapsed().as_millis(),
                pv,
                score,
                self.ttable.filled.load(Ordering::Relaxed),
                self.ttable.num_entries as u64,
            );

            if is_score_near_mate(score) {
                break;
            }
        }
        if TEST {
            println!("{} / {} hash entries filled", self.ttable.filled.load(Ordering::Relaxed), self.ttable.num_entries);
            self.stats.report();
        }
        // resetting counts
        self.stats.reset();
        self.age += 1;
        self.best_move
    }
}
