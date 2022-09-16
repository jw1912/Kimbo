pub mod perft;
pub mod search;
pub mod pawn;
pub mod countermove;
pub mod killer;
pub mod history;

use std::sync::atomic::{AtomicU16, Ordering};

// Move struct for counter move and killer move heuristics

pub struct Move {
    data: AtomicU16
}
impl Clone for Move {
    fn clone(&self) -> Self {
        Self { data: AtomicU16::new(self.data.load(Ordering::Relaxed)) }
    }
}
impl Move {
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