/// This file handles sorting of moves
/// Moves are scored as follows:
/// 1. Hash move (from HashTable)
/// 2. Captures sorted via MMV-LVA
/// 3. Promotions (Queen -> Knight)
/// 4. Killer moves (3 moves per ply in a table)
/// 5. Counter move (from-to table)
/// 6. Castling
/// 7. Quiets

use kimbo_state::MoveList;
use crate::engine::Engine;
use crate::tables::killer::KILLERS_PER_PLY;
use super::{is_capture, is_castling, is_promotion};
use std::mem;
use std::ptr;

// Move ordering scores
const HASH_MOVE: i16 = 100;
const KILLERMOVE: i16 = 5;
const COUNTERMOVE: i16 = 4;
const PROMOTIONS: [i16;4] = [6, 7, 8, 9];
const CASTLE: i16 = 3;
const QUIET: i16 = 0;
const MVV_LVA: [[i16; 7]; 7] = [
    [15, 14, 13, 12, 11, 10, 0], // victim PAWN
    [25, 24, 23, 22, 21, 20, 0], // victim KNIGHT
    [35, 34, 33, 32, 31, 30, 0], // victim BISHOP
    [45, 44, 43, 42, 41, 40, 0], // victim ROOK
    [55, 54, 53, 52, 51, 50, 0], // victim QUEEN
    [ 0,  0,  0,  0,  0,  0, 0], // victim KING (will not be referenced)
    [ 0,  0,  0,  0,  0,  0, 0], // empty square
];

impl Engine {
    pub fn mvv_lva(&self, m: u16) -> i16 {
        let from_idx = m & 0b111111;
        let to_idx = (m >> 6) & 0b111111;
        let moved_pc = self.board.squares[from_idx as usize] as usize;
        let captured_pc = self.board.squares[to_idx as usize] as usize;
        MVV_LVA[captured_pc][moved_pc]
    }

    pub fn score_move<const ROOT: bool>(&mut self, m: u16, hash_move: u16, counter_move: u16, killer_moves: [u16; KILLERS_PER_PLY],move_hit: &mut bool) -> i16 {
        if m == hash_move {
            *move_hit = true;
            HASH_MOVE
        } else if is_capture(m) {
            self.mvv_lva(m)
        } else if is_promotion(m) {
            let pc = (m >> 12) & 3;
            PROMOTIONS[pc as usize]
        } else if killer_moves.contains(&m) {
            self.stats.killermove_hits += 1;
            KILLERMOVE
        } else if !ROOT && m == counter_move {
            self.stats.countermove_hits += 1;
            COUNTERMOVE
        } else if is_castling(m) {
            CASTLE
        } else {
            QUIET
        }
    }
    
    pub fn score_moves<const ROOT: bool>(&mut self, moves: &MoveList, move_scores: &mut MoveScores, hash_move: u16, prev_move: u16, ply: u8, move_hit: &mut bool) {
        let counter_move = self.ctable.get(prev_move);
        let killer_moves = self.ktable.get_ply(ply);
        for i in move_scores.start_idx..moves.len() {
            let m = moves[i]; 
            move_scores.push(self.score_move::<ROOT>(m, hash_move, counter_move, killer_moves, move_hit));
        }
    }

    pub fn score_captures(&self, moves: &MoveList, move_scores: &mut MoveScores) {
        if moves.is_empty() { return }
        for i in move_scores.start_idx..moves.len() {
            let m = moves[i]; 
            move_scores.push(self.mvv_lva(m));
        }
    }
}

pub struct MoveScores {
    list: [i16; 255],
    len: usize,
    start_idx: usize,
}
impl Default for MoveScores {
    fn default() -> Self {
        Self {
            list: unsafe {
                #[allow(clippy::uninit_assumed_init)]
                mem::MaybeUninit::uninit().assume_init()
            },
            len: 0,
            start_idx: 0,
        } 
    }
}
impl MoveScores {
    #[inline(always)]
    fn push(&mut self, m: i16) {
        self.list[self.len] = m;
        self.len += 1;
    }
    #[inline(always)]
    fn swap_unchecked(&mut self, i: usize, j: usize) {
        let ptr = self.list.as_mut_ptr();
        unsafe {
            ptr::swap(ptr.add(i), ptr.add(j));
        }
    }
}

/// Move sort function
/// O(n^2), however with pruning this is actually marginally faster
/// because usually <30% of the moves have to be picked
pub fn get_next_move(moves: &mut MoveList, move_scores: &mut MoveScores) -> Option<u16> {
    if move_scores.start_idx == move_scores.len {
        return None
    }
    let mut best_idx = 0;
    let mut best_score = i16::MIN;
    for i in move_scores.start_idx..move_scores.len {
        let score = move_scores.list[i];
        if score > best_score {
            best_score = score;
            best_idx = i;
        }
    }
    let m = moves[best_idx];
    // swap the found element with the last element in the list
    move_scores.swap_unchecked(best_idx, move_scores.start_idx);
    moves.swap_unchecked(best_idx, move_scores.start_idx);
    move_scores.start_idx += 1;
    Some(m)
}
