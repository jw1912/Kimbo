use super::*;
use super::pruning::tt_prune;
use crate::{engine::EngineMoveContext, hash::search::Bound};
use kimbo_state::{MoveType, Check, movelist::MoveList};
use std::sync::atomic::Ordering;
use std::cmp::{max, min};

impl Search {
    /// Main search
    /// ROOT: is this a root (ply = 0) node?
    /// STATS: are debug stats required?
    pub fn negamax<const ROOT: bool, const STATS: bool>(&mut self, mut alpha: i16, mut beta: i16, depth: u8, ply: u8, pv: &mut Vec<u16>,) -> i16 {
        // if stop token, abort
        if self.stop.load(Ordering::Relaxed) {
            return 0; // immediately bow out of search
        }

        // check if nodes or time limits reached
        if self.search_limits_reached() {
            self.stop.store(true, Ordering::Relaxed);
            return 0;
        }

        // update seldepth (due to extensions)
        if ply > self.stats.seldepth {
            self.stats.seldepth = ply;
        }

        // mate distance pruning
        alpha = max(alpha, -MAX_SCORE + ply as i16);
        beta = min(beta, MAX_SCORE - ply as i16);
        if alpha >= beta {
            return alpha
        }

        // depth 0 quiescence search
        if depth == 0 {
            return self.quiesce::<STATS>(alpha, beta, ply + 1);
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
            // hash score pruning (no pruning on root)
            if !ROOT {
                if let Some(score) = tt_prune(res, depth, alpha, beta) {
                    if STATS { self.stats.tt_prunes += 1 }
                    return score;
                }
            }
        }

        // generating moves
        let mut king_checked = Check::None;
        let mut moves = MoveList::default();
        self.position.board.gen_moves::<{ MoveType::ALL }>(&mut king_checked, &mut moves);
        let king_in_check = king_checked != Check::None;

        // checking for checkmate/stalemate
        if moves.is_empty() {
            return king_in_check as i16 * (-MAX_SCORE + ply as i16);
        }

        // check extensions
        let ext = king_in_check as u8;

        // move sorting
        let mut move_hit: bool = false;
        moves.sort(|m| self.position.score_move(m, hash_move, &mut move_hit));
        if STATS && move_hit {
            self.stats.tt_move_hits += 1;
        }

        // initialising stuff for going through move list
        let mut best_move = 0;
        let mut best_score = -MAX_SCORE;
        let mut ctx: EngineMoveContext;
        let mut score: i16;
        let mut bound: u8 = Bound::UPPER;

        // going through legal moves
        for m_idx in 0..moves.len() {
            let m = moves[m_idx];
            let mut sub_pv = Vec::new();

            // making move, getting score, unmaking move
            ctx = self.position.make_move(m);
            score = -self.negamax::<false, STATS>(-beta, -alpha, depth - 1 + ext, ply + 1, &mut sub_pv);
            self.position.unmake_move(ctx);

            // updating best move and score
            if score > best_score {
                best_score = score;
                best_move = m;
                // improve alpha
                if score > alpha {
                    alpha = score;
                    bound = Bound::EXACT;
                    update_pv(pv, m, &mut sub_pv);
                } 
            }

            // beta pruning
            if score >= beta {
                bound = Bound::LOWER;
                break;
            } 
        }
        // writing to tt
        if !entry_found || alpha != orig_alpha {
            self.ttable.push(zobrist, best_move, depth, self.age, bound, best_score, ply);
        }
        best_score
    }
}
