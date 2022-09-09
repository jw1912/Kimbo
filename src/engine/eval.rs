use super::pst::*;
use super::*;
use std::cmp::max;

pub const PIECE_VALS: [i16; 6] = [100, 320, 330, 500, 900, 0];
pub const PHASE_VALS: [i16; 8] = [0, 1, 1, 2, 4, 0, 0, 0];
pub const TOTALPHASE: i16 = 24;
const SIDE_FACTOR: [i16; 3] = [1, -1, 0];

/// Calculating phase
pub fn calculate_phase(pos: &Position) -> i16 {
    let mut phase: i16 = 0;
    for pc in pos.squares {
        phase += PHASE_VALS[pc as usize];
    }
    phase
}

/// Calculating material scores from scratch
pub fn calculate_mat_mg(pos: &Position) -> [i16; 2] {
    let mut scores = [0; 2];
    for (i, side) in pos.pieces.iter().enumerate() {
        let mut score = 0;
        for (j, piece_val) in PIECE_VALS.iter().enumerate() {
            let mut piece = side[j];
            while piece > 0 {
                score += piece_val;
                piece &= piece - 1
            }
        }
        scores[i] = score;
    }
    scores
}
/// Calculate midgame piece-square tables from scratch
pub fn calculate_pst_mg_scores(pos: &Position) -> [i16; 2] {
    let mut scores = [0; 2];
    for (i, side) in pos.pieces.iter().enumerate() {
        let mut score = 0;
        for (j, &pc) in side.iter().enumerate() {
            let mut piece = pc;
            while piece > 0 {
                let idx = ls1b_scan(piece) as usize;
                score += get_mg_weight(idx, i, j);
                piece &= piece - 1
            }
        }
        scores[i] = score;
    }
    scores
}

/// Calculate endgame piece-square tables from scratch
pub fn calculate_pst_eg_scores(pos: &Position) -> [i16; 2] {
    let mut scores = [0; 2];
    for (i, side) in pos.pieces.iter().enumerate() {
        let mut score = 0;
        for (j, &pc) in side.iter().enumerate() {
            let mut piece = pc;
            while piece > 0 {
                let idx = ls1b_scan(piece) as usize;
                score += get_eg_weight(idx, i, j);
                piece &= piece - 1
            }
        }
        scores[i] = score;
    }
    scores
}

const MVV_LVA: [[i8; 8]; 8] = [
    [15, 14, 13, 12, 11, 10, 0, 0], // victim PAWN
    [25, 24, 23, 22, 21, 20, 0, 0], // victim KNIGHT
    [35, 34, 33, 32, 31, 30, 0, 0], // victim BISHOP
    [45, 44, 43, 42, 41, 40, 0, 0], // victim ROOK
    [55, 54, 53, 52, 51, 50, 0, 0], // victim QUEEN
    [0, 0, 0, 0, 0, 0, 0, 0],       // victim KING (should not be referenced)
    [0, 0, 0, 0, 0, 0, 0, 0],       // oops artifact of 7 != 6
    [5, 0, 0, 0, 0, -1, 0, 0],      // empty
];

impl EnginePosition {
    /// Calculates MVV-LVA score for a move
    pub fn mvv_lva(&self, m: &u16) -> i8 {
        let from_idx = m & 0b111111;
        let to_idx = (m >> 6) & 0b111111;
        let moved_pc = self.board.squares[from_idx as usize] as usize;
        let captured_pc = self.board.squares[to_idx as usize] as usize;
        -MVV_LVA[captured_pc][moved_pc]
    }

    fn kings_endgame(&self, side: u8) -> i16 {
        let white = ls1b_scan(self.board.pieces[0][5]) as usize;
        let black = ls1b_scan(self.board.pieces[0][5]) as usize;
        let k_dst = ((white & 7) as i16 - (black & 7) as i16).abs()
            + ((white >> 3) as i16 - (black >> 3) as i16).abs();
        let opponent = match side {
            0 => white,
            1 => black,
            _ => return 0,
        };
        let opp_ctr_dst = max(3 - ((opponent & 7) as i16), ((opponent & 7) as i16) - 4)
            + max(3 - ((opponent >> 3) as i16), ((opponent >> 3) as i16) - 4);
        14 + opp_ctr_dst - k_dst
    }

    /// static evaluation of position
    pub fn static_eval(&self) -> i16 {
        let mat_eval = self.mat_mg[0] - self.mat_mg[1];
        let side = match mat_eval {
            1.. => 0,
            0 => 2,
            _ => 1,
        };
        let pst_mg = self.pst_mg[0] - self.pst_mg[1];
        let pst_eg = self.pst_eg[0] - self.pst_eg[1];
        let mut phase = self.phase;
        if self.phase > TOTALPHASE {
            phase = TOTALPHASE
        };
        SIDE_FACTOR[self.board.side_to_move]
            * (mat_eval
                + (phase * pst_mg + (TOTALPHASE - phase) * (pst_eg + SIDE_FACTOR[side as usize] * self.kings_endgame(side)))
                    / TOTALPHASE)
    }
}
