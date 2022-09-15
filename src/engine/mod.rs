mod eval;
#[rustfmt::skip]
mod consts;
mod moves;
pub mod zobrist;
pub mod perft;

use eval::*;
use crate::tables::{pawn::PawnHashTable, search::HashTable, countermove::CounterMoveTable};
use crate::io::errors::UciError;
use kimbo_state::{Position, MoveContext};
use std::time::Instant;
use std::sync::{Arc, atomic::AtomicBool};

use self::zobrist::{initialise_zobrist, ZobristVals, initialise_pawnhash};

/// The extended position used by the engine.
#[derive(Clone)]
pub struct Engine {
    /// Basic position, for move generation and making moves
    pub board: Position,
    /// Eval stuff
    mat_mg: [i16; 2],
    mat_eg: [i16; 2],
    pst_mg: [i16; 2],
    pst_eg: [i16; 2],
    phase: i16,
    /// Hashing
    pub zobrist: u64,
    pub pawnhash: u64,
    /// Heap allocated stuff
    pub zobrist_vals: Arc<ZobristVals>,
    pub ttable: Arc<HashTable>,
    pub ptable: Arc<PawnHashTable>,
    pub ctable: Arc<CounterMoveTable>,
    // Search info
    pub stop: Arc<AtomicBool>,
    pub max_move_time: u64,
    pub max_depth: u8,
    pub max_nodes: u64,
    pub age: u8,
    pub stats: Stats,
}

/// Extended move context for incrementally updated eval fields
pub struct EngineMoveContext {
    pub ctx: MoveContext,
    mat_mg: [i16; 2],
    mat_eg: [i16; 2],
    pst_mg: [i16; 2],
    pst_eg: [i16; 2],
    phase: i16,
    zobrist: u64,
    pawnhash: u64
}

impl Engine {
    /// Initialise a new position from a fen string
    #[allow(clippy::too_many_arguments)]
    pub fn from_fen(
        s: &str,
        ttable: Arc<HashTable>,
        ptable: Arc<PawnHashTable>,
    ) -> Result<Self, UciError> {
        let board = Position::from_fen(s)?;
        let mat_mg = calc_material::<true>(&board);
        let mat_eg = calc_material::<false>(&board);
        let pst_mg = calc_pst::<true>(&board);
        let pst_eg = calc_pst::<false>(&board);
        let phase = calculate_phase(&board);
        let zobrist_vals = Arc::new(ZobristVals::default());
        let zobrist = initialise_zobrist(&board, &zobrist_vals);
        let pawnhash = initialise_pawnhash(&board, &zobrist_vals);
        let stats = Stats::default();
        Ok(Self {
            board,
            mat_mg,
            mat_eg,
            pst_mg,
            pst_eg,
            phase,
            zobrist,
            zobrist_vals,
            pawnhash,
            stop: Arc::new(AtomicBool::new(false)),
            max_move_time: 0,
            max_depth: 0,
            max_nodes: 0,
            ttable,
            ptable,
            ctable: Arc::new(CounterMoveTable::default()),
            age: 0,
            stats,
        })
    }
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        board: Position,
        stop: Arc<AtomicBool>,
        max_move_time: u64,
        max_depth: u8,
        max_nodes: u64,
        ttable: Arc<HashTable>,
        ptable: Arc<PawnHashTable>,
        age: u8,
    ) -> Self {
        let mat_mg = calc_material::<true>(&board);
        let mat_eg = calc_material::<false>(&board);
        let pst_mg = calc_pst::<true>(&board);
        let pst_eg = calc_pst::<false>(&board);
        let phase = calculate_phase(&board);
        let zobrist_vals = Arc::new(ZobristVals::default());
        let zobrist = initialise_zobrist(&board, &zobrist_vals);
        let pawnhash = initialise_pawnhash(&board, &zobrist_vals);
        let stats = Stats::default();
        Self {
            board,
            mat_mg,
            mat_eg,
            pst_mg,
            pst_eg,
            phase,
            zobrist,
            zobrist_vals,
            pawnhash,
            stop,
            max_move_time,
            max_depth,
            max_nodes,
            ttable,
            ptable,
            ctable: Arc::new(CounterMoveTable::default()),
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
    pub seldepth: u8,
    // Debugging only
    pub qnode_count: u64,
    pub tt_hits: u64,
    pub tt_move_hits: u64,
    pub tt_prunes: u64,
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
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    pub fn report(&self) {
        let valid = self.tt_move_hits * 100 / (self.tt_hits - self.tt_prunes);
        println!("{}% of nodes are quiescence nodes", self.qnode_count * 100 / self.node_count);
        println!("hash hits: {} ({}% valid moves)", self.tt_hits, valid);
        println!("{}% of tt hits pruned", self.tt_prunes * 100 / self.tt_hits);
        println!("{}% pawn hash table hit rate", self.pwn_hits * 100 / (self.pwn_hits + self.pwn_misses));
    }
}
