use super::{
    Engine,
    MAX_SCORE,
    update_pv,
    pruning::tt_prune,
    sorting::{MoveScores, get_next_move}, 
    is_capture, 
    MAX_PLY
};
use crate::tables::search::Bound;
use crate::position::{MoveType, Check, MoveList}; 
use std::sync::atomic::Ordering;
use std::cmp::{max, min};

// Comments:
// UCI: implemented for the uci protocol / debug stats
// SAFE: will not distort search results to incorrect values
// UNSAFE: potential to distort search results to be incorrect
// JUSTIFICATION: if SAFE, reason why safe, if UNSAFE, reason why included

impl Engine {
    /// Main alpha-beta minimax search
    /// 
    /// Constant parameters:
    ///  - ROOT - is this a root (ply = 0) node?
    ///  - STATS - are debug stats required?
    /// 
    /// SOURCE: https://www.chessprogramming.org/Alpha-Beta
    #[allow(clippy::too_many_arguments)]
    pub fn negamax<const ROOT: bool, const STATS: bool>(
        &mut self, 
        mut alpha: i16, 
        mut beta: i16, 
        depth: i8, 
        ply: i8, 
        pv: &mut Vec<u16>, 
        prev_move: u16,
        king_in_check: bool,
    ) -> i16 {
        
        // UCI: if stop token, abort
        if self.stop.load(Ordering::Relaxed) {
            return 0;
        }

        // UCI: check if nodes or time limits reached
        if self.search_limits_reached() {
            self.stop.store(true, Ordering::Relaxed);
            return 0;
        }

        // UCI: count node
        self.stats.node_count += 1;

        // UCI: update seldepth (due to extensions)
        self.stats.seldepth = std::cmp::max(self.stats.seldepth, ply);

        // draw detection
        // SOURCE: https://www.chessprogramming.org/Draw
        // the if ROOT is needed in case engine is given a position
        // where a draw by repetition is already about to happen
        // to avoid returning immediately at the root with no best move
        if self.board.is_draw_by_50() || self.board.is_draw_by_repetition(2 + ROOT as u8) || self.board.is_draw_by_material() {
            if STATS { self.stats.draws_detected += 1 }
            return 0;
        }

        // SAFE: mate distance pruning
        // SOURCE: https://www.chessprogramming.org/Mate_Distance_Pruning
        // JUSTIFICATION: only applies when a mate score is returned in the previous
        // child node of the parent node, and the cutoff would be caused later anyway
        alpha = max(alpha, -MAX_SCORE + ply as i16);
        beta = min(beta, MAX_SCORE - ply as i16 - 1);
        if alpha >= beta {
            return alpha
        }

        // quiescence search at depth <= 0 or maximum ply
        // SOURCE: https://www.chessprogramming.org/Quiescence_Search
        if depth <= 0 || ply == MAX_PLY {
            return self.quiesce::<STATS>(alpha, beta);
        }

        // probing hash table
        // SOURCE: https://www.chessprogramming.org/Transposition_Table
        let zobrist = self.board.zobrist;
        let mut hash_move = 0;
        let mut write_to_hash = true;
        if let Some(res) = self.ttable.get(zobrist, ply, self.age) {
            if STATS { self.stats.tt_hits += 1; }

            // hash entry found, only write to hash table if this depth search  
            // is deeper than the depth of the hash entry
            write_to_hash = depth > res.depth;

            // SAFE: hash move
            // JUSTIFICATION: move ordering technique
            hash_move = res.best_move;

            // UNSAFE: hash score pruning (no pruning on root)
            // JUSTIFICATION: >99% of hash moves are valid moves on average
            // so unlikely to effect search results too much
            if !ROOT {
                if let Some(score) = tt_prune(&res, depth, alpha, beta) {
                    if STATS { self.stats.tt_prunes += 1 }
                    pv.push(hash_move);
                    return score;
                }
            }
        }

        // SAFE: check extensions
        // SOURCE: https://www.chessprogramming.org/Check_Extensions
        // JUSTIFICATION: not given higher priority than any other searches at this
        // depth (recorded in hash table at same depth as other nodes at this ply)
        let ext = king_in_check as i8;

        // generating moves
        let mut king_checked = Check::None;
        let mut moves = MoveList::default();
        self.board.gen_moves::<{ MoveType::ALL }>(&mut king_checked, &mut moves);

        // checking for checkmate/stalemate
        if moves.is_empty() {
            return king_in_check as i16 * (-MAX_SCORE + ply as i16);
        }

        // move scoring for move ordering
        let mut move_hit: bool = false;
        let mut move_scores = MoveScores::default();
        self.score_moves::<ROOT>(&moves, &mut move_scores, hash_move, prev_move, ply, &mut move_hit);
        if STATS && move_hit { self.stats.tt_move_hits += 1 }
        
        // initialising stuff for going through moves
        let mut best_move = 0;
        let mut best_score = -MAX_SCORE;
        let mut bound: u8 = Bound::UPPER;

        // going through moves
        while let Some((m, m_idx, m_score)) = get_next_move(&mut moves, &mut move_scores) {

            // making move
            self.board.make_move(m);

            // UNSAFE: late move reductions
            // SOURCE: https://www.chessprogramming.org/Late_Move_Reductions
            // JUSTIFICATION: done in PVS, if fails, reduction removed on research
            let check = self.board.is_in_check();
            let do_lmr = self.can_do_lmr::<ROOT>(ext, depth, m_idx, m_score, check);

            // scoring move and getting the pv for it
            // reduced moves are done witihn a pvs framework
            // other moves are searched normally, with extensions if relevant
            let mut sub_pv = Vec::new();
            let score = if do_lmr {
                if STATS { self.stats.lmr_attempts += 1 }
                let reduce = if m_idx < 6 {1} else {m_idx / 3} as i8;
                let lmr_score = -self.negamax::<false, STATS>(-alpha-1, -alpha, depth - 1 - reduce, ply + 1, &mut sub_pv, m, check);
                if lmr_score > alpha {
                    -self.negamax::<false, STATS>(-beta, -alpha, depth - 1, ply + 1, &mut sub_pv, m, check)
                } else {
                    if STATS { self.stats.lmr_successes += 1 }
                    lmr_score
                }
            } else {
                -self.negamax::<false, STATS>(-beta, -alpha, depth - 1 + ext, ply + 1, &mut sub_pv, m, check)
            };

            // unmaking move
            self.board.unmake_move(m);

            // alpha improvements
            if score > best_score {
                // update best move and score
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
                // SAFE: counter move, killer move, history heuristics
                // JUSTICIFICATION: move ordering techniques
                if !is_capture(m) {
                    // SOURCE: https://www.chessprogramming.org/Countermove_Heuristic
                    self.ctable.set(prev_move, m);
                    // SOURCE: https://www.chessprogramming.org/Killer_Heuristic
                    self.ktable.push(m, ply);
                    // SOURCE: https://www.chessprogramming.org/History_Heuristic
                    self.htable.set(self.board.side_to_move, m, depth);
                }
                bound = Bound::LOWER;
                break;
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
