use super::*;
use super::pst::*;

pub const MAX: i16 = 30000;
pub const PIECE_VALS: [i16; 6] = [100, 320, 330, 500, 900, 0];
const SIDE_FACTOR: [i16; 2] = [1, -1];

/// Calculating material scores from scratch
pub fn calculate_material_scores(pos: &Position) -> [i16; 2] {
    let mut scores = [0; 2];
    for i in 0..2 {
        let mut score = 0;
        for j in 0..6 {
            let mut piece = pos.pieces[i][j];
            let piece_val = PIECE_VALS[j];
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
    for i in 0..2 {
        let mut score = 0;
        for j in 0..6 {
            let mut piece = pos.pieces[i][j];
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
    for i in 0..2 {
        let mut score = 0;
        for j in 0..6 {
            let mut piece = pos.pieces[i][j];
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

const MVV_LVA: [[u8; 8]; 8] = [
    [15, 14, 13, 12, 11, 10, 0, 0], // victim PAWN
    [25, 24, 23, 22, 21, 20, 0, 0], // victim KNIGHT
    [35, 34, 33, 32, 31, 30, 0, 0], // victim BISHOP
    [45, 44, 43, 42, 41, 40, 0, 0], // victim ROOK
    [55, 54, 53, 52, 51, 50, 0, 0], // victim QUEEN
    [0, 0, 0, 0, 0, 0, 0, 0], // victim KING (should not be referenced)
    [0, 0, 0, 0, 0, 0, 0, 0], // oops artifact of 7 != 6
    [0, 0, 0, 0, 0, 0, 0, 0] // empty
];

impl EnginePosition { 
    /// Calculates MVV-LVA score for a move
    pub fn mvv_lva(&self, m: &u16) -> u8 {
        let from_idx = m & 0b111111;
        let to_idx = (m >> 6) & 0b111111;
        let moved_pc = self.board.squares[from_idx as usize] as usize;
        let captured_pc = self.board.squares[to_idx as usize] as usize;
        MVV_LVA[moved_pc][captured_pc]
    }

    /// static evaluation of position
    pub fn static_eval(&self) -> i16 {
        let mat_eval = self.material_scores[0] - self.material_scores[1];
        let pst_mg = self.pst_mg[0] - self.pst_mg[1];
        SIDE_FACTOR[self.board.side_to_move] * (mat_eval + pst_mg)
    }
}