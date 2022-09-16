use std::sync::atomic::{AtomicU16, Ordering};

use crate::search::MAX_PLY;

// Countermove tables
// if a beta cutoff is caused by a quiet move record it
// in countermove_table[move.from][move.to]
// then pass the previous move through negamax
// and if a move in the movelist matches the countermove
// entry, give it a bonus
// could also have a 6x64 table of move.pc and move.to, 
// but testing showed it was not effective

pub struct KillerMove {
    data: AtomicU16
}
impl Clone for KillerMove {
    fn clone(&self) -> Self {
        Self { data: AtomicU16::new(self.data.load(Ordering::Relaxed)) }
    }
}
impl KillerMove {
    const fn new() -> Self {
        Self { data: AtomicU16::new(0) }
    }

    pub fn set(&self, m: u16) {
        self.data.store(m, Ordering::Relaxed) 
    }

    pub fn get(&self) -> u16 {
        self.data.load(Ordering::Relaxed)
    }
}

pub const KILLERS_PER_PLY: usize = 3;
pub struct KillerMoveTable {
    pub table: [[KillerMove; KILLERS_PER_PLY]; MAX_PLY as usize]
}
impl Default for KillerMoveTable {
    fn default() -> Self {
        #[allow(clippy::declare_interior_mutable_const)]
        const ENTRY: KillerMove = KillerMove::new();
        #[allow(clippy::declare_interior_mutable_const)]
        const ROW: [KillerMove; 3] = [ENTRY; KILLERS_PER_PLY];
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
    pub fn push(&self, m: u16, ply: u8) {
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

    pub fn get_ply(&self, ply: u8) -> [u16; KILLERS_PER_PLY] {
        let mut moves = [0; KILLERS_PER_PLY];
        for (i, m) in self.table[ply as usize].iter().enumerate() {
            moves[i] = m.get();
        }
        moves
    }

    // shifts all moves 2 ply up
    pub fn age(&self) {
        for ply in 2..MAX_PLY as usize {
            for slot_index in 0..KILLERS_PER_PLY {
                let entry = self.table[ply][slot_index].get();
                self.table[ply - 2][slot_index].set(entry);
            }
        }
        for ply in MAX_PLY as usize - 2..MAX_PLY as usize {
            for entry in &self.table[ply] {
                entry.set(Default::default());
            }
        }
    }
}
