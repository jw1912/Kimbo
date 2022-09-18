use super::consts::*;
use super::{CastleRights, GameState, MoveFlags, Piece, Position, Side};

impl Position {
    /// Makes a move on a position
    pub fn make_move(&mut self, m: u16) {
        let opponent = self.side_to_move ^ 1;
        // extract move data
        let from_idx = (m & 63) as usize;
        let to_idx = ((m >> 6) & 63) as usize;
        let flag = m & MoveFlags::ALL;
        // used derivates of move data
        let from = 1u64 << from_idx;
        let to = 1u64 << to_idx;
        let moved_pc = self.squares[from_idx];
        let mut ctx = GameState { 
            moved_pc,
            captured_pc: Piece::NONE as u8, 
            castle_rights: self.castle_rights, 
            en_passant_sq: self.en_passant_sq, 
            halfmove_clock: self.halfmove_clock,
            zobrist: self.zobrist,
            pawnhash: self.pawnhash,
            phase: self.phase,
            mat_mg: self.mat_mg,
            mat_eg: self.mat_eg,
            pst_mg: self.pst_mg,
            pst_eg: self.pst_eg,
        };
        self.squares[from_idx] = Piece::NONE as u8;
        let mo = from | to;
        // update fields
        self.pieces[self.side_to_move][moved_pc as usize] ^= mo;
        self.sides[self.side_to_move] ^= mo;
        self.en_passant_sq = 0;
        self.pst_mg[self.side_to_move] -= get_weight::<true>(from_idx, self.side_to_move, moved_pc as usize);
        self.pst_eg[self.side_to_move] -= get_weight::<false>(from_idx, self.side_to_move, moved_pc as usize);
        self.zobrist ^= self.zobrist_vals.side_hash();
        self.zobrist ^= self.zobrist_vals.piece_hash(from_idx, self.side_to_move, moved_pc as usize);
        if ctx.en_passant_sq > 0 {
            self.zobrist ^= self.zobrist_vals.en_passant_hash((ctx.en_passant_sq & 7) as usize);
        }
        match flag {
            MoveFlags::QUIET => {
                self.squares[to_idx] = moved_pc;
                self.pst_mg[self.side_to_move] += get_weight::<true>(to_idx, self.side_to_move, moved_pc as usize);
                self.pst_eg[self.side_to_move] += get_weight::<false>(to_idx, self.side_to_move, moved_pc as usize);
                self.zobrist ^= self.zobrist_vals.piece_hash(to_idx, self.side_to_move, moved_pc as usize);
                if moved_pc == 0 {
                    self.pawnhash ^= self.zobrist_vals.piece_hash(from_idx, self.side_to_move, moved_pc as usize);
                    self.pawnhash ^= self.zobrist_vals.piece_hash(to_idx, self.side_to_move, moved_pc as usize);
                }
            }
            MoveFlags::CAPTURE => {
                let captured_pc = self.squares[to_idx] as usize;
                ctx.captured_pc = captured_pc as u8;
                self.pieces[opponent][captured_pc] ^= to;
                self.sides[opponent] ^= to;
                self.squares[to_idx] = moved_pc as u8;
                if captured_pc == Piece::ROOK {
                    self.castle_rights &= CASTLE_RIGHTS[to_idx];
                }
                self.pst_mg[self.side_to_move] += get_weight::<true>(to_idx, self.side_to_move, moved_pc as usize);
                self.pst_eg[self.side_to_move] += get_weight::<false>(to_idx, self.side_to_move, moved_pc as usize);
                // updated captured piece psts
                self.mat_mg[opponent] -= MG_PC_VALS[captured_pc];
                self.mat_eg[opponent] -= EG_PC_VALS[captured_pc];
                self.pst_mg[opponent] -= get_weight::<true>(to_idx, opponent, captured_pc);
                self.pst_eg[opponent] -= get_weight::<false>(to_idx, opponent, captured_pc);
                self.zobrist ^= self.zobrist_vals.piece_hash(to_idx, self.side_to_move, moved_pc as usize);
                self.zobrist ^= self.zobrist_vals.piece_hash(to_idx, opponent, captured_pc);
                self.phase -= PHASE_VALS[captured_pc];
                if moved_pc == 0 {
                    self.pawnhash ^= self.zobrist_vals.piece_hash(from_idx, self.side_to_move, moved_pc as usize);
                    self.pawnhash ^= self.zobrist_vals.piece_hash(to_idx, self.side_to_move, moved_pc as usize);
                }
                if captured_pc == 0 {
                    self.pawnhash ^= self.zobrist_vals.piece_hash(to_idx, opponent, captured_pc);
                }
            }
            MoveFlags::EN_PASSANT => {
                ctx.captured_pc = Piece::PAWN as u8;
                let (pwn, pwn_idx) = match opponent {
                    Side::WHITE => (to << 8, to_idx + 8),
                    Side::BLACK => (to >> 8, to_idx - 8),
                    _ => panic!("Invalid side!"),
                };
                self.pieces[opponent][Piece::PAWN] ^= pwn;
                self.sides[opponent] ^= pwn;
                self.squares[to_idx] = Piece::PAWN as u8;
                self.squares[pwn_idx] = Piece::NONE as u8;
                self.pst_mg[self.side_to_move] += get_weight::<true>(to_idx, self.side_to_move, moved_pc as usize);
                self.pst_eg[self.side_to_move] += get_weight::<false>(to_idx, self.side_to_move, moved_pc as usize);
                self.mat_mg[opponent] -= MG_PC_VALS[0];
                self.mat_eg[opponent] -= EG_PC_VALS[0];
                self.pst_mg[opponent] -= get_weight::<true>(pwn_idx, opponent, 0);
                self.pst_eg[opponent] -= get_weight::<false>(pwn_idx, opponent, 0);
                self.zobrist ^= self.zobrist_vals.piece_hash(to_idx, self.side_to_move, moved_pc as usize);
                self.zobrist ^= self.zobrist_vals.piece_hash(pwn_idx, opponent, 0);
                self.phase -= PHASE_VALS[0];
                self.pawnhash ^= self.zobrist_vals.piece_hash(from_idx, self.side_to_move, moved_pc as usize);
                self.pawnhash ^= self.zobrist_vals.piece_hash(to_idx, self.side_to_move, moved_pc as usize);
                self.pawnhash ^= self.zobrist_vals.piece_hash(pwn_idx, opponent, 0);
            }
            MoveFlags::DBL_PUSH => {
                self.en_passant_sq = match self.side_to_move {
                    Side::WHITE => to_idx - 8,
                    Side::BLACK => to_idx + 8,
                    _ => panic!("Invalid side!"),
                } as u16;
                self.squares[to_idx] = Piece::PAWN as u8;
                self.pst_mg[self.side_to_move] += get_weight::<true>(to_idx, self.side_to_move, moved_pc as usize);
                self.pst_eg[self.side_to_move] += get_weight::<false>(to_idx, self.side_to_move, moved_pc as usize);
                self.zobrist ^= self.zobrist_vals.piece_hash(to_idx, self.side_to_move, moved_pc as usize);
                self.zobrist ^= self.zobrist_vals.en_passant_hash(to_idx & 7);
                self.pawnhash ^= self.zobrist_vals.piece_hash(from_idx, self.side_to_move, moved_pc as usize);
                self.pawnhash ^= self.zobrist_vals.piece_hash(to_idx, self.side_to_move, moved_pc as usize);
            }
            MoveFlags::QS_CASTLE => {
                self.castle_rights &= CastleRights::SIDES[opponent];
                let castle = match self.side_to_move {
                    Side::WHITE => {
                        self.squares[0] = Piece::NONE as u8;
                        self.squares[3] = Piece::ROOK as u8; 
                        A1 | D1
                    },
                    Side::BLACK => {
                        self.squares[56] = Piece::NONE as u8;
                        self.squares[59] = Piece::ROOK as u8; 
                        A8 | D8
                    },
                    _ => panic!("Invalid side!"),
                };
                self.pieces[self.side_to_move][Piece::ROOK] ^= castle;
                self.sides[self.side_to_move] ^= castle;
                self.squares[to_idx] = Piece::KING as u8;
                self.pst_mg[self.side_to_move] += get_weight::<true>(to_idx, self.side_to_move, moved_pc as usize);
                self.pst_eg[self.side_to_move] += get_weight::<false>(to_idx, self.side_to_move, moved_pc as usize);
                let (idx1, idx2) = match self.side_to_move {
                    Side::WHITE => (0, 3),
                    Side::BLACK => (56, 59),
                    _ => panic!("Invalid side!"),
                };
                self.pst_mg[self.side_to_move] -= get_weight::<true>(idx1, self.side_to_move, 3);
                self.pst_eg[self.side_to_move] -= get_weight::<false>(idx1, self.side_to_move, 3);
                self.pst_mg[self.side_to_move] += get_weight::<true>(idx2, self.side_to_move, 3);
                self.pst_eg[self.side_to_move] += get_weight::<false>(idx2, self.side_to_move, 3);
                self.zobrist ^= self.zobrist_vals.piece_hash(idx1, self.side_to_move, 3);
                self.zobrist ^= self.zobrist_vals.piece_hash(idx2, self.side_to_move, 3);
                self.zobrist ^= self.zobrist_vals.piece_hash(to_idx, self.side_to_move, moved_pc as usize);
            }
            MoveFlags::KS_CASTLE => {
                self.castle_rights &= CastleRights::SIDES[opponent];
                let castle = match self.side_to_move {
                    Side::WHITE => {
                        self.squares[7] = Piece::NONE as u8;
                        self.squares[5] = Piece::ROOK as u8; 
                        F1 | H1
                    },
                    Side::BLACK => {
                        self.squares[63] = Piece::NONE as u8;
                        self.squares[61] = Piece::ROOK as u8; 
                        F8 | H8
                    },
                    _ => panic!("Invalid side!"),
                };
                self.pieces[self.side_to_move][Piece::ROOK] ^= castle;
                self.sides[self.side_to_move] ^= castle;
                self.squares[to_idx] = Piece::KING as u8;
                self.pst_mg[self.side_to_move] += get_weight::<true>(to_idx, self.side_to_move, moved_pc as usize);
                self.pst_eg[self.side_to_move] += get_weight::<false>(to_idx, self.side_to_move, moved_pc as usize);
                let (idx1, idx2) = match self.side_to_move {
                    Side::WHITE => (7, 5),
                    Side::BLACK => (63, 61),
                    _ => panic!("Invalid side!"),
                };
                self.pst_mg[self.side_to_move] -= get_weight::<true>(idx1, self.side_to_move, 3);
                self.pst_eg[self.side_to_move] -= get_weight::<false>(idx1, self.side_to_move, 3);
                self.pst_mg[self.side_to_move] += get_weight::<true>(idx2, self.side_to_move, 3);
                self.pst_eg[self.side_to_move] += get_weight::<false>(idx2, self.side_to_move, 3);
                self.zobrist ^= self.zobrist_vals.piece_hash(idx1, self.side_to_move, 3);
                self.zobrist ^= self.zobrist_vals.piece_hash(idx2, self.side_to_move, 3);
                self.zobrist ^= self.zobrist_vals.piece_hash(to_idx, self.side_to_move, moved_pc as usize);
            }
            _ => {
                // promotions
                self.pieces[self.side_to_move][moved_pc as usize] ^= to;
                let promo_pc = (((flag >> 12) & 3) + 1) as usize;
                if flag < MoveFlags::KNIGHT_PROMO_CAPTURE {
                    self.pieces[self.side_to_move][promo_pc] ^= to;
                    self.squares[to_idx] = promo_pc as u8;
                    self.pst_mg[self.side_to_move] += get_weight::<true>(to_idx, self.side_to_move, promo_pc);
                    self.pst_eg[self.side_to_move] += get_weight::<false>(to_idx, self.side_to_move, promo_pc);
                    self.zobrist ^= self.zobrist_vals.piece_hash(to_idx, self.side_to_move, promo_pc);
                    self.phase += PHASE_VALS[promo_pc];
                    self.mat_mg[self.side_to_move] += MG_PC_VALS[promo_pc];
                    self.mat_mg[self.side_to_move] -= MG_PC_VALS[0];
                    self.mat_eg[self.side_to_move] += EG_PC_VALS[promo_pc];
                    self.mat_eg[self.side_to_move] -= EG_PC_VALS[0];
                    self.pawnhash ^= self.zobrist_vals.piece_hash(from_idx, self.side_to_move, moved_pc as usize);
                } else {
                    let captured_pc = self.squares[to_idx] as usize;
                    ctx.captured_pc = captured_pc as u8;
                    self.pieces[self.side_to_move][promo_pc] ^= to;
                    self.pieces[opponent][captured_pc] ^= to;
                    self.sides[opponent] ^= to;
                    self.squares[to_idx] = promo_pc as u8;
                    if captured_pc == Piece::ROOK {
                        self.castle_rights &= CASTLE_RIGHTS[to_idx];
                    }
                    self.pst_mg[self.side_to_move] += get_weight::<true>(to_idx, self.side_to_move, promo_pc);
                    self.pst_eg[self.side_to_move] += get_weight::<false>(to_idx, self.side_to_move, promo_pc);
                    let cap_pc = ctx.captured_pc as usize;
                    self.mat_mg[self.side_to_move] += MG_PC_VALS[promo_pc];
                    self.mat_mg[self.side_to_move] -= MG_PC_VALS[0];
                    self.mat_mg[opponent] -= MG_PC_VALS[cap_pc];
                    self.mat_eg[self.side_to_move] += EG_PC_VALS[promo_pc];
                    self.mat_eg[self.side_to_move] -= EG_PC_VALS[0];
                    self.mat_eg[opponent] -= EG_PC_VALS[cap_pc];
                    self.pst_mg[opponent] -= get_weight::<true>(to_idx, opponent, cap_pc);
                    self.pst_eg[opponent] -= get_weight::<false>(to_idx, opponent, cap_pc);
                    self.zobrist ^= self.zobrist_vals.piece_hash(to_idx, self.side_to_move, promo_pc);
                    self.zobrist ^= self.zobrist_vals.piece_hash(to_idx, opponent, cap_pc);
                    self.phase += PHASE_VALS[promo_pc];
                    self.phase -= PHASE_VALS[cap_pc];
                    self.pawnhash ^= self.zobrist_vals.piece_hash(from_idx, self.side_to_move, moved_pc as usize);
                }
            }
        }
        self.occupied = self.sides[0] | self.sides[1];
        self.fullmove_counter += (self.side_to_move == Side::BLACK) as u16;
        if moved_pc > Piece::PAWN as u8 && flag != MoveFlags::CAPTURE {
            self.halfmove_clock += 1
        } else {
            self.halfmove_clock = 0
        }
        if self.castle_rights > CastleRights::NONE && (moved_pc == Piece::KING as u8 || moved_pc == Piece::ROOK as u8) {
            self.castle_rights &= CASTLE_RIGHTS[from_idx]
        } 
        let mut changed_castle = ctx.castle_rights & !self.castle_rights;
        while changed_castle > 0 {
            let ls1b = changed_castle & changed_castle.wrapping_neg();
            self.zobrist ^= self.zobrist_vals.castle_hash(ctx.castle_rights, ls1b);
            changed_castle &= changed_castle - 1
        }
        if moved_pc == 5 {
            self.pawnhash ^= self.zobrist_vals.piece_hash(from_idx, self.side_to_move, moved_pc as usize);
            self.pawnhash ^= self.zobrist_vals.piece_hash(to_idx, self.side_to_move, moved_pc as usize);
        }
        self.side_to_move ^= 1;
        self.state_stack.push(ctx);
    }

    /// Unmakes a move given the move context
    pub fn unmake_move(&mut self, m: u16) {
        let ctx = self.state_stack.pop().unwrap();
        self.mat_mg = ctx.mat_mg;
        self.mat_eg = ctx.mat_eg;
        self.pst_mg = ctx.pst_mg;
        self.pst_eg = ctx.pst_eg;
        self.phase = ctx.phase;
        self.zobrist = ctx.zobrist;
        self.pawnhash = ctx.pawnhash;
        let opponent = self.side_to_move;
        self.side_to_move ^= 1;
        // extract move data
        let from_idx = (m & 63) as usize;
        let to_idx = ((m >> 6) & 63) as usize;
        let flag = m & MoveFlags::ALL;
        // used derivates of move data
        let from = 1u64 << from_idx;
        let to = 1u64 << to_idx;
        let moved_pc = ctx.moved_pc as usize;
        self.squares[from_idx] = moved_pc as u8;
        let m = from | to;
        // reset fields
        self.pieces[self.side_to_move][moved_pc] ^= m;
        self.sides[self.side_to_move] ^= m;
        self.castle_rights = ctx.castle_rights;
        self.en_passant_sq = ctx.en_passant_sq;
        self.halfmove_clock = ctx.halfmove_clock;
        if self.side_to_move != Side::WHITE {
            self.fullmove_counter -= 1
        }
        // flag specifics
        match flag {
            MoveFlags::QUIET | MoveFlags::DBL_PUSH => {self.squares[to_idx] = Piece::NONE as u8;}
            MoveFlags::CAPTURE => {
                self.pieces[opponent][ctx.captured_pc as usize] ^= to;
                self.sides[opponent] ^= to;
                self.squares[to_idx] = ctx.captured_pc as u8;
            }
            MoveFlags::EN_PASSANT => {
                let (pwn, pwn_idx) = match opponent {
                    Side::WHITE => (to << 8, to_idx + 8),
                    Side::BLACK => (to >> 8, to_idx - 8),
                    _ => panic!("Invalid side!"),
                };
                self.pieces[opponent][Piece::PAWN] ^= pwn;
                self.sides[opponent] ^= pwn;
                self.squares[pwn_idx] = Piece::PAWN as u8;
            }
            MoveFlags::QS_CASTLE => {
                let castle = match self.side_to_move {
                    Side::WHITE => {
                        self.squares[3] = Piece::NONE as u8;
                        self.squares[0] = Piece::ROOK as u8; 
                        A1 | D1
                    },
                    Side::BLACK => {
                        self.squares[59] = Piece::NONE as u8;
                        self.squares[56] = Piece::ROOK as u8; 
                        A8 | D8
                    },
                    _ => panic!("Invalid side!"),
                };
                self.pieces[self.side_to_move][Piece::ROOK] ^= castle;
                self.sides[self.side_to_move] ^= castle;
                self.squares[to_idx] = Piece::NONE as u8;
            }
            MoveFlags::KS_CASTLE => {
                let castle = match self.side_to_move {
                    Side::WHITE => {
                        self.squares[5] = Piece::NONE as u8;
                        self.squares[7] = Piece::ROOK as u8; 
                        F1 | H1
                    },
                    Side::BLACK => {
                        self.squares[61] = Piece::NONE as u8;
                        self.squares[63] = Piece::ROOK as u8; 
                        F8 | H8
                    },
                    _ => panic!("Invalid side!"),
                };
                self.pieces[self.side_to_move][Piece::ROOK] ^= castle;
                self.sides[self.side_to_move] ^= castle;
                self.squares[to_idx] = Piece::NONE as u8;
            }
            _ => {
                // is a promotion
                self.pieces[self.side_to_move][moved_pc] ^= to;
                let promo_pc = (((flag >> 12) & 3) + 1) as usize;
                // if a push promotion
                if flag < MoveFlags::KNIGHT_PROMO_CAPTURE {
                    self.pieces[self.side_to_move][promo_pc] ^= to;
                    self.squares[to_idx] = Piece::NONE as u8;
                } else {
                    self.pieces[self.side_to_move][promo_pc] ^= to;
                    self.pieces[opponent][ctx.captured_pc as usize] ^= to;
                    self.sides[opponent] ^= to;
                    self.squares[to_idx] = ctx.captured_pc;
                }
            }
        }
        self.occupied = self.sides[0] | self.sides[1];
    }

    pub fn make_validation_move(&mut self, m: u16) {
        let opponent = self.side_to_move ^ 1;
        // extract move data
        let from_idx = (m & 63) as usize;
        let to_idx = ((m >> 6) & 63) as usize;
        let flag = m & MoveFlags::ALL;
        // used derivates of move data
        let from = 1u64 << from_idx;
        let to = 1u64 << to_idx;
        let moved_pc = self.squares[from_idx] as usize;
        let mo = from | to;
        self.pieces[self.side_to_move][moved_pc] ^= mo;
        self.sides[self.side_to_move] ^= mo;
        if flag == MoveFlags::EN_PASSANT {
            let pwn = match opponent {
                Side::WHITE => to << 8,
                Side::BLACK => to >> 8,
                _ => panic!("Invalid side!"),
            };
            self.pieces[opponent][Piece::PAWN] ^= pwn;
            self.sides[opponent] ^= pwn;
        }
        self.side_to_move ^= 1;
        self.occupied = self.sides[0] | self.sides[1];
    }
}
