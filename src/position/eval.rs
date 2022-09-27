use crate::tables::pawn::PawnHashTable;
use super::{ls1b_scan, Piece};
use super::consts::*;
use super::*;

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

const SIDE_FACTOR: [i16; 3] = [1, -1, 0];

#[inline(always)]
pub const fn taper(phase: i32, mg: i16, eg: i16) -> i16 {
    ((phase * mg as i32 + (TOTALPHASE - phase) * eg as i32) / TOTALPHASE) as i16
}

#[inline(always)]
fn eval_factor(phase: i32, mg: [i16; 2], eg: [i16; 2]) -> i16 {
    let eval_mg = mg[0] - mg[1];
    let eval_eg = eg[0] - eg[1];
    taper(phase, eval_mg, eval_eg)
}

impl Position {
    /// eval taking only material and psts into account
    pub fn lazy_eval(&self) -> i16 {
        let mut phase = self.phase as i32;
        if phase > TOTALPHASE {
            phase = TOTALPHASE
        };

        // material and psts
        let mat = eval_factor(phase, self.mat_mg, self.mat_eg);
        let pst = eval_factor(phase, self.pst_mg, self.pst_eg);

        // relative to side due to negamax framework
        SIDE_FACTOR[self.side_to_move] * (mat + pst)
    }

    /// static evaluation of position
    pub fn static_eval<const STATS: bool>(&mut self, ptable: &PawnHashTable) -> i16 {
        // phase value for tapered eval
        let mut phase = self.phase as i32;
        if phase > TOTALPHASE {
            phase = TOTALPHASE
        };

        // material and psts
        let mat = eval_factor(phase, self.mat_mg, self.mat_eg);
        let pst = eval_factor(phase, self.pst_mg, self.pst_eg);

        // probing pawn hash table
        let pwn: i16;
        if let Some(val) = ptable.get(self.pawnhash) {
            // if result found, use it
            pwn = val.score;
        } else {
            pwn = self.side_pawn_score(0, phase) - self.side_pawn_score(1, phase);
            ptable.push(self.pawnhash, pwn);
        }

        // endgame "mop-up" eval for king of winning side
        let mut eval = mat + pst + pwn;
        if eval != 0 {
            eval += self.eg_king_score((eval < 0) as usize, phase)
        }

        // relative to side due to negamax framework
        SIDE_FACTOR[self.side_to_move] * eval
    }

    fn side_pawn_score(&self, side: usize, phase: i32) -> i16 {
        let mut doubled = 0;
        let mut isolated = 0;
        let mut passed = 0;
        let mut chained = 0;
        let mut pawns = self.pieces[side][0];
        // doubled and isolated pawns
        for file in 0..8 {
            let count = (FILES[file] & pawns).count_ones();
            doubled += (count > 1) as i16 * count as i16;
            isolated += (count > 0 && RAILS[file] & pawns == 0) as i16;
        }
        // chained and passed pawns
        let enemies = self.pieces[side ^ 1][0];
        while pawns > 0 {
            let pawn = ls1b_scan(pawns) as usize;
            chained += (CHAINS[pawn] & self.pieces[side][Piece::PAWN]).count_ones() as i16;
            passed += (IN_FRONT[side][pawn] & enemies == 0) as i16;
            pawns &= pawns - 1
        }
        // a *very* primitive king safety eval
        let king_idx = ls1b_scan(self.pieces[side][Piece::KING]);
        let king_file = (king_idx & 7) as i8;
        let protecting_pawns = (KING_ATTACKS[king_idx as usize] & self.pieces[side][Piece::PAWN]).count_ones() as i16;
        let mut open_files = 0;
        for file in std::cmp::max(0, king_file - 1)..=std::cmp::min(7, king_file + 1) {
            open_files += (FILES[file as usize] & self.pieces[side][Piece::PAWN] == 0) as i16
        }
        // score
        let mg = doubled * DOUBLED_MG + isolated * ISOLATED_MG 
                    + passed * PASSED_MG + chained * CHAINED_MG 
                    + protecting_pawns * PAWN_SHIELD_MG + open_files * PAWN_OPEN_FILE_MG;
        let eg = doubled * DOUBLED_EG + isolated * ISOLATED_EG 
                    + passed * PASSED_EG + chained * CHAINED_EG
                    + protecting_pawns * PAWN_SHIELD_EG + open_files * PAWN_OPEN_FILE_EG;
        taper(phase, mg, eg)
    }

    fn eg_king_score(&self, winning_side: usize, phase: i32) -> i16 {
        let losing_side = winning_side ^ 1;
        let losing_king = ls1b_scan(self.pieces[losing_side][Piece::KING]) as i16;
        let winning_king = ls1b_scan(self.pieces[winning_side][Piece::KING]) as i16;
        let cmd = CMD[losing_king as usize];
        let md = ((losing_king >> 3) - (winning_king >> 3)).abs() + ((losing_king & 7) - (winning_king & 7)).abs();
        let mut score = 5 * cmd + 2 * ( 14 - md );
        score = taper(phase, 0, score);
        SIDE_FACTOR[winning_side] * score
    }

    pub fn is_draw_by_repetition(&self, num: u8) -> bool {
        let l = self.state_stack.len();
        if l < 6 || self.null_counter > 0 { return false }
        let to = l - 1;
        let mut from = l.wrapping_sub(self.halfmove_clock as usize);
        if from > 1024 { from = 0 }
        let mut repetitions_count = 1;
        for i in (from..to).rev().step_by(2) {
            if self.state_stack[i].zobrist == self.zobrist {
                repetitions_count += 1;
                if repetitions_count >= num { return true }
            }
        }
        false
    }
    
    pub fn is_draw_by_50(&self) -> bool {
        self.null_counter == 0 && self.halfmove_clock >= 100
    }

    /// is the game drawn by insufficient material?
    /// 
    /// SOURCE: https://www.chessprogramming.org/Draw_Evaluation
    /// 
    /// can claim draw by FIDE rules:
    ///  - KvK
    ///  - KvKN or KvKB
    ///  - KBvKB and both bishops same colour
    pub fn is_draw_by_material(&self) -> bool {
        let pawns = self.pieces[0][Piece::PAWN] | self.pieces[1][Piece::PAWN];
        // pawns left? not draw. more than one minor piece on either side? not draw.
        if pawns == 0 && self.mat_eg[0] <= EG_PC_VALS[Piece::BISHOP] && self.mat_eg[1] <= EG_PC_VALS[Piece::BISHOP] {
            let total_mat = self.mat_eg[0] + self.mat_eg[1];
            // two minor pieces left
            if total_mat >= 2 * EG_PC_VALS[Piece::KNIGHT] {
                // its two bishops
                if total_mat == 2 * EG_PC_VALS[Piece::BISHOP] {
                    let bishops = self.pieces[0][Piece::BISHOP] | self.pieces[1][Piece::BISHOP];
                    // are bishops on opposite or same colour squares
                    if bishops & SQ1 == bishops || bishops & SQ2 == bishops {
                        return true
                    }
                }
                return false
            }
            // 1 or zero minor pieces is a draw
            return true
        }
        false
    }

    pub fn is_in_check(&self) -> bool {
        let king_idx = ls1b_scan(self.pieces[self.side_to_move][Piece::KING]) as usize;
        self.is_square_attacked(king_idx, self.side_to_move, self.occupied)
    }
}

const SQ1: u64 = 0x55AA55AA55AA55AA;
const SQ2: u64 = 0xAA55AA55AA55AA55;
