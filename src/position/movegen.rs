use super::{attacks::*, consts::*, *};
use super::MoveList;

const RANK_2: u64 = 0x000000000000FF00;
const RANK_4: u64 = 0x00000000FF000000;
const RANK_5: u64 = 0x000000FF00000000;
const RANK_7: u64 = 0x00FF000000000000;
pub const PENRANK: [u64; 2] = [RANK_7, RANK_2];
const DBLRANK: [u64; 2] = [RANK_4, RANK_5];

fn shift<const SIDE: usize, const AMOUNT: u8>(bb: u64) -> u64 {
    match SIDE {
        Side::WHITE => bb >> AMOUNT,
        Side::BLACK => bb << AMOUNT,
        _ => panic!("Invalid side in fn shift!"),
    }
}

fn idx_shift<const SIDE: usize, const AMOUNT: u16>(idx: u16) -> u16 {
    match SIDE {
        Side::WHITE => idx + AMOUNT,
        Side::BLACK => idx - AMOUNT,
        _ => panic!("Invalid side in fn shift!"),
    }
}

#[inline(always)]
fn push_promos(move_list: &mut MoveList, to: u16, from: u16) {
    move_list.push(MoveFlags::KNIGHT_PROMO | to | from);
    move_list.push(MoveFlags::BISHOP_PROMO | to | from);
    move_list.push(MoveFlags::ROOK_PROMO | to | from);
    move_list.push(MoveFlags::QUEEN_PROMO | to | from);
}

#[inline(always)]
fn push_promo_captures(move_list: &mut MoveList, to: u16, from: u16) {
    move_list.push(MoveFlags::KNIGHT_PROMO_CAPTURE | to | from);
    move_list.push(MoveFlags::BISHOP_PROMO_CAPTURE | to | from);
    move_list.push(MoveFlags::ROOK_PROMO_CAPTURE | to | from);
    move_list.push(MoveFlags::QUEEN_PROMO_CAPTURE | to | from);
}

impl Position {
    fn encode_move<const IS_KING: bool, const IS_CAPTURE: bool>(&self, move_list: &mut MoveList, mut attacks: u64, from: u16) {
        let mut aidx: u16;
        while attacks > 0 {
            aidx = ls1b_scan(attacks);
            if IS_KING
                && self.is_square_attacked(aidx as usize, self.side_to_move, self.occupied & !self.pieces[self.side_to_move][Piece::KING])
            {
                attacks &= attacks - 1;
                continue;
            }
            match IS_CAPTURE {
                true => move_list.push(MoveFlags::CAPTURE | (aidx << 6) | from),
                false => move_list.push((aidx << 6) | from),
            }
            attacks &= attacks - 1;
        }
    }

    /// validates whether pseudo-legal move is legal, by playing the move
    fn validate(mut self, m: u16) -> bool {
        let other = self.side_to_move;
        self.make_validation_move(m);
        let idx = ls1b_scan(self.pieces[other][Piece::KING]) as usize;
        !self.is_square_attacked(idx, other, self.occupied)
    }

    /// en passant if available
    pub fn en_passants(&self, move_list: &mut MoveList) {
        let sq = self.en_passant_sq;
        let mut attackers = PAWN_ATTACKS[self.side_to_move ^ 1][sq as usize] & self.pieces[self.side_to_move][Piece::PAWN];
        while attackers > 0 {
            let cls1b = attackers & attackers.wrapping_neg();
            let cidx = ls1b_scan(cls1b);
            let m = MoveFlags::EN_PASSANT | (sq << 6) | cidx;
            if self.clone().validate(m) { move_list.push( m ) }
            attackers &= attackers - 1;
        }
    }
     
    /// finds all unpinned-pawn pushes
    fn pawn_pushes_general<const SIDE: usize, const IN_CHECK: bool, const PINNED: bool>(
        &self,
        move_list: &mut MoveList,
        pawns: u64,
        free: u64, // for in check only
    ) {
        let empty = !self.occupied;
        let mut pushable_pawns: u64;
        let mut dbl_pushable_pawns: u64;
        match IN_CHECK {
            false => {
                pushable_pawns = shift::<SIDE, 8>(empty) & pawns;
                dbl_pushable_pawns = shift::<SIDE, 8>(shift::<SIDE, 8>(empty & DBLRANK[SIDE]) & empty) & pawns;
            }
            true => {
                pushable_pawns = shift::<SIDE, 8>(empty & free) & pawns;
                dbl_pushable_pawns = shift::<SIDE, 8>(shift::<SIDE, 8>(empty & DBLRANK[SIDE] & free) & empty) & pawns;
            }
        }
        let mut promotable_pawns = pushable_pawns & PENRANK[SIDE];
        pushable_pawns &= !PENRANK[SIDE];
        let mut idx: u16;
        let mut m: u16;
        while pushable_pawns > 0 {
            idx = ls1b_scan(pushable_pawns);
            pushable_pawns &= pushable_pawns - 1;
            m = idx_shift::<SIDE, 8>(idx) << 6 | idx;
            if PINNED {
                if self.clone().validate(m) {
                    move_list.push(m);
                }
                continue;
            } 
            move_list.push(m);
        }
        while promotable_pawns > 0 {
            idx = ls1b_scan(promotable_pawns);
            promotable_pawns &= promotable_pawns - 1;
            m = MoveFlags::KNIGHT_PROMO | idx_shift::<SIDE, 8>(idx) << 6 | idx;
            if PINNED {
                if self.clone().validate(m) {
                    push_promos(move_list, idx_shift::<SIDE, 8>(idx) << 6, idx);
                }
                continue;
            }
            push_promos(move_list, idx_shift::<SIDE, 8>(idx) << 6, idx);
        }
        while dbl_pushable_pawns > 0 {
            idx = ls1b_scan(dbl_pushable_pawns);
            dbl_pushable_pawns &= dbl_pushable_pawns - 1;
            m = MoveFlags::DBL_PUSH | idx_shift::<SIDE, 16>(idx) << 6 | idx;
            if PINNED { 
                if self.clone().validate(m) {
                    move_list.push(m);
                } 
                continue;
            }
            move_list.push(m);
        }
    }

    fn pawn_captures_general<const SIDE: usize, const IN_CHECK: bool, const IS_PINNED: bool>(&self, move_list: &mut MoveList, checkers: u64, mut attackers: u64, king_idx: usize) {
        let mut from: u16;
        let mut idx: usize;
        let mut attacks: u64;
        let opponents = self.sides[SIDE ^ 1];
        let mut promo_attackers = attackers & PENRANK[SIDE];
        attackers &= !PENRANK[SIDE];
        while attackers > 0 {
            from = ls1b_scan(attackers);
            idx = from as usize;
            attacks = PAWN_ATTACKS[self.side_to_move][idx] & opponents;
            if IS_PINNED { attacks &= LINE_THROUGH[king_idx][idx] }
            if IN_CHECK { attacks &= checkers }
            self.encode_move::<false, true>(move_list, attacks, from);
            attackers &= attackers - 1;
        }
        let mut cls1b: u64;
        let mut cidx: u16;
        while promo_attackers > 0 {
            from = ls1b_scan(promo_attackers);
            idx = from as usize;
            attacks = PAWN_ATTACKS[self.side_to_move][idx] & opponents;
            if IS_PINNED { attacks &= LINE_THROUGH[king_idx][idx] }
            if IN_CHECK { attacks &= checkers }
            while attacks > 0 {
                cls1b = attacks & attacks.wrapping_neg();
                cidx = ls1b_scan(cls1b);
                push_promo_captures(move_list, cidx << 6, from);
                attacks &= attacks - 1;
            }
            promo_attackers &= promo_attackers - 1;
        }
    }

    fn piece_moves_general<const PIECE: usize, const IN_CHECK: bool, const IS_PINNED: bool, const IS_KING: bool, const TYPE: u8>(
        &self, move_list: &mut MoveList, king_idx: usize, blockers: u64, mut attackers: u64
    ) { 
        let mut from: u16;
        let mut idx: usize;
        let mut attacks: u64;
        let mut captures: u64;
        let mut quiets: u64;
        while attackers > 0 {
            from = ls1b_scan(attackers);
            idx = from as usize;
            attacks = match PIECE {
                Piece::KNIGHT => KNIGHT_ATTACKS[idx],
                Piece::ROOK => rook_attacks(idx, self.occupied),
                Piece::BISHOP => bishop_attacks(idx, self.occupied),
                Piece::QUEEN => rook_attacks(idx, self.occupied) | bishop_attacks(idx, self.occupied),
                Piece::KING => KING_ATTACKS[idx],
                _ => panic!("Not a valid usize in fn piece_moves_general: {}", PIECE),
            };
            if IS_PINNED {
                attacks &= LINE_THROUGH[king_idx][idx];
            }
            if IN_CHECK {
                attacks &= blockers;
            }
            if TYPE == MoveType::QUIETS || TYPE == MoveType::ALL {
                quiets = attacks & !self.occupied;
                self.encode_move::<IS_KING, false>(move_list, quiets, from);
            }
            if TYPE == MoveType::CAPTURES || TYPE == MoveType::ALL {
                captures = attacks & self.sides[self.side_to_move ^ 1];
                self.encode_move::<IS_KING, true>(move_list, captures, from);
            }
            attackers &= attackers - 1;
        }
    }

    fn castles<const MOVETYPE: u8>(&self, move_list: &mut MoveList) {
        if MOVETYPE == MoveType::CAPTURES { return }
        match self.side_to_move {
            Side::WHITE => {
                if self.castle_rights & CastleRights::WHITE_QS > 0 && self.occupied & (B1 | C1 | D1) == 0
                    && !self.is_square_attacked(3, Side::WHITE, self.occupied) && !self.is_square_attacked(2, Side::WHITE, self.occupied) {
                    move_list.push(MoveFlags::QS_CASTLE | 2 << 6 | 4)
                }
                if self.castle_rights & CastleRights::WHITE_KS > 0 && self.occupied & (F1 | G1) == 0
                    && !self.is_square_attacked(5, Side::WHITE, self.occupied) && !self.is_square_attacked(6, Side::WHITE, self.occupied) {
                    move_list.push(MoveFlags::KS_CASTLE | 6 << 6 | 4)
                }
            }
            Side::BLACK => {
                if self.castle_rights & CastleRights::BLACK_QS > 0 && self.occupied & (B8 | C8 | D8) == 0
                    && !self.is_square_attacked(59, Side::BLACK, self.occupied) && !self.is_square_attacked(58, Side::BLACK, self.occupied) {
                    move_list.push(MoveFlags::QS_CASTLE | 58 << 6 | 60)
                }
                if self.castle_rights & CastleRights::BLACK_KS > 0 && self.occupied & (F8 | G8) == 0
                    && !self.is_square_attacked(61, Side::BLACK, self.occupied) && !self.is_square_attacked(62, Side::BLACK, self.occupied){
                    move_list.push(MoveFlags::KS_CASTLE | 62 << 6 | 60)
                }
            }
            _ => panic!("Invalid side for castling!"),
        }
    }

    fn pawn_moves<const IN_CHECK: bool, const MOVETYPE: u8>(&self, move_list: &mut MoveList, pinned: u64, king_idx: usize, checkers: u64, free: u64) {
        let unpinned_attackers = self.pieces[self.side_to_move][Piece::PAWN] & !pinned;
        let pinned_attackers = self.pieces[self.side_to_move][Piece::PAWN] & pinned;
        if MOVETYPE != MoveType::QUIETS && self.en_passant_sq > 0 {self.en_passants(move_list)}
        match self.side_to_move {
            Side::WHITE => {
                if MOVETYPE != MoveType::CAPTURES {
                    self.pawn_pushes_general::<{Side::WHITE}, IN_CHECK, false>(move_list, unpinned_attackers, free);
                    self.pawn_pushes_general::<{Side::WHITE}, IN_CHECK, true>(move_list, pinned_attackers, free);
                }
                if MOVETYPE != MoveType::QUIETS {
                    self.pawn_captures_general::<{Side::WHITE}, IN_CHECK, false>(move_list, checkers, unpinned_attackers, king_idx);
                    self.pawn_captures_general::<{Side::WHITE}, IN_CHECK, true>(move_list, checkers, pinned_attackers, king_idx);
                }
            },
            Side::BLACK => {
                if MOVETYPE != MoveType::CAPTURES {
                    self.pawn_pushes_general::<{Side::BLACK}, IN_CHECK, false>(move_list, unpinned_attackers, free);
                    self.pawn_pushes_general::<{Side::BLACK}, IN_CHECK, true>(move_list, pinned_attackers, free);
                }
                if MOVETYPE != MoveType::QUIETS {
                    self.pawn_captures_general::<{Side::BLACK}, IN_CHECK, false>(move_list, checkers, unpinned_attackers, king_idx);
                    self.pawn_captures_general::<{Side::BLACK}, IN_CHECK, true>(move_list, checkers, pinned_attackers, king_idx);
                }
            },
            _ => panic!("Invalid side in pawn_captures!")
        }
    }

    fn piece_moves<const PIECE: usize, const IN_CHECK: bool, const TYPE: u8>(&self, move_list: &mut MoveList, pinned: u64, king_idx: usize, blockers: u64) {
        let attackers = self.pieces[self.side_to_move][PIECE];
        self.piece_moves_general::<PIECE, IN_CHECK, false, false, TYPE>(move_list, king_idx, blockers, attackers & !pinned);
        self.piece_moves_general::<PIECE, IN_CHECK, true, false, TYPE>(move_list, king_idx, blockers, attackers & pinned);
    }

    // knight, bishop, rook, queen moves
    fn gen_pnbrq_moves<const IN_CHECK: bool, const MOVETYPE: u8>(&self, move_list: &mut MoveList, pinned: u64, king_idx: usize, blockers: u64, checks: u64, free: u64) {
        self.pawn_moves::<IN_CHECK, MOVETYPE>(move_list, pinned, king_idx, checks, free);
        self.piece_moves::<{ Piece::KNIGHT }, IN_CHECK, MOVETYPE>(move_list, pinned, king_idx, blockers);
        self.piece_moves::<{ Piece::BISHOP }, IN_CHECK, MOVETYPE>(move_list, pinned, king_idx, blockers);
        self.piece_moves::<{ Piece::ROOK }, IN_CHECK, MOVETYPE>(move_list, pinned, king_idx, blockers);
        self.piece_moves::<{ Piece::QUEEN }, IN_CHECK, MOVETYPE>(move_list, pinned, king_idx, blockers);
    }

    // for if you want to generate moves but not recalculate checks and pinned pieces (i.e generate checks then quiets)
    pub fn gen_moves_staged<const MOVETYPE: u8>(&self, move_list: &mut MoveList, check_status: Check, king_idx: usize, checks: u64, pinned: u64, king: u64) {
        self.piece_moves_general::<{Piece::KING}, false, false, true, MOVETYPE>(move_list, 0, 0, king);
        match check_status {
            Check::None => {
                self.gen_pnbrq_moves::<false, MOVETYPE>(move_list, pinned, king_idx, 0, 0, 0);
                self.castles::<MOVETYPE>(move_list);
            }
            Check::Single => {
                let idx = ls1b_scan(checks);
                let free = IN_BETWEEN[king_idx as usize][idx as usize];
                let blockers = free | checks;
                self.gen_pnbrq_moves::<true, MOVETYPE>(move_list, pinned, king_idx, blockers, checks, free);
            }
            Check::Double => {}
        }
    }

    pub fn gen_moves<const MOVETYPE: u8>(&self, move_list: &mut MoveList) {
        // Working out whether in check or not
        let king = self.pieces[self.side_to_move][Piece::KING];
        let king_idx = ls1b_scan(king) as usize;
        let (checks, pinned) = self.checkers_pinned_pieces(self.side_to_move, king_idx);
        let check_status = if checks == 0 {
            Check::None
        } else if checks & (checks - 1) > 0 {
            Check::Double
        } else {
            Check::Single
        };
        self.gen_moves_staged::<MOVETYPE>(move_list, check_status, king_idx, checks, pinned, king)
    }
}
