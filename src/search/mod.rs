use std::{sync::{atomic::AtomicBool, Arc}, time::Instant};
use super::engine::EnginePosition;
mod nsearch;
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
    pub moves_to_go: Option<u8>
}

/// Search info
pub struct Search {
    /// Position to be searched
    pub position: EnginePosition,
    /// Token to say if search needs to be stopped
    pub stop: Arc<AtomicBool>,
    /// Total nodes searched
    pub node_count: u64,
    /// Node count at previous depth (to get count at each depth)
    pub old_count: u64,
    /// Time at start
    pub start_time: Instant,
    /// Elapsed time at previous depth
    pub old_time: u64,
    /// Best move found
    pub best_move: u16,
    /// Force time limit
    pub max_move_time: u64,
    /// Forced depth
    pub max_depth: u8,
    /// Forced nodes
    pub max_nodes: u64,
}

impl Search {
    /// Makes a new search instance
    pub fn new(position: EnginePosition, stop: Arc<AtomicBool>, max_move_time: u64, max_depth: u8, max_nodes: u64) -> Self {
        Search { 
            position, 
            stop, 
            node_count: 0,
            old_count: 0,
            start_time: Instant::now(),
            old_time: 0,
            best_move: 0, 
            max_move_time, 
            max_depth, 
            max_nodes 
        }
    }
}