use crate::hash::{search::HashTable, pawn::PawnHashTable};
use crate::engine::EnginePosition;
use std::{
    sync::{atomic::AtomicBool, Arc},
    time::Instant,
};

pub mod timings;
mod go;
#[rustfmt::skip]
mod negamax;
mod qsearch;
mod sorting;
mod pruning;

/// Checkmate stuff
pub const MAX_SCORE: i16 = 30000;
pub const MATE_THRESHOLD: i16 = MAX_SCORE - u8::MAX as i16;
#[inline(always)]
pub fn is_mate_score(score: i16) -> bool {
    score >= MATE_THRESHOLD || score <= -MATE_THRESHOLD
}

/// Search struct
pub struct Search {
    pub position: EnginePosition,
    searching_side: usize,
    stop: Arc<AtomicBool>,
    pub best_move: u16,
    max_move_time: u64,
    max_depth: u8,
    max_nodes: u64,
    ttable: Arc<HashTable>,
    ptable: Arc<PawnHashTable>,
    age: u8,
    stats: Stats,
}

impl Search {
    /// Makes a new search instance
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        position: EnginePosition,
        stop: Arc<AtomicBool>,
        max_move_time: u64,
        max_depth: u8,
        max_nodes: u64,
        ttable: Arc<HashTable>,
        ptable: Arc<PawnHashTable>,
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
            ptable,
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

/// Statistics within a single negamax search
pub struct Stats {
    /// Always tracked
    node_count: u64,
    start_time: Instant,
    seldepth: u8,
    // Debugging only
    qnode_count: u64,
    tt_hits: u64,
    tt_move_hits: u64,
    tt_prunes: u64,
    pub pwn_hits: u64,
    pub pwn_misses: u64,
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
            pwn_hits: 0,
            pwn_misses: 0,
        } 
    }
}

impl Stats {
    fn reset(&mut self) {
        *self = Self::default();
    }
    fn report(&self) {
        let valid = self.tt_move_hits * 100 / (self.tt_hits - self.tt_prunes);
        println!("{}% of nodes are quiescence nodes", self.qnode_count * 100 / self.node_count);
        println!("hash hits: {} ({}% valid moves)", self.tt_hits, valid);
        println!("{}% of tt hits pruned", self.tt_prunes * 100 / self.tt_hits);
        println!("{}% pawn hash table hit rate", self.pwn_hits * 100 / (self.pwn_hits + self.pwn_misses));
    }
}



// useful functions
fn update_pv(pv: &mut Vec<u16>, m: u16, sub_pv: &mut Vec<u16>) {
    pv.clear();
    pv.push(m);
    pv.append(sub_pv); 
}

fn is_capture(m: u16) -> bool {
    m & 0b0100_0000_0000_0000 > 0
}

fn is_promotion(m: u16) -> bool {
    m & 0b1000_0000_0000_0000 > 0
}

fn is_castling(m: u16) -> bool {
    let flags = m & 0b1111_0000_0000_0000;
    flags == 0b0011_0000_0000_0000 || flags == 0b0010_0000_0000_0000
}
