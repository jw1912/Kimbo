use super::*;
use crate::engine::EngineMoveContext;
use crate::hash::search::CutoffType;
use kimbo_state::{MoveType, Check};
use std::sync::atomic::Ordering;

impl Search {
    /// returns the evaluation of a position to a given depth
    /// CONSTANT PARAMTERS:
    /// PV - is the node a PV node?
    /// ROOT - is this the entry point for the search? (don't want to prune root nodes)
    /// STATS - get debug stats?
    pub fn negamax<const PV: bool, const ROOT: bool, const STATS: bool>(
        &mut self,
        mut alpha: i16,
        mut beta: i16,
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
        if ply > self.stats.seldepth + 1 {
            self.stats.seldepth = ply - 1;
        }
        // depth 0 quiescence search
        if depth == 0 {
            let mut sub_pv = Vec::new();
            let score = self.quiesce::<STATS>(alpha, beta, ply + 1, &mut sub_pv);
            pv.append(&mut sub_pv);
            return score;
        }
        // probing transposition table
        let orig_alpha = alpha;
        let zobrist = self.position.zobrist;
        let mut hash_move = 0;
        let mut entry_found = false;
        if let Some(res) = self.ttable.get(zobrist, ply, self.age) {
            if STATS { self.stats.tt_hits += 1; }
            hash_move = res.best_move;
            entry_found = true;
            if !ROOT && res.depth >= depth {
                match res.cutoff_type {
                    CutoffType::ALPHA => {
                        if res.score > alpha {                           
                            if STATS { self.stats.tt_cutoffs.0 += 1; }
                            alpha = res.score;
                        }
                    }
                    CutoffType::BETA => {
                        if res.score < beta {
                            if STATS { self.stats.tt_cutoffs.1 += 1; }
                            beta = res.score;
                        }
                    }
                    CutoffType::EXACT => {
                        if !PV {
                            if STATS { 
                                self.stats.tt_cutoffs.2 += 1;
                                self.stats.tt_hits_returned += 1; 
                            }
                            self.stats.node_count += 1;
                            return res.score;
                        }
                    }
                    _ => ()
                }
                if alpha >= beta {
                    if STATS { 
                        self.stats.tt_beta_prunes += 1;
                        self.stats.tt_hits_returned += 1;
                    }
                    self.stats.node_count += 1;
                    return res.score;
                }
            }
        }

        // generating move
        let mut _king_checked = Check::None;
        let mut moves = self.position.board.gen_moves::<{ MoveType::ALL }>(&mut _king_checked);
        let king_in_check = _king_checked != Check::None;
        // checking if game is over
        if moves.is_empty() {
            self.stats.node_count += 1;
            // checkmate
            if king_in_check {  
                return -MAX_SCORE + ply as i16 - 1;
            }
            // stalemate
            return 0;
        }

        // move sorting
        let mut move_hit: bool = false;
        moves.sort_by_key(|m| self.position.score_move(m, hash_move, &mut move_hit));
        if STATS && move_hit {
            self.stats.tt_move_hits += 1;
        }

        // tracking best move information
        let mut best_move = 0;
        let mut best_score = -MAX_SCORE;
        let mut best_d = 0;

        // going through legal moves
        let mut ctx: EngineMoveContext;
        let mut score: i16;
        for m in moves {
            // CHECK EXTENSIONS
            let d = if king_in_check {
                1
            } else {
                0
            };

            // new vector
            let mut sub_pv = Vec::new();
            // making move, getting score, unmaking move
            ctx = self.position.make_move(m);
            score = -self.negamax::<false, false, STATS>(-beta, -alpha, depth - 1 + d, ply + 1, &mut sub_pv);
            self.position.unmake_move(ctx);
            // updating best move and score
            if score > best_score {
                best_score = score;
                best_move = m;
                best_d = d;
            }
            // improve alpha
            if score > alpha {
                alpha = score;
                pv.clear();
                pv.push(m);
                pv.append(&mut sub_pv);
            }
            // beta pruning
            if score >= beta {
                break;
            }
        }
        // writing to tt
        if !entry_found || alpha != orig_alpha {
            let cutoff_type = if alpha <= orig_alpha {
                if STATS { self.stats.tt_additions.0 += 1; }
                CutoffType::ALPHA
            } else if alpha <= beta {
                if STATS { self.stats.tt_additions.1 += 1; }
                CutoffType::BETA
            } else {
                if STATS { self.stats.tt_additions.2 += 1; }
                CutoffType::EXACT
            };
            self.ttable
                .push(zobrist, best_score, best_move, depth + 1 + best_d, ply, self.age, cutoff_type);
        }
        self.stats.node_count += 1;
        best_score
    }
}
