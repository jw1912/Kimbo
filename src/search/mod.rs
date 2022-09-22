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
    pub age: u8,
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
        Ok(Self::new(board, Arc::new(AtomicBool::new(false)), 0, 0, 0, ttable, ptable, 0))
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
        age: u8,
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
            age,
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
    // Debugging only
    pub qnode_count: u64,
    pub tt_hits: u64,
    pub tt_move_hits: u64,
    pub tt_prunes: u64,
    pub countermove_hits: u64,
    pub killermove_hits: u64,
    pub history_hits: u64,
    pub lmr_attempts: u64,
    pub lmr_successes: u64,
    pub nmp_attempts: u64,
    pub nmp_successes: u64,
    pub rfp_attempts: u64,
    pub rfp_successes: u64,
    pub razor_attempts: u64,
    pub razor_successes: u64,
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
            countermove_hits: 0,
            killermove_hits: 0,
            history_hits: 0,
            lmr_attempts: 0,
            lmr_successes: 0,
            nmp_attempts: 0,
            nmp_successes: 0,
            razor_attempts: 0,
            razor_successes: 0,
            rfp_attempts: 0,
            rfp_successes: 0,
        } 
    }
}

impl Stats {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    pub fn report(&self) {
        let valid = self.tt_move_hits as f64 * 100.0 / (self.tt_hits as f64 - self.tt_prunes as f64);
        let lmr = self.lmr_successes as f64 * 100.0 / (self.lmr_attempts as f64);
        println!("{}% of nodes are quiescence nodes", self.qnode_count * 100 / self.node_count);
        println!("hash hits: {} ({:.2}% valid moves)", self.tt_hits, valid);
        println!("{}% of tt hits pruned", self.tt_prunes * 100 / self.tt_hits);
        println!("counter move hits : {}", self.countermove_hits);
        println!("killer move hits : {}", self.killermove_hits);
        println!("history move hits : {}", self.history_hits);
        println!("lmr attempts: {}, successes: {} ({:.2}%)", self.lmr_attempts, self.lmr_successes, lmr);
        println!("nmp attempts: {}, successes: {}", self.nmp_attempts, self.nmp_successes);
        println!("rfp attempts: {}, successes: {}", self.rfp_attempts, self.rfp_successes);
        println!("razor attempts: {}, successes: {}", self.razor_attempts, self.razor_successes);
    }
}
