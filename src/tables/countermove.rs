use super::Move;

// Countermove tables
// if a beta cutoff is caused by a quiet move record it
// in countermove_table[move.from][move.to]
// then pass the previous move through negamax
// and if a move in the movelist matches the countermove
// entry, give it a bonus
// could also have a 6x64 table of move.pc and move.to, 
// but testing showed it was not effective

pub struct CounterMoveTable {
    pub table: [[Move; 64]; 64]
}
impl Default for CounterMoveTable {
    fn default() -> Self {
        #[allow(clippy::declare_interior_mutable_const)]
        const ENTRY: Move = Move::new();
        #[allow(clippy::declare_interior_mutable_const)]
        const ROW: [Move; 64] = [ENTRY; 64];
        Self { 
            table: [ROW; 64] 
        }
    }
}
impl CounterMoveTable {
    pub fn set(&self, prev_m: u16, m: u16) {
        self.table[(prev_m & 63) as usize][((prev_m >> 6) & 63) as usize].set(m)
    }
    pub fn get(&self, m: u16) -> u16 {
        self.table[(m & 63) as usize][((m >> 6) & 63) as usize].get()
    }
}
