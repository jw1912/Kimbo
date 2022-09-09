pub mod transposition;
pub mod go;

use crate::engine::EnginePosition;
use transposition::PerftTT;
use std::sync::Arc;

/// Search info
pub struct PerftSearch {
    /// Position to be searched
    pub position: EnginePosition,
    /// Transposition table
    pub ttable: Arc<PerftTT>,
    /// Search stats
    pub stats: PerftStats,
}

/// Search statistics
#[derive(Default)]
pub struct PerftStats {
    pub tt_hits: u64,
}

impl PerftSearch {
    /// Makes a new search instance
    pub fn new(
        position: EnginePosition,
        ttable: Arc<PerftTT>,
    ) -> Self {
        let stats = PerftStats::default();
        PerftSearch {
            position,
            ttable,
            stats,
        }
    }
}

impl PerftStats {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn report(&self) {
        println!("tt hits: {}", self.tt_hits);
    }
}