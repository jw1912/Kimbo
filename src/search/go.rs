use super::*;
use crate::io::SearchStats;
use crate::io::outputs::uci_info;
use std::sync::atomic::Ordering;
use std::time::Instant;

impl Engine {
    /// iterative deepening search
    /// CLI: command line output of info needed?
    /// STATS: debug stats needed?
    pub fn go<const CLI: bool, const STATS: bool>(&mut self) -> u16 {
        // loop of iterative deepening, up to preset max depth
        self.stats.start_time = Instant::now();
        let mut stats = SearchStats::new(0, 0, 0, Vec::new());
        let mut best_move = 0;
        let mut prev_m = 0;
        if !self.board.state_stack.is_empty() {
            prev_m = self.board.state_stack.last().unwrap().m;
        }
        for d in 0..self.max_depth {
            self.stats.seldepth = 0;
            let mut pv = Vec::new();
            let check = self.board.is_in_check();
            let score = self.negamax::<true, true, STATS>(-MAX_SCORE, MAX_SCORE, d + 1, 0, &mut pv, prev_m, check, false);

            if self.stop.load(Ordering::Relaxed) || self.stats.node_count > self.max_nodes {
                break;
            }
            if !pv.is_empty() {
                best_move = pv[0];
            }
            let time = self.stats.start_time.elapsed().as_millis();
            if STATS { 
                stats = SearchStats::new(d + 1, time as u64, self.stats.node_count, pv.clone())
            }
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
        if STATS {
            stats.report();
            self.stats.report();
        }
        // resetting counts
        self.stats.reset();
        best_move
    }
}
