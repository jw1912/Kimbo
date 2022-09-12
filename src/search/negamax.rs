use super::*;
use crate::engine::EngineMoveContext;
use crate::hash::search::CutoffType;
use kimbo_state::{MoveType, Check, movelist::MoveList};
use std::sync::atomic::Ordering;

impl Search {
    /// returns the evaluation of a position to a given depth
    pub fn negamax<const STATS: bool>(
        &mut self,
        mut alpha: i16,
        beta: i16,
        depth: u8,
        ply: u8,
        pv: &mut Vec<u16>,
    ) -> i16 {
        // if stop token, abort
        if self.stop.load(Ordering::Relaxed) {
            return 0; // immediately bow out of search
        }

        // check if nodes or time limits reached
        if self.stats.node_count > self.max_nodes
            || self.stats.start_time.elapsed().as_millis() as u64 > self.max_move_time
        {
            self.stop.store(true, Ordering::Relaxed);
            return 0;
        }

        // update seldepth (due to extensions)
        if ply > self.stats.seldepth {
            self.stats.seldepth = ply;
        }

        // depth 0 quiescence search
        if depth == 0 {
            let score = self.quiesce::<STATS>(alpha, beta, ply + 1);
            return score;
        }

        // now will be generating moves, so this node is counted as visited
        self.stats.node_count += 1;

        // probing hash table
        let orig_alpha = alpha;
        let zobrist = self.position.zobrist;
        let mut hash_move = 0;
        let mut entry_found = false;
        if let Some(res) = self.ttable.get(zobrist, ply, self.age) {
            if STATS { self.stats.tt_hits += 1; }
            hash_move = res.best_move;
            entry_found = true;
        }

        // generating moves
        let mut _king_checked = Check::None;
        let mut moves = MoveList::default();
        self.position.board.gen_moves::<{ MoveType::ALL }>(&mut _king_checked, &mut moves);
        let king_in_check = _king_checked != Check::None;

        // checking for checkmate/stalemate
        if moves.is_empty() {
            // checkmate
            if king_in_check {
                return -MAX_SCORE + ply as i16;
            }
            // stalemate
            return 0;
        }

        // CHECK EXTENSIONS
        let ext = if king_in_check {
            1
        } else {
            0
        };

        // move sorting
        let mut move_hit: bool = false;
        moves.sort(|m| self.position.score_move(m, hash_move, &mut move_hit));
        if STATS && move_hit {
            self.stats.tt_move_hits += 1;
        }

        // tracking best move information
        let mut best_move = 0;
        let mut best_score = -MAX_SCORE;

        // going through legal moves
        let mut ctx: EngineMoveContext;
        let mut score: i16;
        for m_idx in 0..moves.len() {
            let m = moves[m_idx];
            // new vector
            let mut sub_pv = Vec::new();
            // making move, getting score, unmaking move
            ctx = self.position.make_move(m);
            score = -self.negamax::<STATS>(-beta, -alpha, depth - 1 + ext, ply + 1, &mut sub_pv);
            self.position.unmake_move(ctx);
            // updating best move and score
            if score > best_score {
                best_score = score;
                best_move = m;  
            }

            // beta pruning
            if score >= beta {
                if STATS && m == hash_move {
                    self.stats.tt_beta_prunes += 1;
                }
                break;
            } 

            // improve alpha
            if score > alpha {
                alpha = score;
                if STATS && m == hash_move {
                    self.stats.tt_alpha_improvements += 1;
                }
                update_pv(pv, m, &mut sub_pv);
            }         
        }
        // writing to tt
        if !entry_found || alpha != orig_alpha {
            self.ttable
                .push(zobrist, best_score, best_move, depth + 1, ply, self.age, CutoffType::EXACT);
        }
        best_score
    }
}
