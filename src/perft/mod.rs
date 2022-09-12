pub mod go;

use crate::engine::EnginePosition;
use crate::hash::perft::PerftTT;
use std::sync::Arc;

/// Search info
pub struct PerftSearch {
    /// Position to be searched
    position: EnginePosition,
    /// Transposition table
    pub ttable: Arc<PerftTT>,
    /// Search stats
    pub stats: PerftStats,
}

/// Search statistics
#[derive(Default)]
pub struct PerftStats {
    tt_hits: u64,
}

impl PerftSearch {
    /// Makes a new search instance
    pub fn new(position: EnginePosition, ttable: Arc<PerftTT>,) -> Self {
        let stats = PerftStats::default();
        PerftSearch {position, ttable, stats}
    }

    pub fn report(&self) {
        self.ttable.report();
        self.stats.report();
    }
}

impl PerftStats {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    fn report(&self) {
        println!("tt hits: {}", self.tt_hits);
    }
}