use crate::bitloop;

use super::{consts::*, movegen::*, square_str_to_index, MoveList};
use std::{cmp, str::FromStr};

/// State that is copied for undoing moves.
#[derive(Clone, Copy, Default)]
struct State {
    castle_rights: u8,
    en_passant: u8,
    halfmove_clock: u8,
    zobrist: u64,
}

/// Required info to undo a move.
#[derive(Clone, Copy)]
struct MoveContext {
    r#move: u16,
    moved: u8,
    captured: u8,
    state: State,
}

struct Castle {
    mask: [u8; 64],
    rooks: [u8; 2],
    chess960: bool,
}

/// Holds all random values for hashing.
struct ZobristVals {
    pieces: [[[u64; 64]; 6]; 2],
    castling: [u64; 4],
    en_passant: [u64; 8],
    side: u64,
}

/// Holds all info abut current position.
pub struct Position {
    pieces: [u64; 6],
    sides: [u64; 2],
    stm: bool,
    state: State,
    phase: i16,
    null_counter: u8,
    stack: Vec<MoveContext>,
    zobrist_vals: ZobristVals,
    castle: Castle,
}

fn random(seed: &mut u64) -> u64 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    *seed
}

impl Default for ZobristVals {
    fn default() -> Self {
        let mut seed = 180_620_142;
        let mut vals = Self {
            pieces: [[[0; 64]; 6]; 2],
            castling: [0; 4],
            en_passant: [0; 8],
            side: random(&mut seed),
        };

        for i in 0..2 {
            for j in 0..6 {
                for k in 0..64 {
                    vals.pieces[i][j][k] = random(&mut seed);
                }
            }
        }

        for i in 0..4 {
            vals.castling[i] = random(&mut seed);
        }

        for i in 0..8 {
            vals.en_passant[i] = random(&mut seed);
        }

        vals
    }
}

impl FromStr for Position {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split = s.split_whitespace().collect::<Vec<&str>>();
        let mut pos = Position {
            pieces: [0; 6],
            sides: [0; 2],
            stm: false,
            state: State::default(),
            phase: 0,
            null_counter: 0,
            stack: Vec::new(),
            zobrist_vals: ZobristVals::default(),
            castle: Castle {
                mask: [15; 64],
                rooks: [0, 7],
                chess960: false,
            },
        };

        // main part of fen
        let board_str = split.get(0).ok_or("empty string")?;
        let mut col = 0;
        let mut row = 7;
        for ch in board_str.chars() {
            if ch == '/' {
                row -= 1;
                col = 0;
            } else if let Ok(clear) = ch.to_string().parse::<i16>() {
                if !(1..=8).contains(&clear) {
                    return Err(String::from("invalid number of empty squares"));
                }
                col += clear;
            } else {
                let idx = ['P', 'N', 'B', 'R', 'Q', 'K', 'p', 'n', 'b', 'r', 'q', 'k']
                    .iter()
                    .position(|&el| el == ch)
                    .ok_or("invalid letter in fen")?;
                let side = usize::from(idx > 5);
                let piece = idx - 6 * side;
                let sq = 8 * row + col;
                pos.toggle(side, piece, 1 << sq);
                col += 1;
            }
        }

        // state info
        let stm_str = split.get(1).ok_or("no side to move provided")?;
        pos.stm = stm_str == &"b";
        let en_passant_str = split.get(3).ok_or("no en passant square provided")?;
        pos.state.en_passant = if en_passant_str == &"-" {
            0
        } else {
            square_str_to_index(en_passant_str)? as u8
        };
        let halfmove_str = split.get(4).unwrap_or(&"0");
        pos.state.halfmove_clock = halfmove_str.parse::<u8>().unwrap();

        // castling
        let castle_str = split.get(2).ok_or("no castling rights provided")?;
        let mut rights = 0;
        let mut king_col = 4;
        for ch in castle_str.chars() {
            rights |= match ch as u8 {
                b'Q' => CastleRights::WHITE_QS,
                b'K' => CastleRights::WHITE_KS,
                b'q' => CastleRights::BLACK_QS,
                b'k' => CastleRights::BLACK_KS,
                b'A'..=b'H' => {
                    pos.castle.chess960 = true;
                    let wkc = pos.piece(Side::WHITE, Piece::KING).trailing_zeros() as u8 & 7;
                    king_col = wkc as usize;
                    let rook_col = ch as u8 - b'A';
                    if rook_col < wkc {
                        pos.castle.rooks[0] = rook_col;
                        CastleRights::WHITE_QS
                    } else {
                        pos.castle.rooks[1] = rook_col;
                        CastleRights::WHITE_KS
                    }
                }
                b'a'..=b'h' => {
                    pos.castle.chess960 = true;
                    let bkc = pos.piece(Side::BLACK, Piece::KING).trailing_zeros() as u8 & 7;
                    king_col = bkc as usize;
                    let rook_col = ch as u8 - b'a';
                    if rook_col < bkc {
                        pos.castle.rooks[0] = rook_col;
                        CastleRights::BLACK_QS
                    } else {
                        pos.castle.rooks[1] = rook_col;
                        CastleRights::BLACK_KS
                    }
                }
                _ => 0,
            };
        }
        pos.state.castle_rights = rights;
        pos.castle.mask[usize::from(pos.castle.rooks[0])] = 7;
        pos.castle.mask[usize::from(pos.castle.rooks[1])] = 11;
        pos.castle.mask[56 + usize::from(pos.castle.rooks[0])] = 13;
        pos.castle.mask[56 + usize::from(pos.castle.rooks[1])] = 14;
        pos.castle.mask[king_col] = 3;
        pos.castle.mask[56 + king_col] = 12;

        Ok(pos)
    }
}

impl Default for Position {
    fn default() -> Self {
        Fens::STARTPOS.parse().expect("hard coded")
    }
}

impl Position {
    /// Adds/removes a piece from the position bitboards.
    #[inline]
    fn toggle(&mut self, side: usize, piece: usize, bit: u64) {
        self.pieces[piece] ^= bit;
        self.sides[side] ^= bit;
    }

    /// Calculates occupancy bitboard.
    #[inline]
    pub fn occ(&self) -> u64 {
        self.sides[Side::WHITE] | self.sides[Side::BLACK]
    }

    /// Return a bitboard of a piece of given colour.
    #[inline]
    pub fn piece(&self, side: usize, piece: usize) -> u64 {
        self.sides[side] & self.pieces[piece]
    }

    /// Returns a bitboard of given side uccupancy.
    #[inline]
    pub fn side(&self, side: usize) -> u64 {
        self.sides[side]
    }

    /// Getter for if chess960.
    pub fn chess960(&self) -> bool {
        self.castle.chess960
    }

    /// Getter for if chess960.
    pub fn stm(&self) -> usize {
        usize::from(self.stm)
    }

    /// Getter for castling rook files.
    pub fn castling_rooks(&self) -> [u8; 2] {
        self.castle.rooks
    }

    /// Determines if the side to move is in check.
    pub fn in_check(&self) -> bool {
        let side = usize::from(self.stm);
        let kidx = self.piece(side, Piece::KING).trailing_zeros() as usize;
        self.is_square_attacked(kidx, side, self.occ())
    }

    /// Checks if given square is attacked by opposite side.
    #[inline]
    pub fn is_square_attacked(&self, idx: usize, side: usize, occ: u64) -> bool {
        let opps = self.sides[side ^ 1];
        (Attacks::KNIGHT[idx] & opps & self.pieces[Piece::KNIGHT] > 0)
            || (Attacks::KING[idx] & opps & self.pieces[Piece::KING] > 0)
            || (Attacks::PAWN[side][idx] & opps & self.pieces[Piece::PAWN] > 0)
            || (Attacks::bishop(idx, occ)
                & opps
                & (self.pieces[Piece::BISHOP] | self.pieces[Piece::QUEEN])
                > 0)
            || (Attacks::rook(idx, occ)
                & opps
                & (self.pieces[Piece::ROOK] | self.pieces[Piece::QUEEN])
                > 0)
    }

    /// Finds the piece at a given index.
    #[inline]
    pub fn get_piece(&self, bit: u64, occ: u64) -> usize {
        6 * usize::from(occ & bit == 0)
            + usize::from(
                (self.pieces[Piece::KNIGHT] | self.pieces[Piece::ROOK] | self.pieces[Piece::KING])
                    & bit
                    > 0,
            )
            + 2 * usize::from((self.pieces[Piece::BISHOP] | self.pieces[Piece::ROOK]) & bit > 0)
            + 4 * usize::from((self.pieces[Piece::QUEEN] | self.pieces[Piece::KING]) & bit > 0)
    }

    /// Makes a move, returns if it was legal.
    pub fn r#do(&mut self, r#move: u16) -> bool {
        // determine moved and captured pieces
        let flag = r#move & MoveFlag::ALL;
        let from = (r#move >> 6) & 63;
        let to = r#move & 63;
        let occ = self.occ();
        let moved = self.get_piece(1 << from, occ);
        let captured = if flag & MoveFlag::CAPTURE == 0 || flag == MoveFlag::EN_PASSANT {
            Piece::NONE
        } else {
            self.get_piece(1 << to, occ)
        };

        // make move
        let side = usize::from(self.stm);
        self.stack.push(MoveContext {
            r#move,
            moved: moved as u8,
            captured: captured as u8,
            state: self.state,
        });
        self.state.castle_rights &=
            self.castle.mask[usize::from(to)] & self.castle.mask[usize::from(from)];
        self.state.en_passant = if flag == MoveFlag::DBL_PUSH {
            (if side == Side::WHITE { to - 8 } else { to + 8 }) as u8
        } else {
            0
        };
        self.state.halfmove_clock = if moved > Piece::PAWN && flag != MoveFlag::CAPTURE {
            self.state.halfmove_clock + 1
        } else {
            0
        };
        self.r#move(r#move, side, moved, captured);

        // checking if legal
        let kidx = self.piece(side, Piece::KING).trailing_zeros() as usize;
        let invalid = self.is_square_attacked(kidx, side, self.occ());
        if invalid {
            self.undo()
        }

        invalid
    }

    pub fn undo(&mut self) {
        let MoveContext {
            r#move,
            moved,
            captured,
            state,
        } = self.stack.pop().unwrap();
        self.state = state;
        self.r#move(
            r#move,
            usize::from(!self.stm),
            usize::from(moved),
            usize::from(captured),
        );
    }

    /// Common do-undo fuctionality.
    #[inline]
    fn r#move(&mut self, r#move: u16, side: usize, moved: usize, captured: usize) {
        // extract move info
        let from = (r#move >> 6) & 63;
        let to = r#move & 63;
        let flag = r#move & MoveFlag::ALL;
        let from_bit = 1 << from;
        let to_bit = 1 << to;

        // basic updates
        self.stm = !self.stm;
        self.toggle(side, moved, from_bit ^ to_bit);
        if captured != Piece::NONE {
            self.toggle(side ^ 1, captured, to_bit)
        }

        // updates for more complex moves
        match flag {
            MoveFlag::KS_CASTLE | MoveFlag::QS_CASTLE => {
                let (bits, _, _) = CM[side][usize::from(flag == MoveFlag::KS_CASTLE)];
                self.toggle(side, Piece::ROOK, bits);
            }
            MoveFlag::EN_PASSANT => {
                let pawn_idx = usize::from(to + [8u16.wrapping_neg(), 8][side]);
                self.toggle(side ^ 1, Piece::PAWN, 1 << pawn_idx);
            }
            MoveFlag::KNIGHT_PROMO.. => {
                let promo = usize::from(((flag >> 12) & 3) + 1);
                self.pieces[Piece::PAWN] ^= to_bit;
                self.pieces[promo] ^= to_bit;
            }
            _ => {}
        }
    }

    /// Generates all pseudo-legal moves in a given position.
    pub fn generate<const QUIET: bool>(&self) -> MoveList {
        let mut moves = MoveList::uninit();
        let mref = &mut moves;

        // useful bitboards
        let side = usize::from(self.stm);
        let friends = self.sides[side];
        let opps = self.sides[side ^ 1];
        let occ = friends ^ opps;
        let pawns = self.piece(side, Piece::PAWN);

        // if generating quiet moves
        if QUIET {
            if self.state.castle_rights & CastleRights::SIDES[side] > 0 && !self.in_check() {
                self.castles(mref, occ);
            }
            if side == Side::WHITE {
                pawn_pushes::<{ Side::WHITE }>(mref, occ, pawns);
            } else {
                pawn_pushes::<{ Side::BLACK }>(mref, occ, pawns);
            }
        }

        if self.state.en_passant > 0 {
            en_passants(mref, pawns, self.state.en_passant as u16, side)
        }

        pawn_captures(mref, pawns, opps, side);
        piece_moves::<{ Piece::KNIGHT }, QUIET>(mref, occ, opps, self.piece(side, Piece::KNIGHT));
        piece_moves::<{ Piece::BISHOP }, QUIET>(mref, occ, opps, self.piece(side, Piece::BISHOP));
        piece_moves::<{ Piece::ROOK }, QUIET>(mref, occ, opps, self.piece(side, Piece::ROOK));
        piece_moves::<{ Piece::QUEEN }, QUIET>(mref, occ, opps, self.piece(side, Piece::QUEEN));
        piece_moves::<{ Piece::KING }, QUIET>(mref, occ, opps, self.piece(side, Piece::KING));

        moves
    }

    fn castles(&self, moves: &mut MoveList, occ: u64) {
        let r = self.state.castle_rights;
        let kbb = self.piece(self.stm(), Piece::KING);
        let ksq = kbb.trailing_zeros() as u16;
        if self.stm {
            if r & CastleRights::BLACK_QS > 0
                && self.can_castle::<{ Side::BLACK }, 0>(occ, kbb, 1 << 58, 1 << 59)
            {
                moves.add(MoveFlag::QS_CASTLE | 58 | ksq << 6);
            }
            if r & CastleRights::BLACK_KS > 0
                && self.can_castle::<{ Side::BLACK }, 1>(occ, kbb, 1 << 62, 1 << 61)
            {
                moves.add(MoveFlag::KS_CASTLE | 62 | ksq << 6);
            }
        } else {
            if r & CastleRights::WHITE_QS > 0
                && self.can_castle::<{ Side::WHITE }, 0>(occ, kbb, 1 << 2, 1 << 3)
            {
                moves.add(MoveFlag::QS_CASTLE | 2 | ksq << 6);
            }
            if r & CastleRights::WHITE_KS > 0
                && self.can_castle::<{ Side::WHITE }, 1>(occ, kbb, 1 << 6, 1 << 5)
            {
                moves.add(MoveFlag::KS_CASTLE | 6 | ksq << 6);
            }
        }
    }

    fn path<const SIDE: usize>(&self, mut path: u64, occ: u64) -> bool {
        bitloop!(
            path,
            idx,
            if self.is_square_attacked(idx as usize, SIDE, occ) {
                return false;
            }
        );
        true
    }

    #[inline]
    fn can_castle<const SIDE: usize, const KS: usize>(
        &self,
        occ: u64,
        kbb: u64,
        kto: u64,
        rto: u64,
    ) -> bool {
        let bit = 1 << (56 * SIDE + usize::from(self.castle.rooks[KS]));
        (occ ^ bit) & (between(kbb, kto) ^ kto) == 0
            && (occ ^ kbb) & (between(bit, rto) ^ rto) == 0
            && self.path::<SIDE>(between(kbb, kto), occ)
    }
}

#[inline]
fn between(bit1: u64, bit2: u64) -> u64 {
    (cmp::max(bit1, bit2) - cmp::min(bit1, bit2)) ^ cmp::min(bit1, bit2)
}
