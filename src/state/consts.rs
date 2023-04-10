use crate::init;

pub struct Side;
impl Side {
    pub const WHITE: usize = 0;
    pub const BLACK: usize = 1;
}

pub struct Piece;
impl Piece {
    pub const PAWN: usize = 0;
    pub const KNIGHT: usize = 1;
    pub const BISHOP: usize = 2;
    pub const ROOK: usize = 3;
    pub const QUEEN: usize = 4;
    pub const KING: usize = 5;
    pub const NONE: usize = 6;
}

pub struct Squares;
impl Squares {
    pub const LIGHT: u64 = 0x55AA55AA55AA55AA;
    pub const DARK: u64 = 0xAA55AA55AA55AA55;
}

// temporary stuff , removed when add FRC support
pub const B1C1D1: u64 = 0xE;
pub const F1G1: u64 = 0x60;
pub const B8C8D8: u64 = 0xE00000000000000;
pub const F8G8: u64 = 0x6000000000000000;
pub const CM: [[(u64, usize, usize); 2]; 2] = [
    [(9, 0, 3), (160, 7, 5)],
    [(0x900000000000000, 56, 59), (0xA000000000000000, 63, 61)],
];

pub struct CastleRights;
impl CastleRights {
    pub const NONE: u8 = 0;
    pub const WHITE_QS: u8 = 8;
    pub const WHITE_KS: u8 = 4;
    pub const BLACK_QS: u8 = 2;
    pub const BLACK_KS: u8 = 1;
    pub const SIDES: [u8; 2] = [
        Self::WHITE_KS | Self::WHITE_QS,
        Self::BLACK_KS | Self::BLACK_QS,
    ];
}

pub struct MoveFlag;
impl MoveFlag {
    pub const ALL: u16 = 15 << 12;
    // main flags
    pub const QUIET: u16 = 0 << 12;
    pub const DBL_PUSH: u16 = 1 << 12;
    pub const KS_CASTLE: u16 = 2 << 12;
    pub const QS_CASTLE: u16 = 3 << 12;
    pub const CAPTURE: u16 = 4 << 12;
    pub const EN_PASSANT: u16 = 5 << 12;
    // promotion options
    pub const KNIGHT_PROMO: u16 = 8 << 12;
    pub const BISHOP_PROMO: u16 = 9 << 12;
    pub const ROOK_PROMO: u16 = 10 << 12;
    pub const QUEEN_PROMO: u16 = 11 << 12;
    // capture promotion options
    pub const KNIGHT_PROMO_CAPTURE: u16 = 12 << 12;
    pub const BISHOP_PROMO_CAPTURE: u16 = 13 << 12;
    pub const ROOK_PROMO_CAPTURE: u16 = 14 << 12;
    pub const QUEEN_PROMO_CAPTURE: u16 = 15 << 12;
}

// Movegen
pub struct Rank;
impl Rank {
    pub const PENULTIMATE: [u64; 2] = [0xFF000000000000, 0xFF00];
    pub const DOUBLE: [u64; 2] = [0xFF000000000000, 0xFF00];
}

pub struct File;
impl File {
    pub const A: u64 = 0x101010101010101;
    pub const H: u64 = Self::A << 7;
}

#[derive(Clone, Copy)]
pub struct Mask {
    pub bit: u64,
    pub right: u64,
    pub left: u64,
    pub file: u64,
}

pub struct Attacks;
impl Attacks {
    pub const PAWN: [[u64; 64]; 2] = [
        init!(i, {
            (((1 << i) & !File::A) << 7) | (((1 << i) & !File::H) << 9)
        }),
        init!(i, {
            (((1 << i) & !File::A) >> 9) | (((1 << i) & !File::H) >> 7)
        }),
    ];

    pub const KNIGHT: [u64; 64] = init!(i, {
        let n = 1 << i;
        let h1 = ((n >> 1) & 0x7f7f7f7f7f7f7f7f) | ((n << 1) & 0xfefefefefefefefe);
        let h2 = ((n >> 2) & 0x3f3f3f3f3f3f3f3f) | ((n << 2) & 0xfcfcfcfcfcfcfcfc);
        (h1 << 16) | (h1 >> 16) | (h2 << 8) | (h2 >> 8)
    });

    pub const KING: [u64; 64] = init!(i, {
        let mut k = 1 << i;
        k |= (k << 8) | (k >> 8);
        k |= ((k & !File::A) >> 1) | ((k & !File::H) << 1);
        k ^ (1 << i)
    });

    const DIAGS: [u64; 15] = [
        0x0100000000000000,
        0x0201000000000000,
        0x0402010000000000,
        0x0804020100000000,
        0x1008040201000000,
        0x2010080402010000,
        0x4020100804020100,
        0x8040201008040201,
        0x0080402010080402,
        0x0000804020100804,
        0x0000008040201008,
        0x0000000080402010,
        0x0000000000804020,
        0x0000000000008040,
        0x0000000000000080,
    ];

    const WEST: [u64; 64] = init!(i, ((1 << i) - 1) & (0xFF << (i & 56)));

    const BISHOP_MASK: [Mask; 64] = init!(i,
        let bit = 1 << i;
        Mask {
            bit,
            right: bit ^ Self::DIAGS[(7 + (i & 7) - i / 8)],
            left: bit ^ Self::DIAGS[((i & 7) + i / 8)].swap_bytes(),
            file: bit.swap_bytes(),
        }
    );

    const ROOK_MASK: [Mask; 64] = init!(i,
        let bit = 1 << i;
        let left = (bit - 1) & (0xFF << (i & 56));
        Mask {
            bit,
            right: bit ^ left ^ (0xFF << (i & 56)),
            left,
            file: bit ^ File::A << (i & 7),
        }
    );

    pub const fn bishop(idx: usize, occ: u64) -> u64 {
        let mask = Self::BISHOP_MASK[idx];

        let mut diag = occ & mask.right;
        let mut rev = diag.swap_bytes();
        diag = diag.wrapping_sub(mask.bit);
        rev = rev.wrapping_sub(mask.file);
        diag ^= rev.swap_bytes();

        let mut anti = occ & mask.left;
        rev = anti.swap_bytes();
        anti = anti.wrapping_sub(mask.bit);
        rev = rev.wrapping_sub(mask.file);
        anti ^= rev.swap_bytes();

        (diag & mask.right) | (anti & mask.left)
    }

    pub const fn rook(idx: usize, occ: u64) -> u64 {
        let mask = Self::ROOK_MASK[idx];

        let mut file = occ & mask.file;
        let mut rev = file.swap_bytes();
        file = file.wrapping_sub(mask.bit);
        rev = rev.wrapping_sub(mask.bit.swap_bytes());
        file ^= rev.swap_bytes();

        let mut east = occ & mask.right;
        rev = east & east.wrapping_neg();
        east = rev ^ (rev - mask.bit);

        let west = Self::WEST[(((mask.left & occ) | 1).leading_zeros() ^ 63) as usize];

        (file & mask.file) | (east & mask.right) | (west ^ mask.left)
    }
}
