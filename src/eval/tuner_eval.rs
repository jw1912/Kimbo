// This is only used for tuning
use crate::position::{*, attacks::{bishop_attacks, rook_attacks}};
use super::{*, tuner::TunerPosition};

pub const NUM_PARAMS: usize = 34;

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
    pub pawn_mg: i16,
    pub pawn_eg: i16,
    pub knight_mg: i16,
    pub knight_eg: i16,
    pub bishop_mg: i16,
    pub bishop_eg: i16,
    pub rook_mg: i16,
    pub rook_eg: i16,
    pub queen_mg: i16,
    pub queen_eg: i16,
    pub king_mg: i16,
    pub king_eg: i16,
    pub outer_knight_mg: i16,
    pub outer_knight_eg: i16,
    pub outer_bishop_mg: i16,
    pub outer_bishop_eg: i16,
    pub outer_rook_mg: i16,
    pub outer_rook_eg: i16,
    pub outer_queen_mg: i16,
    pub outer_queen_eg: i16,
    pub outer_pawn_mg: i16,
    pub outer_pawn_eg: i16,
    pub outer_king_mg: i16,
    pub outer_king_eg: i16,
}

impl From<[i16; NUM_PARAMS]> for ParamContainer {
    fn from(x: [i16; NUM_PARAMS]) -> Self {
        Self {
            doubled_mg: x[0], doubled_eg: x[1],
            isolated_mg: x[2], isolated_eg: x[3],
            passed_mg: x[4], passed_eg: x[5],
            shield_mg: x[6], shield_eg: x[7],
            open_file_mg: x[8], open_file_eg: x[9],
            pawn_mg: x[10], pawn_eg: x[11],
            knight_mg: x[12], knight_eg: x[13],
            bishop_mg: x[14], bishop_eg: x[15],
            rook_mg: x[16], rook_eg: x[17],
            queen_mg: x[18], queen_eg: x[19],
            king_mg: x[20], king_eg: x[21],
            outer_pawn_mg: x[22], outer_pawn_eg: x[23],
            outer_knight_mg: x[24], outer_knight_eg: x[25],
            outer_bishop_mg: x[26], outer_bishop_eg: x[27],
            outer_rook_mg: x[28], outer_rook_eg: x[29],
            outer_queen_mg: x[30], outer_queen_eg: x[31],
            outer_king_mg: x[32], outer_king_eg: x[33]
        }
    }
}

impl From<ParamContainer> for [i16; NUM_PARAMS] {
    fn from(x: ParamContainer) -> Self {
        [
        x.doubled_mg, x.doubled_eg,
        x.isolated_mg, x.isolated_eg,
        x.passed_mg, x.passed_eg,
        x.shield_mg, x.shield_eg,
        x.open_file_mg, x.open_file_eg,
        x.pawn_mg, x.pawn_eg,
        x.knight_mg, x.knight_eg,
        x.bishop_mg, x.bishop_eg,
        x.rook_mg, x.rook_eg,
        x.queen_mg, x.queen_eg,
        x.king_mg, x.king_eg,
        x.outer_pawn_mg, x.outer_pawn_eg,
        x.outer_knight_mg, x.outer_knight_eg,
        x.outer_bishop_mg, x.outer_bishop_eg,
        x.outer_rook_mg, x.outer_rook_eg,
        x.outer_queen_mg, x.outer_queen_eg,
        x.outer_king_mg, x.outer_king_eg,
        ]
    }
}

/// static evaluation of position
#[inline(always)]
pub fn tuner_eval(pos: &TunerPosition, params: &[i16; NUM_PARAMS]) -> i16 {
    pos.pst + pawn_eval(pos.phase, pos.pawns, params) + mob_eval(pos.phase, pos.mob, params)
}

#[inline(always)]
pub fn pawn_eval(phase: i16, pawns: [i16; 5], params: &[i16; NUM_PARAMS]) -> i16 {
    let mut mg = 0;
    let mut eg = 0;
    for i in 0..5 {
        mg += pawns[i] * params[2 * i];
        eg += pawns[i] * params[2 * i + 1];
    }
    taper(phase as i32, mg, eg)
}

pub fn mob_eval(phase: i16, mob: [i16; 12], params: &[i16; NUM_PARAMS]) -> i16 {
    let mut mg = 0;
    let mut eg = 0;
    for i in 0..12 {
        mg += mob[i] * params[10 + 2 * i];
        eg += mob[i] * params[10 + 2 * i + 1];
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
    let protecting_pawns = (KING_ATTACKS[king_idx as usize] & pos.pieces[side][Piece::PAWN]).count_ones() as i16;
    let mut open_files = 0;
    for file in std::cmp::max(0, king_file - 1)..=std::cmp::min(7, king_file + 1) {
        open_files += (FILES[file as usize] & pos.pieces[side][Piece::PAWN] == 0) as i16
    }
    [doubled, isolated, passed, protecting_pawns, open_files]
}

pub fn tuner_mobility_score(pos: &Position, side: usize) -> [i16; 12] {
    let mut mob = [0; 12];
    for pc in Piece::PAWN..=Piece::KING {
        let m = piece_mobility(pos, side, pos.occupied, pc);
        mob[pc] = m.0;
        mob[pc + 6] = m.1;
    }
    mob
}

pub fn piece_mobility(pos: &Position, side: usize, mut occupied: u64, pc: usize) -> (i16, i16) {
    let mut from: u16;
    let mut idx: usize;
    let mut cattacks: i16 = 0;
    let mut rattacks: i16 = 0;
    let mut centers: u64;
    let mut rims: u64;
    let mut attackers: u64 = pos.pieces[side][pc];
    // queen doesn't get in the way of anybody
    occupied &= !pos.pieces[side][Piece::QUEEN];
    match pc {
        Piece::ROOK => occupied &= !pos.pieces[side][Piece::ROOK],
        Piece::BISHOP => occupied &= !pos.pieces[side][Piece::BISHOP],
        Piece::QUEEN => occupied &= !(pos.pieces[side][Piece::BISHOP] | pos.pieces[side][Piece::ROOK]),
        _ => {}
    }
    while attackers > 0 {
        from = ls1b_scan(attackers);
        idx = from as usize;
        centers = match pc {
            Piece::PAWN => PAWN_ATTACKS[side][idx],
            Piece::KNIGHT => KNIGHT_ATTACKS[idx],
            Piece::ROOK => rook_attacks(idx, occupied),
            Piece::BISHOP => bishop_attacks(idx, occupied),
            Piece::QUEEN => rook_attacks(idx, occupied) | bishop_attacks(idx, occupied),
            Piece::KING => KING_ATTACKS[idx],
            _ => panic!("Not a valid usize in fn piece_moves_general: {}", pc),
        } & !pos.sides[side];
        rims = centers & RIM;
        centers &= CENTER;
        cattacks += centers.count_ones() as i16;
        rattacks += rims.count_ones() as i16;
        attackers &= attackers - 1;
    }
    (cattacks, rattacks)
}