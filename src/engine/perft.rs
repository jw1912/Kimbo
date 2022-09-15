use crate::{tables::perft::PerftTT, engine::Engine};
use std::sync::Arc;
use crate::engine::EngineMoveContext;
use crate::io::outputs::u16_to_uci;
use kimbo_state::{MoveType, MoveList};

/// Search info
pub struct PerftSearch {
    /// Position to be searched
    position: Engine,
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
impl PerftStats {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    fn report(&self) {
        println!("tt hits: {}", self.tt_hits);
    }
}

impl PerftSearch {
    /// Makes a new search instance
    pub fn new(position: Engine, ttable: Arc<PerftTT>,) -> Self {
        let stats = PerftStats::default();
        PerftSearch {position, ttable, stats}
    }

    pub fn report(&self) {
        self.ttable.report();
        self.stats.report();
    }

    pub fn perft<const TT_ACTIVE: bool>(&mut self, depth_left: u8) -> u64 {
        if depth_left == 0 {
            return 1;
        }
        // probing transposition table
        let zobrist = self.position.zobrist;
        if TT_ACTIVE {
            if let Some(res) = self.ttable.get(zobrist, depth_left) {
                self.stats.tt_hits += 1;
                return res;
            }
        }
        // bulk counting on depth 1
        let mut moves = MoveList::default();
        self.position.board.gen_moves::<{ MoveType::ALL }>(&mut kimbo_state::Check::None, &mut moves);
        if depth_left == 1 {
            return moves.len() as u64;
        }
        // calculate number of positions
        let mut positions: u64 = 0;
        let mut ctx: EngineMoveContext;
        for m_idx in 0..moves.len() {
            let m = moves[m_idx];
            ctx = self.position.make_move(m);
            positions += self.perft::<TT_ACTIVE>(depth_left - 1);
            self.position.unmake_move(ctx);
        }
        // push position info to tt
        if TT_ACTIVE {
            self.ttable.push(zobrist, positions, depth_left);
        }

        positions
    }

    pub fn go(&mut self, depth: u8) -> u64 {
        // works like stockfish's perft function
        if depth == 0 {
            return 1;
        }
        let mut new_move_list: Vec<(u16, u64)> = Vec::new();
        let mut moves = MoveList::default();
        self.position.board.gen_moves::<{ MoveType::ALL }>(&mut kimbo_state::Check::None, &mut moves);
        let mut ctx: EngineMoveContext;
        let mut score: u64;
        for m_idx in 0..moves.len() {
            let m = moves[m_idx];
            ctx = self.position.make_move(m);
            score = self.perft::<true>(depth - 1);
            self.position.unmake_move(ctx);
            new_move_list.push((m, score));
            println!("{}: {}", u16_to_uci(&m), score);
        }
        let mut positions: u64 = 0;
        for sm in new_move_list {
            positions += sm.1;
        }
        positions
    }
}