use kimbo_state::movelist::MoveList;

use super::EnginePosition;
use std::mem;
use std::ptr;

const MVV_LVA: [[i8; 7]; 7] = [
    [15, 14, 13, 12, 11, 10, 0], // victim PAWN
    [25, 24, 23, 22, 21, 20, 0], // victim KNIGHT
    [35, 34, 33, 32, 31, 30, 0], // victim BISHOP
    [45, 44, 43, 42, 41, 40, 0], // victim ROOK
    [55, 54, 53, 52, 51, 50, 0], // victim QUEEN
    [ 0,  0,  0,  0,  0,  0, 0], // victim KING (will not be referenced)
    [ 0,  0,  0,  0,  0,  0, 0], // empty square
];

impl EnginePosition {
    /// Calculates MVV-LVA score for a move
    pub fn mvv_lva(&self, m: u16) -> i8 {
        let from_idx = m & 0b111111;
        let to_idx = (m >> 6) & 0b111111;
        let moved_pc = self.board.squares[from_idx as usize] as usize;
        let captured_pc = self.board.squares[to_idx as usize] as usize;
        MVV_LVA[captured_pc][moved_pc]
    }
    /// Scores moves as follows:
    /// 1. Hash move
    /// 2. Captures sorted via MMV-LVA
    /// 3. Quiets
    pub fn score_move(&self, m: u16, hash_move: u16, move_hit: &mut bool) -> i8 {
        if m == hash_move {
            *move_hit = true;
            i8::MAX
        } else {
            self.mvv_lva(m)
        }
    }

    pub fn score_moves(&self, moves: &MoveList, move_scores: &mut MoveScores, hash_move: u16, move_hit: &mut bool) {
        for i in 0..moves.len() {
            let m = moves[i]; 
            move_scores.push(-self.score_move(m, hash_move, move_hit));
        }
    }

    pub fn score_captures(&self, moves: &MoveList, move_scores: &mut MoveScores) {
        for i in 0..moves.len() {
            let m = moves[i]; 
            move_scores.push(-self.mvv_lva(m));
        }
    }
}

pub struct MoveScores {
    list: [i8; 255],
    len: usize,
}

impl Default for MoveScores {
    fn default() -> Self {
        Self {
            list: unsafe {
                #[allow(clippy::uninit_assumed_init)]
                mem::MaybeUninit::uninit().assume_init()
            },
            len: 0
        } 
    }
}

impl MoveScores {
    #[inline(always)]
    fn push(&mut self, m: i8) {
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

// O(n^2) move sorting, however due to most nodes being cut, its marginally faster than presorting moves
// and more importantly, will be essential when adding more performance intensive move ordering (SEE)
// as 
pub fn get_next_move(moves: &mut MoveList, move_scores: &mut MoveScores, m_idx: &mut usize) -> Option<u16> {
    if *m_idx == move_scores.len {
        return None
    }
    let mut best_idx = 0;
    let mut best_score = i8::MAX;
    for i in *m_idx..move_scores.len {
        let score = move_scores.list[i];
        if score < best_score {
            best_score = score;
            best_idx = i;
        }
    }
    let m = moves[best_idx];
    // swap the found element with the last element in the list
    move_scores.swap_unchecked(best_idx, *m_idx);
    moves.swap_unchecked(best_idx, *m_idx);
    *m_idx += 1;
    Some(m)
}
