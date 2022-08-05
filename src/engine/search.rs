//use crate::io::outputs::u16_to_uci;

use super::{*,eval::MAX, qsearch::{QS_CALLS, count_qs_plus}};
use std::sync::atomic::Ordering;

impl EnginePosition {
    /// returns the evaluation of a position to a given depth
    pub fn negamax(&mut self, mut alpha: i16, beta: i16, depth: u8) -> i16 {
        if depth == 0 {
            return self.quiesce(alpha, beta, 4)
        }
        let moves = self.board.gen_moves::<{MoveType::ALL}>();
        // game over
        if moves.is_empty() {
            let side = self.board.side_to_move;
            let idx = ls1b_scan(self.board.pieces[side][5]) as usize;
            count_qs_plus();
            // checkmate
            if self.board.is_square_attacked(idx, side, self.board.occupied) {
                return -MAX
            }
            // stalemate
            return 0
        }
        let mut ctx: MoveContext;
        let mut score: i16;
        for m in moves {
            ctx = self.make_move(m);
            score = -self.negamax(-beta, -alpha, depth-1);
            self.unmake_move(ctx);
            // beta pruning
            if score >= beta { 
                count_qs_plus();
                return beta 
            }
            // improve alpha bound
            if score > alpha { alpha = score }
        }
        count_qs_plus();
        alpha
    }

    /// root search
    pub fn negamax_root(&mut self, move_list: Vec<(u16, i16)>, mut alpha: i16, beta: i16, depth: u8) -> Vec<(u16, i16)> {
        let mut new_move_list = Vec::new();
        let mut ctx: MoveContext;
        let mut score: i16;
        for m in move_list {
            ctx = self.make_move(m.0);
            score = -self.negamax(-beta, -alpha, depth-1);
            self.unmake_move(ctx);
            // improve alpha bound
            if score > alpha { alpha = score - 1 }
            new_move_list.push((m.0,score));
        }
        new_move_list.sort_by_key(|a| a.1);
        new_move_list.reverse();
        new_move_list
    }

    /// iterative deepening search
    pub fn analyse(&mut self, depth: u8) -> u16 {
        let moves = self.board.gen_moves::<{MoveType::ALL}>();
        // creating the initial scored move list with all scores set to 0
        let mut move_list: Vec<(u16, i16)> = Vec::new();
        for m in moves {
            move_list.push((m, 0));
        }
        // loop of iterative deepening, up to preset max depth
        for d in 1..(depth+1) {
            let old_list = move_list.clone();
            move_list = self.negamax_root(old_list, -MAX, MAX, d);
            //for m in &move_list {
            //    println!("{}: {}", u16_to_uci(&m.0), &m.1);
            //}
            // if a forced checkmate is found the search ends obviously
            println!("Score at depth {d}: {}", move_list[0].1);
            if move_list[0].1 == MAX || move_list[0].1 == -MAX  {
                break;
            }
        }
        println!("{} leaf nodes searched.", QS_CALLS.load(Ordering::SeqCst));
        QS_CALLS.store(0, Ordering::SeqCst);
        move_list[0].0
    }
}