use std::sync::{Arc, atomic::AtomicBool};
use crate::{
    state::Position,
    tables::HashTable,
    engine::{
        consts::*,
        util::{PvLine, u16_to_uci},
        search::search,
    }
};

pub mod consts;
pub mod util;
mod eval;
mod limits;
mod qsearch;
mod search;

use limits::Limits;

pub struct Engine {
    pub position: Position,
    pub limits: Limits,
    pub hash_table: Box<HashTable>,
    ply: i16,
    nodes: u64,
    qnodes: u64,
}

impl Engine {
    pub fn new(abort_signal: Arc<AtomicBool>) -> Self {
        Self {
            position: Position::default(),
            limits: Limits::new(abort_signal),
            hash_table: {
                let mut table = HashTable::default();
                table.resize(1);
                Box::new(table)
            },
            ply: 0,
            nodes: 0,
            qnodes: 0
        }
    }

    pub fn go(&mut self) {
        self.limits.reset();
        let in_check = self.position.in_check();
        let mut best_move = 0;

        for depth in 1..=self.limits.depth() {
            let mut pv_line = PvLine::with_capacity(depth);

            let score = search(self, -Score::MAX, Score::MAX, depth, in_check, &mut pv_line);

            // stop searching if time up
            if self.limits.aborting() {
                break;
            }

            // update best move
            best_move = pv_line.first();

            // UCI output
            let pv_str = pv_line.to_string(&self.position);
            let time = self.limits.elapsed();
            let nodes = self.nodes + self.qnodes;
            let (score_type, score_value) = if score.abs() > Score::MATE {
                (
                    "mate",
                    if score < 0 {
                        score.abs() - Score::MAX
                    } else {
                        Score::MAX - score + 1
                    } / 2,
                )
            } else {
                ("cp", score)
            };
            println!(
                "info depth {depth} score {score_type} {score_value} time {} nodes {nodes} nps {:.0} hashfull {} pv {pv_str}",
                time.as_millis(),
                nodes as f64 / time.as_secs_f64(),
                1000 * self.hash_table.filled() / self.hash_table.capacity()
            );
        }

        println!("bestmove {}", u16_to_uci(&self.position, best_move));
    }
}