// This is only used for tuning
use super::{tuner::TunerPosition, *};
use crate::position::*;

#[derive(Copy, Clone, Debug, Default)]
pub struct ParamContainer {
    pub doubled_mg: i16,
    pub doubled_eg: i16,
    pub isolated_mg: i16,
    pub isolated_eg: i16,
    pub passed_mg: i16,
    pub passed_eg: i16,
    pub shield_mg: i16,
    pub shield_eg: i16,
    pub open_file_mg: i16,
    pub open_file_eg: i16,
}

impl From<[i16; 10]> for ParamContainer {
    fn from(x: [i16; 10]) -> Self {
        Self {
            doubled_mg: x[0],
            doubled_eg: x[1],
            isolated_mg: x[2],
            isolated_eg: x[3],
            passed_mg: x[4],
            passed_eg: x[5],
            shield_mg: x[6],
            shield_eg: x[7],
            open_file_mg: x[8],
            open_file_eg: x[9],
        }
    }
}

impl From<ParamContainer> for [i16; 10] {
    fn from(x: ParamContainer) -> Self {
        [
            x.doubled_mg,
            x.doubled_eg,
            x.isolated_mg,
            x.isolated_eg,
            x.passed_mg,
            x.passed_eg,
            x.shield_mg,
            x.shield_eg,
            x.open_file_mg,
            x.open_file_eg,
        ]
    }
}

/// static evaluation of position
#[inline(always)]
pub fn tuner_eval(pos: &TunerPosition, params: &[i16; 10]) -> i16 {
    pos.pst + pawn_eval(pos.phase, pos.pawns, params)
}

#[inline(always)]
pub fn pawn_eval(phase: i16, pawns: [i16; 5], params: &[i16; 10]) -> i16 {
    let mut mg = 0;
    let mut eg = 0;
    for i in 0..5 {
        mg += pawns[i] * params[2 * i];
        eg += pawns[i] * params[2 * i + 1];
    }
    taper(phase as i32, mg, eg)
}

pub fn tuner_pawn_score(pos: &Position, side: usize) -> [i16; 5] {
    let mut doubled = 0;
    let mut isolated = 0;
    let mut passed = 0;
    let mut pawns = pos.pieces[side][0];
    for file in 0..8 {
        let count = (FILES[file] & pawns).count_ones();
        doubled += (count > 1) as i16 * count as i16;
        isolated += (count > 0 && RAILS[file] & pawns == 0) as i16;
    }
    let enemies = pos.pieces[side ^ 1][0];
    while pawns > 0 {
        let pawn = ls1b_scan(pawns) as usize;
        passed += (IN_FRONT[side][pawn] & enemies == 0) as i16;
        pawns &= pawns - 1
    }
    let king_idx = ls1b_scan(pos.pieces[side][Piece::KING]);
    let king_file = (king_idx & 7) as i8;
    let protecting_pawns =
        (KING_ATTACKS[king_idx as usize] & pos.pieces[side][Piece::PAWN]).count_ones() as i16;
    let mut open_files = 0;
    for file in std::cmp::max(0, king_file - 1)..=std::cmp::min(7, king_file + 1) {
        open_files += (FILES[file as usize] & pos.pieces[side][Piece::PAWN] == 0) as i16
    }
    [doubled, isolated, passed, protecting_pawns, open_files]
}
