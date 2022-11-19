pub mod timings;
mod go;
#[rustfmt::skip]
mod negamax;
mod qsearch;
pub mod sorting;
mod pruning;

use crate::tables::history::HistoryTable;
use crate::tables::killer::KillerMoveTable;
use crate::tables::{pawn::PawnHashTable, search::HashTable, countermove::CounterMoveTable};
use crate::position::{Position, zobrist::ZobristVals};
use std::sync::{Arc, atomic::AtomicBool};
use std::time::Instant;
use crate::io::errors::UciError;

pub const MAX_PLY: i8 = i8::MAX;

/// Checkmate stuff
pub const MAX_SCORE: i16 = 30000;
pub const MATE_THRESHOLD: i16 = MAX_SCORE - u8::MAX as i16;
#[inline(always)]
pub fn is_mate_score(score: i16) -> bool {
    score >= MATE_THRESHOLD || score <= -MATE_THRESHOLD
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
    m & 0b1110_0000_0000_0000 == 0b0010_0000_0000_0000
}

#[derive(Clone)]
pub struct Engine {
    /// Basic position, for move generation and making moves
    pub board: Position,
    // tables
    pub ttable: Arc<HashTable>,
    pub ptable: Arc<PawnHashTable>,
    pub ctable: Arc<CounterMoveTable>,
    pub ktable: Arc<KillerMoveTable>,
    pub htable: Arc<HistoryTable>,
    // Search info
    pub stop: Arc<AtomicBool>,
    pub max_move_time: u64,
    pub max_depth: i8,
    pub max_nodes: u64,
    pub stats: Stats,
}

impl Engine {
    /// Initialise a new position from a fen string
    #[allow(clippy::too_many_arguments)]
    pub fn from_fen(
        s: &str,
        ttable: Arc<HashTable>,
        ptable: Arc<PawnHashTable>,
        zobrist_vals: Arc<ZobristVals>
    ) -> Result<Self, UciError> {
        let board = Position::from_fen(s, zobrist_vals)?;
        Ok(Self::new(board, Arc::new(AtomicBool::new(false)), 0, 0, 0, ttable, ptable))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        board: Position,
        stop: Arc<AtomicBool>,
        max_move_time: u64,
        max_depth: i8,
        max_nodes: u64,
        ttable: Arc<HashTable>,
        ptable: Arc<PawnHashTable>,
    ) -> Self {
        let stats = Stats::default();
        Self {
            board,
            stop,
            max_move_time,
            max_depth,
            max_nodes,
            ttable,
            ptable,
            ctable: Arc::new(CounterMoveTable::default()),
            ktable: Arc::new(KillerMoveTable::default()),
            htable: Arc::new(HistoryTable::default()),
            stats,
        }
    }

    #[inline(always)]
    pub fn search_limits_reached(&self) -> bool {
        self.stats.node_count > self.max_nodes // node count reached
        || self.stats.start_time.elapsed().as_millis() as u64 > self.max_move_time // search time exceeded
    }
}

/// Statistics within a single negamax search
#[derive(Clone)]
pub struct Stats {
    /// Always tracked
    pub node_count: u64,
    pub start_time: Instant,
    pub seldepth: i8,
}
impl Default for Stats {
    fn default() -> Self {
        Self {
            node_count: 0,
            seldepth: 0,
            start_time: Instant::now(),
        }
    }
}

impl Stats {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}
