use super::{consts::*, movegen::*, Move, MoveList};

/// State that is copied for undoing moves.
#[derive(Clone, Copy)]
pub struct State {
    castle_rights: u8,
    en_passant: u8,
    halfmove_clock: u8,
    zobrist: u64,
}

/// Required info to undo a move.
#[derive(Clone, Copy)]
pub struct MoveContext {
    r#move: u16,
    moved: u8,
    captured: u8,
    state: State,
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
    castle_mask: [u8; 64],
}

/// Holds all random values for hashing.
pub struct ZobristVals {
    pieces: [[[u64; 64]; 6]; 2],
    castling: [u64; 4],
    en_passant: [u64; 8],
    side: u64,
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
        if occ & bit == 0 {
            return 6;
        }
        usize::from(
            (self.pieces[Piece::KNIGHT] | self.pieces[Piece::ROOK] | self.pieces[Piece::KING])
                & bit
                > 0,
        ) + 2 * usize::from((self.pieces[Piece::BISHOP] | self.pieces[Piece::ROOK]) & bit > 0)
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
            self.castle_mask[usize::from(to)] & self.castle_mask[usize::from(from)];
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
        let invalid = self.is_square_attacked(kidx, side, occ);
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
                let promo = usize::from((flag & 3) + 1);
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
        if self.stm {
            if r & CastleRights::BLACK_QS > 0
                && occ & B8C8D8 == 0
                && !self.is_square_attacked(59, Side::BLACK, occ)
            {
                moves.add(60 << 6 | 58 | MoveFlag::QS_CASTLE);
            }
            if r & CastleRights::BLACK_KS > 0
                && occ & F8G8 == 0
                && !self.is_square_attacked(61, Side::BLACK, occ)
            {
                moves.add(60 << 6 | 62 | MoveFlag::KS_CASTLE);
            }
        } else {
            if r & CastleRights::WHITE_QS > 0
                && occ & B1C1D1 == 0
                && !self.is_square_attacked(3, Side::WHITE, occ)
            {
                moves.add(4 << 6 | 2 | MoveFlag::QS_CASTLE);
            }
            if r & CastleRights::WHITE_KS > 0
                && occ & F1G1 == 0
                && !self.is_square_attacked(5, Side::WHITE, occ)
            {
                moves.add(4 << 6 | 6 | MoveFlag::KS_CASTLE);
            }
        }
    }
}
