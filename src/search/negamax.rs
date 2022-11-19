use super::{
    Engine,
    MAX_SCORE,
    update_pv,
    pruning::{can_do_lmr, can_do_nmp, can_do_rfp, tt_prune, can_do_pruning, RFP_MARGIN_PER_DEPTH},
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
    /// fail-soft
    #[allow(clippy::too_many_arguments)]
    pub fn negamax<const PV: bool, const ROOT: bool>(
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
        if self.stats.node_count & 2047 == 0 && self.search_limits_reached() {
            self.stop.store(true, Ordering::Relaxed);
            return 0;
        }
        self.stats.seldepth = max(self.stats.seldepth, ply);

        // draw detection
        if self.board.is_draw_by_50() || self.board.is_draw_by_repetition(2 + ROOT as u8) || self.board.is_draw_by_material() {
            return 0;
        }

        // mate distance pruning
        alpha = max(alpha, -MAX_SCORE + ply as i16);
        beta = min(beta, MAX_SCORE - ply as i16 - 1);
        if alpha >= beta {
            return alpha
        }

        // check extensions
        depth += king_in_check as i8;

        // quiescence search at depth <= 0 or maximum ply
        if depth <= 0 || ply == MAX_PLY {
            return self.quiesce(alpha, beta);
        }

        // not a quiescence node so count it
        self.stats.node_count += 1;

        // probing hash table
        let mut hash_move = 0;
        let mut write_to_hash = true;
        if let Some(res) = self.ttable.get(self.board.zobrist, ply) {
            // hash entry found, only write to hash table if this depth search
            // is deeper than the depth of the hash entry
            write_to_hash = depth > res.depth;

            // hash move for move ordering
            hash_move = res.best_move;

            // hash score pruning (no pruning on root)
            if !ROOT &&tt_prune::<PV>(&res, depth, alpha, beta, self.board.halfmove_clock).is_some() {
                    return res.score;
            }

            // we only null move prune when we expect a beta cutoff
            if res.bound != Bound::LOWER && res.score < beta {
                allow_null = false
            }
        }

        // pruning
        if can_do_pruning::<PV>(king_in_check, beta) {
            // just psts and material
            let lazy_eval = self.board.lazy_eval();

            // reverse futility pruning (static null move pruning)
            if can_do_rfp(depth, beta, lazy_eval) {
                return lazy_eval + RFP_MARGIN_PER_DEPTH * depth as i16
            }

            // null move pruning
            if can_do_nmp(allow_null, self.board.phase, depth, beta, lazy_eval) {
                let ctx = self.board.make_null_move();
                let score = -self.negamax::<false, false>(-beta, 1 - beta, depth - 3, ply + 1, &mut Vec::new(), 0, false, false);
                self.board.unmake_null_move(ctx);
                if score >= beta {
                    return score
                }
            }
        }

        // generating moves
        let mut moves = MoveList::default();
        self.board.gen_moves::<{ MoveType::ALL }>(&mut moves);

        // checking for (stale)mate
        if moves.is_empty() {
            return king_in_check as i16 * (-MAX_SCORE + ply as i16);
        }

        // scoring moves
        let mut move_scores = MoveScores::default();
        self.score_moves(&moves, &mut move_scores, hash_move, prev_move, ply);

        // stuff for going through moves
        let mut best_move = 0;
        let mut best_score = -MAX_SCORE;
        let mut bound: u8 = Bound::UPPER;

        // going through moves
        while let Some((m, m_idx, m_score)) = get_next_move(&mut moves, &mut move_scores) {
            let mut sub_pv = Vec::new();

            self.board.make_move(m);

            // late move reductions
            let check = self.board.is_in_check();
            let do_lmr = can_do_lmr::<ROOT>(king_in_check, m_idx, m_score, check, depth);
            let reduction = do_lmr as i8 * (1 + min(2 - PV as i8, ((m_idx - 2) / 4) as i8));

            // pvs framework
            // relies on good move ordering!
            let score = if m_idx == 0 {
                -self.negamax::<PV, false>(-beta, -alpha, depth - 1, ply + 1, &mut sub_pv, m, check, false)
            } else {
                // do a null window search
                let null_window_score = -self.negamax::<false, false>(-alpha - 1, -alpha, depth - 1 - reduction, ply + 1, &mut sub_pv, m, check, true);
                // if it fails high re-search w/ full window and w/out reductions
                if (alpha != beta - 1 || reduction > 0) && null_window_score > alpha {
                    -self.negamax::<PV, false>(-beta, -alpha, depth - 1, ply + 1, &mut sub_pv, m, check, false)
                } else {
                    null_window_score
                }
            };

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
                    update_pv(pv, m, &mut sub_pv);
                }
            }

            // beta pruning
            if score >= beta {
                // counter move, killer move, history heuristics
                if !is_capture(m) {
                    self.ctable.set(prev_move, m);
                    self.ktable.push(m, ply);
                    self.htable.set(self.board.side_to_move, m, depth);
                }
                // lower bound
                bound = Bound::LOWER;
                break;
            }
        }

        // writing to hash table
        if write_to_hash {
            self.ttable.push(self.board.zobrist, best_move, depth, bound, best_score, ply);
        }

        // fail-soft
        best_score
    }
}
