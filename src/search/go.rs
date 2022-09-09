use super::*;
use crate::io::outputs::uci_info;
use kimbo_state::MoveType;
use std::sync::atomic::Ordering;
use std::time::Instant;

impl Search {
    /// iterative deepening search
    pub fn go<const TEST: bool>(&mut self) -> u16 {
        let moves = self.position.board.gen_moves::<{ MoveType::ALL }>();
        // creating the initial scored move list with all scores set to 0
        let mut move_list: Vec<(u16, i16)> = Vec::with_capacity(64);
        for m in moves {
            move_list.push((m, 0));
        }
        // loop of iterative deepening, up to preset max depth
        self.stats.start_time = Instant::now();
        for d in 0..self.max_depth {
            let mut pv = Vec::new();
            let score = self.negamax(-MAX, MAX, d + 1, 1, &mut pv);

            if self.stop.load(Ordering::Relaxed) || self.stats.node_count > self.max_nodes {
                break;
            }
            self.best_move = pv[0];
            uci_info(
                d + 1,
                self.stats.seldepth,
                self.stats.node_count,
                self.stats.start_time.elapsed().as_millis(),
                pv,
                score,
                self.ttable.filled.load(Ordering::Relaxed),
                self.ttable.num_entries as u64,
            );

            if score == MAX || score == -MAX {
                break;
            }
        }
        if TEST {
            self.stats.report();
        }
        // resetting counts
        self.stats.reset();
        self.age += 1;
        self.best_move
    }
}
