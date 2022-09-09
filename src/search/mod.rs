use crate::hash::search::TT;
use std::{
    sync::{atomic::AtomicBool, Arc},
    time::Instant,
};

use super::engine::EnginePosition;
mod go;
mod negamax;
mod qsearch;
mod timings;

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
    pub moves_to_go: Option<u8>,
}

/// Search info
pub struct Search {
    /// Position to be searched
    pub position: EnginePosition,
    /// Side searching for a move
    pub searching_side: usize,
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
    /// Total quiescent node count
    pub qnode_count: u64,
    /// Time at start
    pub start_time: Instant,
    pub tt_hits: u64,
    pub tt_move_hits: u64,
    pub cutoff_hits: u64,
    pub collisions: u64,
}
impl Default for Stats {
    fn default() -> Self {
        Self {
            node_count: 0,
            qnode_count: 0,
            start_time: Instant::now(),
            tt_hits: 0,
            tt_move_hits: 0,
            cutoff_hits: 0,
            collisions: 0,
        } 
    }
}

impl Stats {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    pub fn report(&self) {
        let time = self.start_time.elapsed().as_millis();
        println!("total nodes: {} ({}% quiescent)", self.node_count, self.qnode_count * 100 / self.node_count);
        println!("time: {}ms", time);
        println!("nps: {}", self.node_count * 1000 / (time + 1) as u64);
        println!("tt hits: {}", self.tt_hits);
        println!("tt move hits: {}", self.tt_move_hits);
        println!("beta cuttoff hits: {}", self.cutoff_hits);
        println!("collisions: {}", self.collisions);
    }
}

impl Search {
    /// Makes a new search instance
    pub fn new(
        position: EnginePosition,
        stop: Arc<AtomicBool>,
        max_move_time: u64,
        max_depth: u8,
        max_nodes: u64,
        ttable: Arc<TT>,
        age: u8,
    ) -> Self {
        let stats = Stats::default();
        let searching_side = position.board.side_to_move;
        Search {
            position,
            searching_side,
            stop,
            best_move: 0,
            max_move_time,
            max_depth,
            max_nodes,
            ttable,
            age,
            stats,
        }
    }
}
