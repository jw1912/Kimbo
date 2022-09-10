use super::*;
use crate::engine::{EngineMoveContext, sorting::is_score_near_mate};
use crate::hash::search::CutoffType;
use kimbo_state::{ls1b_scan, MoveType, Check};
use std::sync::atomic::Ordering;
use std::cmp::min;

pub const _RAZORING_MIN_DEPTH: u8 = 1;
pub const _RAZORING_MAX_DEPTH: u8 = 4;
pub const _RAZORING_DEPTH_MARGIN_BASE: i16 = 300;
pub const _RAZORING_DEPTH_MARGIN_MULTIPLIER: i16 = 300;

const _LMP_MIN_DEPTH: u8 = 1;
const _LMP_MAX_DEPTH: u8 = 4;
const _LMP_IDX_BASE: usize = 2;
const _LMP_IDX_MULTIPLIER: usize = 4;
const _LMP_MIN_SCORE: i8 = 0;

const _LMR_MIN_IDX: usize = 2;
const _LMR_MIN_DEPTH: u8 = 2;
const _LMR_MIN_SCORE: i8 = -90;
const _LMR_MAX_REDUCTION: u8 = 3;
const _LMR_PVS_MAX_REDUCTION: u8 = 2;
const _LMR_BASE: usize = 1;
const _LMR_STEP: usize = 4;
const _LMR_PVS_BASE: usize = 1;
const _LMR_PVS_STEP: usize = 8;
const _LMR_PVS_MIN_IDX: usize = 2;

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
        if self.stop.load(Ordering::Relaxed) {
            return 0; // immediately bow out of search
        }

        if self.stats.node_count > self.max_nodes
            || self.stats.start_time.elapsed().as_millis() as u64 > self.max_move_time
        {
            self.stop.store(true, Ordering::Relaxed);
            return 0;
        }

        if ply - 1 > self.stats.seldepth {
            self.stats.seldepth = ply - 1;
        }

        if depth == 0 {
            return self.quiesce::<STATS>(alpha, beta, ply + 1);
        }

        // probing transposition table
        let orig_alpha = alpha;
        let zobrist = self.position.zobrist;
        let mut hash_move = 0;
        let mut entry_found = false;
        let tt_result = self.ttable.get(zobrist, ply);
        if let Some(res) = tt_result {
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
        //let is_searcher_turn = self.searching_side == self.position.board.side_to_move;
        //let friendly_in_check = king_in_check & is_searcher_turn;
        // checking if game is over
        if moves.is_empty() {
            self.stats.node_count += 1;
            let side = self.position.board.side_to_move;
            let idx = ls1b_scan(self.position.board.pieces[side][5]) as usize;
            // checkmate
            if self
                .position
                .board
                .is_square_attacked(idx, side, self.position.board.occupied)
            {   
                return -MAX + ply as i16 - 1;
            }
            // stalemate
            return 0;
        }

        // UNCOMMENT when adding other things that require lazy eval
        //let mut lazy_eval = None;

        //if !PV && _can_razor(depth, alpha, friendly_in_check) {
        //    let margin = _razoring_get_margin(depth);
        //    //let lazy_eval_value = match lazy_eval {
        //    //    Some(value) => value,
        //    //    None => self.position.lazy_eval(),
        //    //};
        //    let lazy_eval_value = self.position.lazy_eval();
//
        //    if lazy_eval_value + margin <= alpha {
        //        let score = self.quiesce(alpha, beta, ply);
        //        if score <= alpha {
        //            return score;
        //        }
        //    }
    //
        //    //lazy_eval = Some(lazy_eval_value);
        //}

        // move sorting
        let mut move_hit: bool = false;
        moves.sort_by_key(|m| self.position.score_move(m, hash_move, &mut move_hit));
        if STATS && move_hit {
            self.stats.tt_move_hits += 1;
        }

        // tracking best score and move, and if alpha changes for ttable
        let mut best_move = 0;
        let mut best_score = -MAX;
        let mut best_d = 0;

        // going through legal moves
        let mut ctx: EngineMoveContext;
        let mut score: i16;
        //for (m_idx, &m) in moves.iter().enumerate() {
        for m in moves {
            // CHECK EXTENSIONS
            let d = if king_in_check {
                1
            } else {
                0
            };
            // LATE MOVE REDUCTIONS
            //let r = if self._can_lmr(depth, m_idx, &m, king_in_check, hash_move) {
            //    min(_get_lmr_depth::<PV>(m_idx), depth - 1)
            //} else {
            //    0
            //};

            // new vector
            let mut sub_pv = Vec::new();
            // making move, getting score, unmaking move
            ctx = self.position.make_move(m);
            // PVS scoring
            //score = if PV {
            //    if m_idx == 0 {
            //        -self.negamax::<true, false>(-beta, -alpha, depth - 1 + d, ply + 1, &mut sub_pv)
            //    }
            //    else {
            //        let zero_score = -self.negamax::<false, false>(-alpha - 1, -alpha, depth - r - 1 + d, ply + 1, &mut sub_pv);
            //        if zero_score > alpha && (alpha != beta - 1 || r > 0) {
            //            -self.negamax::<true, false>(-beta, -alpha, depth - 1 + d, ply + 1, &mut sub_pv)
            //        } else {
            //            zero_score
            //        }
            //    }
            //} else {
            //    let zero_score = -self.negamax::<false, false>(-beta, -alpha, depth - r - 1 + d, ply + 1, &mut sub_pv);
            //    if zero_score > alpha && r > 0 {
            //        -self.negamax::<false, false>(-beta, -alpha, depth - 1 + d, ply + 1, &mut sub_pv)
            //    } else {
            //        zero_score
            //    }
            //};
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

    fn _can_lmr(
        &self,
        depth: u8,
        move_index: usize,
        m: &u16,
        king_checked: bool,
        hash_move: u16
    ) -> bool {
        depth >= _LMR_MIN_DEPTH
            && move_index >= _LMR_MIN_IDX
            && !king_checked
            && (m & 0b0100_0000_0000_0000 == 0)
            && self.position.score_move(m, hash_move, &mut false) >= _LMR_MIN_SCORE
    }
}

fn _get_lmr_depth<const PVS: bool>(m_idx: usize) -> u8 {
    if PVS {
        min(
            _LMR_PVS_MAX_REDUCTION,
            (_LMR_PVS_BASE + (m_idx - _LMR_PVS_MIN_IDX) / _LMR_PVS_STEP) as u8
        )
    } else {
        min(
            _LMR_MAX_REDUCTION,
            (_LMR_BASE + (m_idx - _LMR_MIN_IDX) / _LMR_STEP) as u8
        )
    }
}

fn _can_razor(depth: u8, alpha: i16, friendly_in_check: bool) -> bool {
    (_RAZORING_MIN_DEPTH..=_RAZORING_MAX_DEPTH).contains(&depth) 
        && !is_score_near_mate(alpha) 
        && !friendly_in_check
}

fn _razoring_get_margin(depth: u8) -> i16 {
    _RAZORING_DEPTH_MARGIN_BASE + ((depth - _RAZORING_MIN_DEPTH) as i16) * _RAZORING_DEPTH_MARGIN_MULTIPLIER
}


