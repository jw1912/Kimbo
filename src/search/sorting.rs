/// This file handles sorting of moves
/// Moves are sorted as follows:
/// 1. Hash move (from HashTable)
/// 2. Captures sorted via MMV-LVA
/// 3. Promotions (Queen -> Knight)
/// 4. Killer moves (3 moves per ply in KillerMoveTable)
/// 5. Counter move (from-to CounterMoveTable)
/// 6. Castling
/// 7. Quiets

use crate::position::MoveList;
use crate::tables::killer::KILLERS_PER_PLY;
use super::{Engine, is_capture, is_castling, is_promotion};
use std::mem;
use std::ptr;

// Move ordering scores
const HASH_MOVE: i16 = 30000;
const KILLERMOVE: i16 = 500;
const COUNTERMOVE: i16 = 400;
const PROMOTIONS: [i16;4] = [600, 700, 800, 900];
const CASTLE: i16 = 300;
pub const HISTORY_MAX: i16 = 200;
//const QUIET: i16 = 0;
const MVV_LVA: [[i16; 7]; 7] = [
    [1500, 1400, 1300, 1200, 1100, 1000,   0], // victim PAWN
    [2500, 2400, 2300, 2200, 2100, 2000,   0], // victim KNIGHT
    [3500, 3400, 3300, 3200, 3100, 3000,   0], // victim BISHOP
    [4500, 4400, 4300, 4200, 4100, 4000,   0], // victim ROOK
    [5500, 5400, 5300, 5200, 5100, 5000,   0], // victim QUEEN
    [   0,    0,    0,    0,    0,    0,   0], // victim KING (will not be referenced)
    [   0,    0,    0,    0,    0,    0,   0], // empty square
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
            self.htable.get(self.board.side_to_move, m, &mut self.stats.history_hits)
        }
    }
    
    pub fn score_moves<const ROOT: bool>(&mut self, moves: &MoveList, move_scores: &mut MoveScores, hash_move: u16, prev_move: u16, ply: i8, move_hit: &mut bool) {
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
pub fn get_next_move(moves: &mut MoveList, move_scores: &mut MoveScores) -> Option<(u16, usize, i16)> {
    let m_idx = move_scores.start_idx;
    if m_idx == move_scores.len {
        return None
    }
    let mut best_idx = 0;
    let mut best_score = i16::MIN;
    for i in m_idx..move_scores.len {
        let score = move_scores.list[i];
        if score > best_score {
            best_score = score;
            best_idx = i;
        }
    }
    let m = moves[best_idx];
    // swap the found element with the last element in the list
    move_scores.swap_unchecked(best_idx, m_idx);
    moves.swap_unchecked(best_idx, m_idx);
    move_scores.start_idx += 1;
    Some((m, m_idx, best_score))
}
