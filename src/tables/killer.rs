use super::Move;
use crate::search::MAX_PLY;

// Killer move heuristic
// if a non-capture at ply x causes a beta cutoff
// record it in a table indexed by ply
// so sibling nodes in that ply can give it priority

pub const KILLERS_PER_PLY: usize = 3;
pub struct KillerMoveTable {
    pub table: [[Move; KILLERS_PER_PLY]; MAX_PLY as usize]
}
impl Default for KillerMoveTable {
    fn default() -> Self {
        #[allow(clippy::declare_interior_mutable_const)]
        const ENTRY: Move = Move::new();
        #[allow(clippy::declare_interior_mutable_const)]
        const ROW: [Move; 3] = [ENTRY; KILLERS_PER_PLY];
        Self {
            table: [ROW; MAX_PLY as usize]
        }
    }
}
impl KillerMoveTable {
    // shifts all moves up one
    // puts new move at first index
    // unless the move is already in the table
    // in that case it acts as a wrapping shift
    pub fn push(&self, m: u16, ply: i8) {
        let lost_move = self.table[ply as usize][KILLERS_PER_PLY - 1].get();
        let mut copy_found = false;
        for idx in (1..KILLERS_PER_PLY).rev() {
            let entry = self.table[ply as usize][idx - 1].get();
            if entry == m { copy_found = true }
            self.table[ply as usize][idx].set(entry);
        }
        if copy_found {
            self.table[ply as usize][0].set(lost_move)
        } else {
            self.table[ply as usize][0].set(m)
        }
    }

    pub fn get_ply(&self, ply: i8) -> [u16; KILLERS_PER_PLY] {
        let mut moves = [0; KILLERS_PER_PLY];
        for (i, m) in self.table[ply as usize].iter().enumerate() {
            moves[i] = m.get();
        }
        moves
    }
}
