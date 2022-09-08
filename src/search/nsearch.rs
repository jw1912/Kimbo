use super::Search;
use crate::{io::outputs::uci_info, engine::transposition::CuttoffType};
use std::{sync::atomic::Ordering, time::Instant};
use crate::engine::EngineMoveContext;
use kimbo_state::{MoveType, ls1b_scan};
use super::*;

impl Search {
    /// returns the evaluation of a position to a given depth
    pub fn negamax(&mut self, mut alpha: i16, mut beta: i16, depth: u8, ply: u8) -> i16 {
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
                self.mates += 1;
                return -MAX;
            }
            // stalemate
            return 0;
        }

        // probing transposition table
        let zobrist = self.position.zobrist;
        let mut collision = false;
        let mut hash_move = 0;
        let mut entry_found = false;
        let tt_result = self.ttable.get(zobrist, &mut collision);
        if let Some(res) = tt_result {
            self.tt_hits.0 += 1;
            hash_move = res.best_move;
            entry_found = true;
            if res.depth >= depth {
                self.tt_hits.1 += 1;
                match res.cutoff_type {
                    CuttoffType::ALPHA => {
                        if res.score < beta {
                            beta = res.score;
                        }
                    }
                    CuttoffType::BETA => {
                        if res.score > alpha {
                            alpha = res.score;
                        }
                    }
                    CuttoffType::EXACT => {
                        return res.score
                    }
                    _ => ()
                }
            }
        }

        let orig_alpha = alpha;
        let mut ctx: EngineMoveContext;
        let mut score: i16;
        moves.sort_by_key(|m| 
            if *m == hash_move {
                -100
            }
            else {self.position.mvv_lva(m)}
        );
        let mut best_move = 0;
        let mut best_score = -MAX - 1;

        for m in moves {
            ctx = self.position.make_move(m);
            score = -self.negamax(-beta, -alpha, depth - 1, ply + 1);
            self.position.unmake_move(ctx);
            if score > best_score {
                best_score = score;
                best_move = m;
            }

            // improve alpha bound
            if score > alpha {
                alpha = score
            }

            // beta pruning
            if score >= beta {
                //self.ttable.push(zobrist, beta, m, depth, self.age, CuttoffType::BETA);
                //return beta;
                break;
            }
        }
        self.node_count += 1;
        if !entry_found || alpha != orig_alpha {
            let cutoff_type = if alpha <= orig_alpha {
                CuttoffType::ALPHA
            } else if alpha >= beta {
                CuttoffType::BETA
            } else {
                CuttoffType::EXACT
            };
            self.ttable.push(zobrist, best_score, best_move, depth, self.age, cutoff_type);
        }
        
        best_score
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
            score = -self.negamax(-beta, -alpha, depth - 1, 1);
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
            if (self.start_time.elapsed().as_millis() as f64) / (self.max_move_time as f64) >= 0.5 
            || (self.node_count as f64) / (self.max_nodes as f64) >= 0.5 
            {
                break;
            }

            move_list = self.negamax_root(move_list, -MAX, MAX, d + 1);
            let score = move_list[0].1;
            
            if self.stop.load(Ordering::Relaxed) || self.node_count > self.max_nodes {
                break;
            }

            let elapsed = self.start_time.elapsed().as_millis() as u64;
            uci_info(d + 1, self.node_count - self.old_count, elapsed - self.old_time, vec![self.best_move], score, self.ttable.filled.load(Ordering::Relaxed), self.ttable.num_entries as u64);
            println!("table hits: {}, quality hits: {}, mates: {}, total nodes: {}", self.tt_hits.0, self.tt_hits.1, self.mates, self.node_count);
            self.old_time = elapsed;
            self.old_count = self.node_count;

            if score == MAX || score == -MAX {
                break;
            }
        }
        self.node_count = 0;
        self.old_count = 0;
        self.old_time = 0;
        self.tt_hits = (0,0);
        self.mates = 0;
        self.age += 1;
        self.best_move
    }
}
