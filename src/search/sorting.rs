use kimbo_state::MoveList;

use crate::engine::Engine;
use super::is_capture;
use super::is_castling;
use super::is_promotion;
use std::mem;
use std::ptr;

// Move ordering scores
const HASH_MOVE: i16 = 100;
const COUNTERMOVE: i16 = 60;
const PROMOTIONS: [i16;4] = [2, 3, 4, 5];
const CASTLE: i16 = 1;
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
    /// Calculates MVV-LVA score for a move
    pub fn mvv_lva(&self, m: u16) -> i16 {
        let from_idx = m & 0b111111;
        let to_idx = (m >> 6) & 0b111111;
        let moved_pc = self.board.squares[from_idx as usize] as usize;
        let captured_pc = self.board.squares[to_idx as usize] as usize;
        MVV_LVA[captured_pc][moved_pc]
    }

    /// Scores moves as follows:
    /// 1. Hash move
    /// 2. Counter move
    /// 3. Captures sorted via MMV-LVA
    /// 4. Promotions
    /// 5. Castling
    /// 6. Quiets
    pub fn score_move<const ROOT: bool>(&self, m: u16, hash_move: u16, counter_move: u16, move_hit: &mut bool) -> i16 {
        let mut score = if m == hash_move {
            *move_hit = true;
            HASH_MOVE
        } else if is_capture(m) {
            self.mvv_lva(m)
        } else if is_promotion(m) {
            let pc = (m >> 12) & 3;
            PROMOTIONS[pc as usize]
        } else if is_castling(m) {
            CASTLE
        } else {
            QUIET
        };
        if !ROOT && m == counter_move {
            score += COUNTERMOVE
        }
        score
    }
    
    pub fn score_moves<const ROOT: bool>(&self, moves: &MoveList, move_scores: &mut MoveScores, hash_move: u16, prev_move: u16, move_hit: &mut bool) {
        let counter_move = self.ctable.get(prev_move);
        for i in move_scores.start_idx..moves.len() {
            let m = moves[i]; 
            move_scores.push(self.score_move::<ROOT>(m, hash_move, counter_move,  move_hit));
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

/// Score list for move list
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
