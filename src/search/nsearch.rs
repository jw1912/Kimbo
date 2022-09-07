use super::Search;
use crate::io::outputs::uci_info;
use std::{sync::atomic::Ordering, time::Instant};
use crate::engine::EngineMoveContext;
use kimbo_state::{MoveType, ls1b_scan};
use super::*;

impl Search {
    /// returns the evaluation of a position to a given depth
    pub fn negamax(&mut self, mut alpha: i16, beta: i16, depth: u8) -> i16 {
        if self.stop.load(Ordering::Relaxed) {
            return 0 // immediately bow out of search
        }

        if self.node_count > self.max_nodes || self.start_time.elapsed().as_millis() as u64 > self.max_move_time {
            self.stop.store(true, Ordering::Relaxed);
            return 0
        }

        if depth == 0 {
            return self.quiesce(alpha, beta);
        }
        let mut moves = self.position.board.gen_moves::<{ MoveType::ALL }>();
        // game over
        if moves.is_empty() {
            self.node_count += 1;
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
        let mut ctx: EngineMoveContext;
        let mut score: i16;
        moves.sort_by_key(|m| self.position.mvv_lva(m));
        for m in moves {
            ctx = self.position.make_move(m);
            score = -self.negamax(-beta, -alpha, depth - 1);
            self.position.unmake_move(ctx);
            // beta pruning
            if score >= beta {
                self.node_count += 1;
                return beta;
            }
            // improve alpha bound
            if score > alpha {
                alpha = score
            }
        }
        self.node_count += 1;
        alpha
    }

    /// root search
    pub fn negamax_root(
        &mut self,
        move_list: Vec<(u16, i16)>,
        mut alpha: i16,
        beta: i16,
        depth: u8,
    ) -> Vec<(u16, i16)> {
        let mut new_move_list = Vec::with_capacity(64);
        let mut ctx: EngineMoveContext;
        let mut score: i16;
        for m in move_list {
            ctx = self.position.make_move(m.0);
            score = -self.negamax(-beta, -alpha, depth - 1);
            self.position.unmake_move(ctx);
            // improve alpha bound
            if score > alpha {
                alpha = score - 1
            }
            new_move_list.push((m.0, score));
        }
        new_move_list.sort_by_key(|a| -a.1);
        if !self.stop.load(Ordering::Relaxed) {
            self.best_move = new_move_list[0].0;
        }
        new_move_list
    }

    /// iterative deepening search
    pub fn go(&mut self) -> u16 {
        let moves = self.position.board.gen_moves::<{ MoveType::ALL }>();
        // creating the initial scored move list with all scores set to 0
        let mut move_list: Vec<(u16, i16)> = Vec::with_capacity(64);
        for m in moves {
            move_list.push((m, 0));
        }
        // loop of iterative deepening, up to preset max depth
        self.start_time = Instant::now();
        for d in 0..self.max_depth {
            if (self.start_time.elapsed().as_millis() as f64) / (self.max_move_time as f64) >= 0.4 
            || (self.node_count as f64) / (self.max_nodes as f64) >= 0.5 
            {
                break;
            }

            move_list = self.negamax_root(move_list, -MAX, MAX, d + 1);

            if self.stop.load(Ordering::Relaxed) || self.node_count > self.max_nodes {
                break;
            }
            let elapsed = self.start_time.elapsed().as_millis() as u64;
            uci_info(d + 1, self.node_count - self.old_count, elapsed - self.old_time, vec![self.best_move], move_list[0].1);
            self.old_time = elapsed;
            self.old_count = self.node_count;
            if move_list[0].1 == MAX || move_list[0].1 == -MAX {
                break;
            }
        }
        self.node_count = 0;
        self.old_count = 0;
        self.old_time = 0;
        self.best_move
    }
}
