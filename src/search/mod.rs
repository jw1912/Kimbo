use std::{sync::{atomic::AtomicBool, Arc}, time::Instant};
use crate::engine::transposition::TT;

use super::engine::EnginePosition;
mod negamax;
mod qsearch;
mod timings;
mod go;

/// maximal score (for mate)
pub const MAX: i16 = 30000;

/// Move timing info
#[derive(Default, PartialEq, Eq)]
pub struct Times {
    /// White time on clock
    pub wtime: u64,
    /// Black time on clock
    pub btime: u64,
    /// White time increment
    pub winc: u64,
    /// Black time increment
    pub binc: u64,
    /// Moves until next time control
    pub moves_to_go: Option<u8>
}

/// Search info
pub struct Search {
    /// Position to be searched
    pub position: EnginePosition,
    /// Token to say if search needs to be stopped
    pub stop: Arc<AtomicBool>,
    /// Best move found
    pub best_move: u16,
    /// Force time limit
    pub max_move_time: u64,
    /// Forced depth
    pub max_depth: u8,
    /// Forced nodes
    pub max_nodes: u64,
    /// Transposition table
    pub ttable: Arc<TT>,
    /// number of searches run, for overwriting the tt
    pub age: u8,
    /// Search stats
    pub stats: Stats,
}

/// Search statistics
pub struct Stats {
    /// Total nodes searched
    pub node_count: u64,
    /// Node count at previous depth (to get count at each depth)
    pub old_count: u64,
    /// Time at start
    pub start_time: Instant,
    /// Elapsed time at previous depth
    pub old_time: u64,
    pub tt_hits: (u64, u64, u64),
    pub collisions: u64,
}
impl Stats {
    fn reset(&mut self) {
        *self = Stats {
            node_count: 0,
            old_count: 0,
            start_time: Instant::now(),
            old_time: 0,
            tt_hits: (0, 0, 0),
            collisions: 0,
        };
    }
    fn report(&self) {
        let time = self.start_time.elapsed().as_millis();
        println!("total nodes: {}", self.node_count);
        println!("total time: {}ms", time);
        println!("total nps: {}", self.node_count * 1000 / time as u64);
        println!("total tt hits: {}", self.tt_hits.0);
        println!("total collisions: {}", self.collisions);
    }
}

impl Search {
    /// Makes a new search instance
    pub fn new(position: EnginePosition, stop: Arc<AtomicBool>, max_move_time: u64, max_depth: u8, max_nodes: u64, ttable: Arc<TT>, age: u8) -> Self {
        let stats = Stats {
            node_count: 0,
            old_count: 0,
            start_time: Instant::now(),
            old_time: 0,
            tt_hits: (0, 0, 0),
            collisions: 0,
        };
        Search { 
            position, 
            stop, 

            best_move: 0, 
            max_move_time, 
            max_depth, 
            max_nodes,
            ttable,
            age,
            stats
        }
    }
}