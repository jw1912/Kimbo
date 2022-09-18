// this deals with finding attacks for the rooks and bishops
// the idea is that these functions can be exposed for use other than move generation
// and can be changed to be more efficient without needing to mess with other code
use super::consts::*;
use super::{ls1b_scan, ms1b_scan, Piece, Position};

pub const fn rook_attacks(idx: usize, occupied: u64) -> u64 {
    let masks = MASKS[idx];
    let mut forward = occupied & masks.file;
    let mut reverse = forward.swap_bytes();
    forward -= masks.bitmask;
    reverse -= masks.bitmask.swap_bytes();
    forward ^= reverse.swap_bytes();
    forward &= masks.file;

    let mut easts = EAST[idx];
    let mut blocker = easts & occupied;
    let mut sq = ls1b_scan(blocker | MSB) as usize;
    easts ^= EAST[sq];

    let mut wests = WEST[idx];
    blocker = wests & occupied;
    sq = ms1b_scan(blocker | LSB) as usize;
    wests ^= WEST[sq];

    forward | easts | wests
}

pub const fn bishop_attacks(idx: usize, occ: u64) -> u64 {
    let masks = MASKS[idx];
    let mut forward = occ & masks.diag;
    let mut reverse = forward.swap_bytes();
    forward -= masks.bitmask;
    reverse -= masks.bitmask.swap_bytes();
    forward ^= reverse.swap_bytes();
    forward &= masks.diag;

    let mut forward2 = occ & masks.antidiag;
    let mut reverse2 = forward2.swap_bytes();
    forward2 -= masks.bitmask;
    reverse2 -= masks.bitmask.swap_bytes();
    forward2 ^= reverse2.swap_bytes();
    forward2 &= masks.antidiag;

    forward | forward2
}

#[inline(always)]
pub fn xray_rook_attacks(occupied: u64, blockers: u64, idx: usize) -> u64 {
    let attacks = rook_attacks(idx, occupied);
    let blockers2 = blockers & attacks;
    attacks ^ rook_attacks(idx, occupied ^ blockers2)
}
#[inline(always)]
pub fn xray_bishop_attacks(occupied: u64, blockers: u64, idx: usize) -> u64 {
    let attacks = bishop_attacks(idx, occupied);
    let blockers2 = blockers & attacks;
    attacks ^ bishop_attacks(idx, occupied ^ blockers2)
}

impl Position {
    /// Returns a bitboard of attackers to the given square, considering occupancy
    #[inline(always)]
    pub fn attacks_to_square(&self, idx: usize, side: usize, occupied: u64) -> u64 {
        let other = side ^ 1;
        // all opposing pieces
        let op_pawns = self.pieces[other][Piece::PAWN];
        let op_knights = self.pieces[other][Piece::KNIGHT];
        let op_queen = self.pieces[other][Piece::QUEEN];
        let op_rooks_and_queen = self.pieces[other][Piece::ROOK] | op_queen;
        let op_bishops_and_queen = self.pieces[other][Piece::BISHOP] | op_queen;
        let op_king = self.pieces[other][Piece::KING];
        // all attackers
        (PAWN_ATTACKS[side][idx] & op_pawns)
            | (KNIGHT_ATTACKS[idx] & op_knights)
            | (bishop_attacks(idx, occupied) & op_bishops_and_queen)
            | (rook_attacks(idx, occupied) & op_rooks_and_queen)
            | (KING_ATTACKS[idx] & op_king)
    }

    /// Returns whether a given square is occupied, considering occupancy
    #[inline(always)]
    pub fn is_square_attacked(&self, idx: usize, side: usize, occupied: u64) -> bool {
        let other = side ^ 1;
        // all opposing pieces
        let op_pawns = self.pieces[other][Piece::PAWN];
        let op_knights = self.pieces[other][Piece::KNIGHT];
        let op_queen = self.pieces[other][Piece::QUEEN];
        let op_rooks_and_queen = self.pieces[other][Piece::ROOK] | op_queen;
        let op_bishops_and_queen = self.pieces[other][Piece::BISHOP] | op_queen;
        let op_king = self.pieces[other][Piece::KING];
        // all attackers
        (KNIGHT_ATTACKS[idx] & op_knights > 0)
        || (KING_ATTACKS[idx] & op_king > 0)
        || (PAWN_ATTACKS[side][idx] & op_pawns > 0)
        || (rook_attacks(idx, occupied) & op_rooks_and_queen > 0)
        || (bishop_attacks(idx, occupied) & op_bishops_and_queen > 0)
            
    }

    pub fn checkers_pinned_pieces(&self, side: usize, king_idx: usize) -> (u64, u64) {
        let other = side ^ 1;
        let op_queen = self.pieces[other][Piece::QUEEN];
        let op_rooks_and_queen = self.pieces[other][Piece::ROOK] | op_queen;
        let op_bishops_and_queen = self.pieces[other][Piece::BISHOP] | op_queen;
        let checkers = (PAWN_ATTACKS[side][king_idx] & self.pieces[other][Piece::PAWN])
            | (KNIGHT_ATTACKS[king_idx] & self.pieces[other][Piece::KNIGHT])
            | (bishop_attacks(king_idx, self.occupied) & op_bishops_and_queen)
            | (rook_attacks(king_idx, self.occupied) & op_rooks_and_queen);
        let own_usizes = self.sides[side];
        let mut pinned = 0;
        let mut pinner = xray_rook_attacks(self.occupied, own_usizes, king_idx) & op_rooks_and_queen;
        let mut sq: usize;
        while pinner > 0 {
            sq = ls1b_scan(pinner) as usize;
            pinned |= IN_BETWEEN[sq][king_idx] & own_usizes;
            pinner &= pinner - 1;
        }
        pinner = xray_bishop_attacks(self.occupied, own_usizes, king_idx) & op_bishops_and_queen;
        while pinner > 0 {
            sq = ls1b_scan(pinner) as usize;
            pinned |= IN_BETWEEN[sq][king_idx] & own_usizes;
            pinner &= pinner - 1;
        }
        (checkers, pinned)
    }
}
