use super::{
    Engine,
    MAX_SCORE,
    update_pv,
    pruning::{can_do_hlp, can_do_rfp, can_razor, tt_prune, RFP_MARGIN_PER_DEPTH, RAZOR_MARGIN_PER_DEPTH},
    sorting::{MoveScores, get_next_move}, 
    is_capture, 
    MAX_PLY
};
use crate::tables::search::Bound;
use crate::position::{MoveType, MoveList}; 
use std::sync::atomic::Ordering;
use std::cmp::{max, min};

impl Engine {
    /// Main alpha-beta minimax search
    /// 
    /// Constant parameters:
    ///  - ROOT - is this a root (ply = 0) node?
    ///  - STATS - are debug stats required?
    /// 
    /// source: https://www.chessprogramming.org/Alpha-Beta
    #[allow(clippy::too_many_arguments)]
    pub fn negamax<const PV: bool, const ROOT: bool, const STATS: bool>(
        &mut self, 
        mut alpha: i16, 
        mut beta: i16, 
        mut depth: i8, 
        ply: i8, 
        pv: &mut Vec<u16>, 
        prev_move: u16,
        king_in_check: bool,
        mut allow_null: bool,
    ) -> i16 {
        
        // UCI stuff
        if self.stop.load(Ordering::Relaxed) {
            return 0;
        }
        if self.search_limits_reached() {
            self.stop.store(true, Ordering::Relaxed);
            return 0;
        }
        self.stats.seldepth = std::cmp::max(self.stats.seldepth, ply);

        // draw detection
        // source: https://www.chessprogramming.org/Draw
        if self.board.is_draw_by_50() || self.board.is_draw_by_repetition(2 + ROOT as u8) || self.board.is_draw_by_material() {
            return 0;
        }

        // mate distance pruning
        // source: https://www.chessprogramming.org/Mate_Distance_Pruning
        alpha = max(alpha, -MAX_SCORE + ply as i16);
        beta = min(beta, MAX_SCORE - ply as i16 - 1);
        if alpha >= beta {
            return alpha
        }

        // check extensions
        // source: https://www.chessprogramming.org/Check_Extensions
        depth += king_in_check as i8;

        // quiescence search at depth <= 0 or maximum ply
        // source: https://www.chessprogramming.org/Quiescence_Search
        if depth <= 0 || ply == MAX_PLY {
            return self.quiesce::<STATS>(alpha, beta);
        }

        // not a quiescence node so count it
        self.stats.node_count += 1;

        // probing hash table
        // source: https://www.chessprogramming.org/Transposition_Table
        let zobrist = self.board.zobrist;
        let mut hash_move = 0;
        let mut write_to_hash = true;
        if let Some(res) = self.ttable.get(zobrist, ply, self.age) {
            if STATS { self.stats.tt_hits += 1; }

            // hash entry found, only write to hash table if this depth search  
            // is deeper than the depth of the hash entry
            write_to_hash = depth > res.depth;

            // hash move for move ordering
            hash_move = res.best_move;

            // hash score pruning (no pruning on root)
            if !ROOT {
                if let Some(score) = tt_prune(&res, depth, alpha, beta, self.board.halfmove_clock) {
                    if STATS { self.stats.tt_prunes += 1 }
                    return score;
                }
            }

            // we only null move prune when we expect a beta cutoff
            if res.bound != Bound::LOWER && res.score < beta {
                allow_null = false
            }
        }

        // pruning
        if !PV && !king_in_check {
            let lazy_eval = self.board.lazy_eval();

            // razoring
            // https://www.chessprogramming.org/Razoring
            if can_razor(depth, alpha) {
                let margin = RAZOR_MARGIN_PER_DEPTH * depth as i16;
                if lazy_eval + margin < alpha {
                    if STATS { self.stats.razor_attempts += 1 }
                    let score = self.quiesce::<STATS>(alpha, beta);
                    if score <= alpha {
                        if STATS { self.stats.razor_successes += 1 }
                        return alpha
                    }
                }
            }

            // reverse futility pruning (static null move pruning)
            // source: https://www.chessprogramming.org/Reverse_Futility_Pruning
            if can_do_rfp(depth, beta) {
                if STATS { self.stats.rfp_attempts += 1 }
                let margin = RFP_MARGIN_PER_DEPTH * depth as i16;
                if lazy_eval - margin >= beta {
                    if STATS { self.stats.rfp_successes += 1 }
                    return beta
                }
            }

            // null move pruning
            // source: https://www.chessprogramming.org/Null_Move_Pruning
            if self.can_do_nmp(allow_null, depth, beta) && lazy_eval >= beta - 50 {
                if STATS { self.stats.nmp_attempts += 1 }
                // make null move
                let ctx = self.board.make_null_move();
                // get a score
                let score = -self.negamax::<false, false, STATS>(-beta, 1 - beta, depth - 3, ply + 1, &mut Vec::new(), 0, false, false);
                // unmake null move
                self.board.unmake_null_move(ctx);
                // prune
                if score >= beta {
                    // this was a cause of much pain, forgetting that the tt_hit is never used is pruned here
                    if STATS { self.stats.nmp_successes += 1 }
                    return beta
                }
            }
        }

        // generating moves
        let mut moves = MoveList::default();
        self.board.gen_moves::<{ MoveType::ALL }>(&mut moves);

        // checking for checkmate/stalemate
        if moves.is_empty() {
            return king_in_check as i16 * (-MAX_SCORE + ply as i16);
        }

        // move scoring for move ordering
        let mut move_hit: bool = false;
        let mut move_scores = MoveScores::default();
        self.score_moves::<ROOT>(&moves, &mut move_scores, hash_move, prev_move, ply, &mut move_hit);
        if STATS { 
            if hash_move > 0 { self.stats.tt_move_tries += 1 }
            if move_hit { 
                self.stats.tt_move_hits += 1 
            } else if hash_move > 0 { 
                self.stats.tt_move_misses += 1 
            }
        }
        
        // initialising stuff for going through moves
        let mut best_move = 0;
        let mut best_score = -MAX_SCORE;
        let mut bound: u8 = Bound::UPPER;
        let mut do_pvs = false;

        // going through moves
        while let Some((m, m_idx, m_score)) = get_next_move(&mut moves, &mut move_scores) {

            // making move
            self.board.make_move(m);
            let check = self.board.is_in_check();

            // history leaf pruning
            // source: https://www.chessprogramming.org/History_Leaf_Pruning
            if can_do_hlp::<PV>(king_in_check, depth, m_idx, m_score, check) {
                self.board.unmake_move();
                continue;
            }

            let mut sub_pv = Vec::new();

            // late move reductions
            // source: https://www.chessprogramming.org/Late_Move_Reductions
            let do_lmr = self.can_do_lmr::<ROOT>(king_in_check, depth, m_idx, m_score, check);
            let reduction = do_lmr as i8 * (1 + (m_idx >= 6) as i8);

            // pvs framework
            // source: https://www.chessprogramming.org/Principal_Variation_Search
            let score = if PV || do_pvs {
                if m_idx == 0 {
                    -self.negamax::<true, false, STATS>(-beta, -alpha, depth - 1, ply + 1, &mut sub_pv, m, check, false)
                } else {
                    if STATS { self.stats.lmr_attempts += 1 }
                    let lmr_score = -self.negamax::<false, false, STATS>(-alpha - 1, -alpha, depth - 1 - reduction, ply + 1, &mut sub_pv, m, check, true);
                    if lmr_score > alpha && (lmr_score < beta || reduction > 0) {
                        -self.negamax::<true, false, STATS>(-beta, -alpha, depth - 1, ply + 1, &mut sub_pv, m, check, false)
                    } else {
                        if STATS { self.stats.lmr_successes += 1 }
                        lmr_score
                    }
                }
            } else {
                let lmr_score = -self.negamax::<false, false, STATS>(-beta, -alpha, depth - 1 - reduction, ply + 1, &mut sub_pv, m, check, true);
                if lmr_score > alpha && reduction > 0 {
                    -self.negamax::<false, false, STATS>(-beta, -alpha, depth - 1, ply + 1, &mut sub_pv, m, check, false)
                } else {
                    lmr_score
                }
            };

            // unmaking move
            self.board.unmake_move();

            // alpha improvements
            if score > best_score {
                // update best move and score
                best_score = score;
                best_move = m;
                // improve alpha
                if score > alpha {
                    alpha = score;
                    bound = Bound::EXACT;
                    do_pvs = true;
                    update_pv(pv, m, &mut sub_pv);
                } 
            }

            // beta pruning
            if score >= beta {
                // counter move, killer move, history heuristics
                if !is_capture(m) {
                    // source: https://www.chessprogramming.org/Countermove_Heuristic
                    self.ctable.set(prev_move, m);
                    // source: https://www.chessprogramming.org/Killer_Heuristic
                    self.ktable.push(m, ply);
                    // source: https://www.chessprogramming.org/History_Heuristic
                    self.htable.set(self.board.side_to_move, m, depth);
                }
                bound = Bound::LOWER;
                break;
            } else if !is_capture(m) {
                self.htable.reduce(self.board.side_to_move, m);
            }
        }

        // writing to hash table
        if write_to_hash {
            self.ttable.push(zobrist, best_move, depth, self.age, bound, best_score, ply);
        }

        // return best score
        best_score
    }
}
