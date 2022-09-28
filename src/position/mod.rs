pub mod attacks;
pub mod consts;
pub mod makemove;
pub mod movegen;
pub mod zobrist;
pub mod perft;

use self::zobrist::ZobristVals;
use std::sync::Arc;
use std::{mem, ops::{Index, IndexMut}};
use std::ptr;


/// Position struct
/// Stores the same info as a fen string in bitboard format, with auxiliary info .squares and .sides
#[derive(Clone)]
pub struct Position {
    pub pieces: [[u64; 6]; 2],
    pub sides: [u64; 2],
    pub occupied: u64,
    pub squares: [u8; 64],
    pub side_to_move: usize,
    // gamestate
    pub castle_rights: u8,
    pub en_passant_sq: u16,
    pub halfmove_clock: u8,
    pub fullmove_counter: u16,
    pub zobrist: u64,
    pub pawnhash: u64,
    // heap allocated
    pub state_stack: Vec<GameState>, 
    pub zobrist_vals: Arc<ZobristVals>,
    /// Eval stuff
    pub mat_mg: [i16; 2],
    pub mat_eg: [i16; 2],
    pub pst_mg: [i16; 2],
    pub pst_eg: [i16; 2],
    pub phase: i16,
    pub null_counter: u8, // for draw detection, null moves don't count
}

/// Extended move context for incrementally updated eval fields
#[derive(Clone, Copy)]
pub struct GameState {
    pub m: u16,
    pub moved_pc: u8,
    pub captured_pc: u8,
    pub castle_rights: u8,
    pub halfmove_clock: u8,
    pub en_passant_sq: u16,
    pub phase: i16,
    pub zobrist: u64,
    pub pawnhash: u64,
    pub mat_mg: [i16; 2],
    pub mat_eg: [i16; 2],
    pub pst_mg: [i16; 2],
    pub pst_eg: [i16; 2],
}

#[derive(Clone, Debug)]
pub struct MoveList {
    /// Most moves from a position is 218
    list: [u16; 255],
    /// Length of list utilised currently
    len: usize,
}

impl Default for Position {
    fn default() -> Self {
        Self::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", Arc::new(ZobristVals::default())).unwrap()
    }
}

/// For indexing into position.pieces and position.sides
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

pub struct CastleRights;
impl CastleRights {
    pub const ALL: u8 = 15;
    pub const WHITE_QS: u8 = 8;
    pub const WHITE_KS: u8 = 4;
    pub const BLACK_QS: u8 = 2;
    pub const BLACK_KS: u8 = 1;
    pub const SIDES: [u8; 2] = [
        Self::WHITE_KS | Self::WHITE_QS,
        Self::BLACK_KS | Self::BLACK_QS,
    ];
    pub const NONE: u8 = 0;
}

#[inline(always)]
pub const fn ls1b_scan(bb: u64) -> u16 {
    bb.trailing_zeros() as u16
}
#[inline(always)]
pub const fn ms1b_scan(bb: u64) -> u16 {
    63 ^ bb.leading_zeros() as u16
}

pub fn bitboard_out(bb: &u64) {
    let bytes = &bb.to_be_bytes();
    for byte in bytes {
        println!("{:08b}", byte.reverse_bits());
    }
}

/*  Move Encoding
    Moves are encoded into a u16, in the following format:
    MSB <- 0000        000000     000000   -> LSB
           move flags  to index   from index
*/
pub struct MoveFlags;
impl MoveFlags {
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

/// Types of check
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Check {
    None,
    Single,
    Double,
}

#[derive(PartialEq, Eq)]
pub struct MoveType;
impl MoveType {
    pub const ALL: u8 = 0;
    pub const CAPTURES: u8 = 1;
    pub const QUIETS: u8 = 2;
}

impl Default for MoveList {
    fn default() -> Self {
        Self {
            list: unsafe {
                #[allow(clippy::uninit_assumed_init)]
                mem::MaybeUninit::uninit().assume_init()
            },
            len: 0,
        } 
    }
}

impl MoveList {
    pub const fn new(list: [u16; 255], len: usize) -> Self {
        Self { list, len }
    }
    #[inline(always)]
    pub fn push(&mut self, m: u16) {
        self.list[self.len] = m;
        self.len += 1;
    }
    #[inline(always)]
    pub fn clear(&mut self) {
        self.len = 0;
    }
    #[inline(always)]
    pub const fn len(&self) -> usize {
        self.len
    }
    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }
    #[inline(always)]
    pub fn swap_unchecked(&mut self, i: usize, j: usize) {
        let ptr = self.list.as_mut_ptr();
        unsafe {
            ptr::swap(ptr.add(i), ptr.add(j));
        }
    }
}

impl Index<usize> for MoveList {
    type Output = u16;
    fn index(&self, index: usize) -> &Self::Output {
        &self.list[index]
    }
}
impl IndexMut<usize> for MoveList {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.list[index]
    }
}
