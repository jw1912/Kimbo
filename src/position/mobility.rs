use super::{*, consts::*, attacks::*, eval::taper};

const CENTER: u64 = 0x3c3c3c3c0000;
const OUTSIDE: u64 = !CENTER;
// develop bishops and knights
const MG_MOBILITY: [i16; 6] = [0, 3, 3, 2, 1, 0];
// rooks and queen need to be active
const EG_MOBILITY: [i16; 6] = [0, 1, 1, 4, 4, 0];
// bonus for controlling center squares
// dont want rooks controlling the center
// bishop control center from afar is good
const CENTER_BONUS: [i16; 6] = [0, 2, 3, 1, 2, 0];


impl Position {
    pub fn piece_mobility_general<const SIDE: usize, const PIECE: usize>(
        &self,
        king_box: u64,
        king_box_attacks: &mut u32,
    ) -> i16 { 
        let mut mobility = 0;
        let mut from: u16;
        let mut idx: usize;
        let mut attacks: u64;
        let mut attackers = self.pieces[SIDE][PIECE];
        let mut occupied = self.occupied;
        // shared files between friendly sliding pieces are good, although they reduce the number
        // of possible moves compared to being on different files, so discount them from occupancy
        occupied ^= match PIECE {
            Piece::BISHOP => self.pieces[SIDE][Piece::BISHOP] | self.pieces[SIDE][Piece::QUEEN],
            Piece::ROOK => self.pieces[SIDE][Piece::ROOK] | self.pieces[SIDE][Piece::QUEEN],
            Piece::QUEEN => self.pieces[SIDE][Piece::BISHOP] | self.pieces[SIDE][Piece::ROOK] | self.pieces[SIDE][Piece::QUEEN],
            _ => 0,
        };
        while attackers > 0 {
            from = ls1b_scan(attackers);
            idx = from as usize;
            attacks = match PIECE {
                Piece::KNIGHT => KNIGHT_ATTACKS[idx],
                Piece::ROOK => rook_attacks(idx, occupied),
                Piece::BISHOP => bishop_attacks(idx, occupied),
                Piece::QUEEN => rook_attacks(idx, occupied) | bishop_attacks(idx, occupied),
                _ => panic!("Not a piece used in mobility: {}", PIECE),
            } & !self.sides[SIDE];
            // bonus to center moves
            mobility += CENTER_BONUS[PIECE] * (attacks & CENTER).count_ones() as i16;
            mobility += (attacks & OUTSIDE).count_ones() as i16;
            *king_box_attacks += (king_box & attacks).count_ones();
            attackers &= attackers - 1;
        }
        mobility
    }

    pub fn side_mobility<const SIDE: usize>(&self, phase: i32, enemy_king_danger: &mut u32) -> i16 {
        let enemy_king_idx = ls1b_scan(self.pieces[SIDE^1][Piece::KING]) as usize; 
        let king_box = KING_ATTACKS[enemy_king_idx];

        let knight = self.piece_mobility_general::<SIDE, { Piece::KNIGHT }>(king_box, enemy_king_danger);
        let bishop = self.piece_mobility_general::<SIDE, { Piece::BISHOP }>(king_box, enemy_king_danger);
        let rook = self.piece_mobility_general::<SIDE, { Piece::ROOK }>(king_box, enemy_king_danger);
        let queen = self.piece_mobility_general::<SIDE, { Piece::QUEEN }>(king_box, enemy_king_danger);

        let mg = knight * MG_MOBILITY[Piece::KNIGHT] + bishop * MG_MOBILITY[Piece::BISHOP] + rook * MG_MOBILITY[Piece::ROOK] + queen * MG_MOBILITY[Piece::QUEEN];
        let eg = knight * EG_MOBILITY[Piece::KNIGHT] + bishop * EG_MOBILITY[Piece::BISHOP] + rook * EG_MOBILITY[Piece::ROOK] + queen * EG_MOBILITY[Piece::QUEEN];
        taper(phase, mg, eg)
    }
}