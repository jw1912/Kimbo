use super::*;
use crate::engine::{transposition::CuttoffType, EngineMoveContext};
use kimbo_state::{ls1b_scan, MoveType};
use std::sync::atomic::Ordering;

impl Search {
    /// returns the evaluation of a position to a given depth
    pub fn negamax(
        &mut self,
        mut alpha: i16,
        beta: i16,
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

        if ply > self.stats.seldepth {
            self.stats.seldepth = ply;
        }

        if depth == 0 {
            return self.quiesce(alpha, beta, ply + 1);
        }

        // probing transposition table
        let zobrist = self.position.zobrist;
        let mut collision = false;
        let mut hash_move = 0;
        let mut entry_found = false;
        let tt_result = self.ttable.get(zobrist, &mut collision);
        if let Some(res) = tt_result {
            self.stats.tt_hits += 1;
            hash_move = res.best_move;
            entry_found = true;
            if res.depth >= depth && res.cutoff_type == CuttoffType::BETA && res.score > alpha {
                self.stats.cutoff_hits += 1;
                alpha = res.score;
            }
        }
        if collision {
            self.stats.collisions += 1;
        }

        // generating move
        let mut moves = self.position.board.gen_moves::<{ MoveType::ALL }>();
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
                return -MAX;
            }
            // stalemate
            return 0;
        }

        // move sorting
        let mut move_hit: bool = false;
        moves.sort_by_key(|m| {
            if *m == hash_move {
                move_hit = true;
                -100
            } else {
                self.position.mvv_lva(m)
            }
        });
        if move_hit {
            self.stats.tt_move_hits += 1;
        }

        // tracking best score and move, and if alpha changes for ttable
        let orig_alpha = alpha;
        let mut best_move = 0;
        let mut best_score = -MAX - 1;

        // going through legal moves
        let mut ctx: EngineMoveContext;
        let mut score: i16;
        for m in moves {
            // new vector
            let mut sub_pv = Vec::new();
            // making move, getting score, unmaking move
            ctx = self.position.make_move(m);
            score = -self.negamax(-beta, -alpha, depth - 1, ply + 1, &mut sub_pv);
            self.position.unmake_move(ctx);

            // updating best move and score
            if score > best_score {
                best_score = score;
                best_move = m;
                pv.clear();
                pv.push(m);
                pv.append(&mut sub_pv);
            }

            // improve alpha bound
            if score > alpha {
                alpha = score;
            }

            // beta pruning
            if score >= beta {
                break;
            }
        }
        self.stats.node_count += 1;
        // writing to tt
        if !entry_found || alpha != orig_alpha {
            let cutoff_type = if alpha <= orig_alpha {
                CuttoffType::ALPHA
            } else if alpha >= beta {
                CuttoffType::BETA
            } else {
                CuttoffType::EXACT
            };
            self.ttable
                .push(zobrist, best_score, best_move, depth, self.age, cutoff_type);
        }

        best_score
    }
}
