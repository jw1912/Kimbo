use super::pst::*;
use super::*;

pub const MG_PC_VALS: [i16; 6] = [82, 337, 365, 477, 1025,  0];
pub const EG_PC_VALS: [i16; 6] = [94, 281, 297, 512,  936,  0];
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
pub fn calc_material<const MG: bool>(pos: &Position) -> [i16; 2] {
    let mut scores = [0; 2];
    for (i, side) in pos.pieces.iter().enumerate() {
        let mut score = 0;
        for (j, piece_val) in (if MG {MG_PC_VALS} else {EG_PC_VALS}).iter().enumerate() {
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
pub fn calc_pst<const MG: bool>(pos: &Position) -> [i16; 2] {
    let mut scores = [0; 2];
    for (i, side) in pos.pieces.iter().enumerate() {
        let mut score = 0;
        for (j, &pc) in side.iter().enumerate() {
            let mut piece = pc;
            while piece > 0 {
                let idx = ls1b_scan(piece) as usize;
                score += get_weight::<MG>(idx, i, j);
                piece &= piece - 1
            }
        }
        scores[i] = score;
    }
    scores
}

impl EnginePosition {
    /// static evaluation of position
    pub fn static_eval(&self) -> i16 {
        let mat_mg = self.mat_mg[0] - self.mat_mg[1];
        let mat_eg = self.mat_eg[0] - self.mat_eg[1];
        let pst_mg = self.pst_mg[0] - self.pst_mg[1];
        let pst_eg = self.pst_eg[0] - self.pst_eg[1];
        let mut phase = self.phase;
        if phase > TOTALPHASE {
            phase = TOTALPHASE
        };
        SIDE_FACTOR[self.board.side_to_move] * (phase * (mat_mg + pst_mg) + (TOTALPHASE - phase) * (mat_eg + pst_eg)) / TOTALPHASE
    }
}
