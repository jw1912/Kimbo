// ZobristVals struct copied from Inanis
// Inanis: https://github.com/Tearth/Inanis/blob/master/src/state/zobrist.rs
// simply because this is the only way to do it
// initialise_zobrist function is my own

use fastrand;
use super::{ls1b_scan, Position};

pub struct ZobristVals {
    pieces: [[[u64; 64]; 6]; 2],
    castle: [u64; 4],
    en_passant: [u64; 8],
    side: u64,
}

impl ZobristVals {
    #[inline(always)]
    pub fn piece_hash(&self, idx: usize, side: usize, piece: usize) -> u64 {
        self.pieces[side][piece][idx]
    }
    #[inline(always)]
    pub fn castle_hash(&self, current: u8, update: u8) -> u64 {
        if current & update == 0 {
            return 0;
        }
        self.castle[ls1b_scan(update as u64) as usize]
    }
    #[inline(always)]
    pub fn en_passant_hash(&self, file: usize) -> u64 {
        self.en_passant[file]
    }
    #[inline(always)]
    pub fn side_hash(&self) -> u64 {
        self.side
    }
}

impl Default for ZobristVals {
    fn default() -> Self {
        fastrand::seed(353012);
        
        let mut vals = Self {
            pieces: [[[0; 64]; 6]; 2],
            castle: [0; 4],
            en_passant: [0; 8],
            side: fastrand::u64(1..u64::MAX),
        };
        
        for color in 0..2 {
            for piece in 0..6 {
                for sq_idx in 0..64 {
                    vals.pieces[color][piece][sq_idx] = fastrand::u64(1..u64::MAX);
                }
            }
        }

        for idx in 0..4 {
            vals.castle[idx] = fastrand::u64(1..u64::MAX);
        }

        for idx in 0..8 {
            vals.en_passant[idx] = fastrand::u64(1..u64::MAX);
        }

        vals
    }
}

pub fn initialise_zobrist(pos: &Position) -> u64 {
    let mut zobrist = 0;
    for (i, side) in pos.pieces.iter().enumerate() {
        for (j, &pc) in side.iter().enumerate() {
            let mut piece = pc;
            while piece > 0 {
                let idx = ls1b_scan(piece) as usize;
                zobrist ^= pos.zobrist_vals.piece_hash(idx, i, j);
                piece &= piece - 1
            }
        }
    }
    let mut castle_rights = pos.castle_rights;
    while castle_rights > 0 {
        let ls1b = castle_rights & castle_rights.wrapping_neg();
        zobrist ^= pos.zobrist_vals.castle_hash(0b1111, ls1b);
        castle_rights &= castle_rights - 1
    }
    if pos.en_passant_sq > 0 {
        zobrist ^= pos.zobrist_vals.en_passant_hash((pos.en_passant_sq & 7) as usize);
    }
    if pos.side_to_move == 0 {
        zobrist ^= pos.zobrist_vals.side_hash();
    }
    zobrist
}

pub fn initialise_pawnhash(pos: &Position) -> u64 {
    let mut hash = 0;
    for side in 0..2 {
        for pc in [0, 5] {
            let mut piece = pos.pieces[side][pc];
            while piece > 0 {
                let idx = ls1b_scan(piece) as usize;
                hash ^= pos.zobrist_vals.piece_hash(idx, side, pc);
                piece &= piece - 1
            }
        }
    }
    hash
}
