use crate::hash::search::HashTable;
use crate::engine::EnginePosition;
use std::{
    sync::{atomic::AtomicBool, Arc},
    time::Instant,
};

mod go;
#[rustfmt::skip]
mod negamax;
mod qsearch;
mod timings;
mod sorting;
mod pruning;

/// Checkmate stuff
pub const MAX_SCORE: i16 = 30000;
pub const MATE_THRESHOLD: i16 = MAX_SCORE - u8::MAX as i16;
#[inline(always)]
pub fn is_mate_score(score: i16) -> bool {
    score >= MATE_THRESHOLD || score <= -MATE_THRESHOLD
}

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

/// Search struct
pub struct Search {
    /// Position to be searched
    pub position: EnginePosition,
    /// Side searching for a move
    searching_side: usize,
    /// Token to say if search needs to be stopped
    stop: Arc<AtomicBool>,
    /// Best move found
    pub best_move: u16,
    /// Force time limit
    max_move_time: u64,
    /// Forced depth
    max_depth: u8,
    /// Forced nodes
    max_nodes: u64,
    /// Transposition table
    ttable: Arc<HashTable>,
    /// number of searches run, for overwriting the tt
    age: u8,
    /// Search stats
    stats: Stats,
}

impl Search {
    /// Makes a new search instance
    pub fn new(
        position: EnginePosition,
        stop: Arc<AtomicBool>,
        max_move_time: u64,
        max_depth: u8,
        max_nodes: u64,
        ttable: Arc<HashTable>,
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

    #[inline(always)]
    fn search_limits_reached(&self) -> bool {
        self.stats.node_count > self.max_nodes // node count reached
        || self.stats.start_time.elapsed().as_millis() as u64 > self.max_move_time // search time exceeded
    }
}

/// Search statistics
struct Stats {
    /// Always tracked
    node_count: u64,
    start_time: Instant,
    seldepth: u8,
    // Debugging only
    qnode_count: u64,
    tt_hits: u64,
    tt_move_hits: u64,
    tt_prunes: u64,
}
impl Default for Stats {
    fn default() -> Self {
        Self {
            node_count: 0,
            seldepth: 0,
            start_time: Instant::now(),
            qnode_count: 0,
            tt_hits: 0,
            tt_move_hits: 0,
            tt_prunes: 0,
        } 
    }
}

impl Stats {
    fn reset(&mut self) {
        *self = Self::default();
    }
    fn report(&self) {
        let time = self.start_time.elapsed().as_millis();
        println!("total nodes: {} ({}% quiescent)", self.node_count, self.qnode_count * 100 / self.node_count);
        println!("time: {}ms", time);
        println!("nps: {}", self.node_count * 1000 / (time + 1) as u64);
        println!("hash hits: {} ({}% valid moves)", self.tt_hits, self.tt_move_hits * 100 / (self.tt_hits - self.tt_prunes));
        println!("{}% of tt hits pruned", self.tt_prunes * 100 / self.tt_hits);
    }
}

fn update_pv(pv: &mut Vec<u16>, m: u16, sub_pv: &mut Vec<u16>) {
    pv.clear();
    pv.push(m);
    pv.append(sub_pv); 
}
