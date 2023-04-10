use super::{consts::*, MoveList};
use crate::bitloop;

#[inline]
fn encode<const FLAG: u16>(moves: &mut MoveList, mut attacks: u64, from: u16) {
    let mov = from << 6 | FLAG;
    bitloop!(attacks, to, moves.add(mov | to))
}

pub(super) fn piece_moves<const PC: usize, const QUIETS: bool>(
    moves: &mut MoveList,
    occ: u64,
    opps: u64,
    mut attackers: u64,
) {
    bitloop!(attackers, from, {
        let f = from as usize;
        let attacks = match PC {
            Piece::KNIGHT => Attacks::KNIGHT[f],
            Piece::ROOK => Attacks::rook(f, occ),
            Piece::BISHOP => Attacks::bishop(f, occ),
            Piece::QUEEN => Attacks::rook(f, occ) | Attacks::bishop(f, occ),
            Piece::KING => Attacks::KING[f],
            _ => 0,
        };
        encode::<{ MoveFlag::CAPTURE }>(moves, attacks & opps, from);
        if QUIETS {
            encode::<{ MoveFlag::QUIET }>(moves, attacks & !occ, from);
        }
    });
}

pub(super) fn pawn_captures(moves: &mut MoveList, mut attackers: u64, opps: u64, c: usize) {
    let mut promo = attackers & Rank::PENULTIMATE[c];
    attackers &= !Rank::PENULTIMATE[c];
    bitloop!(
        attackers,
        from,
        encode::<{ MoveFlag::CAPTURE }>(moves, Attacks::PAWN[c][from as usize] & opps, from)
    );
    bitloop!(promo, from, {
        let mut attacks = Attacks::PAWN[c][from as usize] & opps;
        bitloop!(attacks, to, {
            let f = from << 6;
            moves.add(f | to | MoveFlag::KNIGHT_PROMO_CAPTURE);
            moves.add(f | to | MoveFlag::BISHOP_PROMO_CAPTURE);
            moves.add(f | to | MoveFlag::ROOK_PROMO_CAPTURE);
            moves.add(f | to | MoveFlag::QUEEN_PROMO_CAPTURE);
        });
    });
}

pub(super) fn en_passants(moves: &mut MoveList, pawns: u64, sq: u16, c: usize) {
    let mut attackers = Attacks::PAWN[c ^ 1][sq as usize] & pawns;
    bitloop!(
        attackers,
        from,
        moves.add(from << 6 | sq | MoveFlag::EN_PASSANT)
    )
}

fn shift<const SIDE: usize>(bb: u64) -> u64 {
    if SIDE == Side::WHITE {
        bb >> 8
    } else {
        bb << 8
    }
}

fn idx_shift<const SIDE: usize, const AMOUNT: u16>(idx: u16) -> u16 {
    if SIDE == Side::WHITE {
        idx + AMOUNT
    } else {
        idx - AMOUNT
    }
}

pub(super) fn pawn_pushes<const SIDE: usize>(moves: &mut MoveList, occ: u64, pawns: u64) {
    let empty = !occ;
    let mut dbl = shift::<SIDE>(shift::<SIDE>(empty & Rank::DOUBLE[SIDE]) & empty) & pawns;
    let mut push = shift::<SIDE>(empty) & pawns;
    let mut promo = push & Rank::PENULTIMATE[SIDE];
    push &= !Rank::PENULTIMATE[SIDE];
    bitloop!(
        push,
        from,
        moves.add(from << 6 | idx_shift::<SIDE, 8>(from) | MoveFlag::QUIET)
    );
    bitloop!(promo, from, {
        let f = from << 6;
        let to = idx_shift::<SIDE, 8>(from);
        moves.add(f | to | MoveFlag::KNIGHT_PROMO);
        moves.add(f | to | MoveFlag::BISHOP_PROMO);
        moves.add(f | to | MoveFlag::ROOK_PROMO);
        moves.add(f | to | MoveFlag::QUEEN_PROMO);
    });
    bitloop!(
        dbl,
        from,
        moves.add(from << 6 | idx_shift::<SIDE, 16>(from) | MoveFlag::DBL_PUSH)
    );
}
