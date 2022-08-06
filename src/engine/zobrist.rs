use kimbo_state::{ls1b_scan, Position};
use fastrand;

pub struct ZobristVals {
    pieces: [[[u64; 64]; 6]; 2],
    castle: [u64; 4],
    en_passant: [u64; 8],
    side: u64,
}

impl ZobristVals {
    pub fn piece_hash(&self, idx: usize, side: usize, piece: usize) -> u64 {
        self.pieces[side][piece][idx]
    }
    pub fn castle_hash(&self, current: u8, update: u8) -> u64 {
        if current & update == 0 {
            return 0
        }
        self.castle[ls1b_scan(update as u64) as usize]
    }
    pub fn en_passant_hash(&self, file: usize) -> u64 {
        self.en_passant[file]
    }
    pub fn side_hash(&self) -> u64 {
        self.side
    }
}

impl Default for ZobristVals {
    fn default() -> Self {
        let mut vals = Self {
            pieces: [[[0; 64]; 6]; 2],
            castle: [0; 4],
            en_passant: [0; 8],
            side: 0,
        };
        fastrand::seed(353012);
        for color in 0..2 {
            for piece in 0..6 {
                for field_idx in 0..64 {
                    vals.pieces[color][piece][field_idx] = fastrand::u64(1..u64::MAX);
                }
            }
        }

        for castle_idx in 0..4 {
            vals.castle[castle_idx] = fastrand::u64(1..u64::MAX);
        }

        for enp_idx in 0..8 {
            vals.en_passant[enp_idx] = fastrand::u64(1..u64::MAX);
        }
        vals
    }
}

pub fn init_zobrist(pos: &Position, zvals: &ZobristVals) -> u64 {
    let mut zobrist = 0;
    for (i, side) in pos.pieces.iter().enumerate() {
        for (j, &pc) in side.iter().enumerate() {
            let mut piece = pc;
            while piece > 0 {
                let idx = ls1b_scan(piece) as usize;
                zobrist ^= zvals.piece_hash(idx, i, j);
                piece &= piece - 1
            }
        }
    }
    let mut castle_rights = pos.castle_rights;
    while castle_rights > 0 {
        let ls1b = castle_rights & castle_rights.wrapping_neg();
        zobrist ^= zvals.castle_hash(0b1111, ls1b);
        castle_rights &= castle_rights - 1
    }
    if pos.en_passant_sq > 0 {
        zobrist ^= zvals.en_passant_hash((pos.en_passant_sq & 7) as usize);
    }
    zobrist
}