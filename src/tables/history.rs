use crate::search::sorting::HISTORY_MAX;
use std::sync::atomic::{AtomicU32, Ordering};

// History heuristic
// If a non-capture causes a beta cutoff,
// add to a table indexed by from and to squares depth*depth
// order quiet moves that are not counter or killer moves
// according to the value in the table of their from and
// to squares

pub struct HistoryScore {
    data: AtomicU32,
}
impl Clone for HistoryScore {
    fn clone(&self) -> Self {
        Self {
            data: AtomicU32::new(self.data.load(Ordering::Relaxed)),
        }
    }
}
impl HistoryScore {
    const fn new() -> Self {
        Self {
            data: AtomicU32::new(0),
        }
    }

    pub fn get(&self) -> u32 {
        self.data.load(Ordering::Relaxed)
    }
}

pub struct HistoryTable {
    pub table: [[[HistoryScore; 64]; 64]; 2],
    pub max: AtomicU32,
}
impl Default for HistoryTable {
    fn default() -> Self {
        #[allow(clippy::declare_interior_mutable_const)]
        const ENTRY: HistoryScore = HistoryScore::new();
        #[allow(clippy::declare_interior_mutable_const)]
        const ROW: [HistoryScore; 64] = [ENTRY; 64];
        #[allow(clippy::declare_interior_mutable_const)]
        const SIDE: [[HistoryScore; 64]; 64] = [ROW; 64];
        Self {
            table: [SIDE; 2],
            max: AtomicU32::new(1),
        }
    }
}
impl HistoryTable {
    pub fn set(&self, side: usize, m: u16, depth: i8) {
        let locale = &self.table[side][(m & 63) as usize][((m >> 6) & 63) as usize];
        let new = locale.get() + (depth as u32) * (depth as u32);
        if new > self.max.load(Ordering::Relaxed) {
            self.max.store(new, Ordering::Relaxed)
        }
        locale.data.store(new, Ordering::Relaxed)
    }

    pub fn get(&self, side: usize, m: u16) -> i16 {
        let val = self.table[side][(m & 63) as usize][((m >> 6) & 63) as usize].get();
        let max = self.max.load(Ordering::Relaxed);
        ((val * HISTORY_MAX as u32 + max - 1) / max) as i16
    }
}
