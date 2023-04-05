pub mod countermove;
pub mod history;
pub mod killer;
pub mod pawn;
pub mod search;

use std::{
    ops::Deref,
    sync::atomic::{AtomicU16, Ordering},
};

// Move struct for counter move and killer move heuristics
pub struct Move(AtomicU16);

impl Deref for Move {
    type Target = AtomicU16;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Clone for Move {
    fn clone(&self) -> Self {
        Self(AtomicU16::new(self.load(Ordering::Relaxed)))
    }
}
impl Move {
    const fn new() -> Self {
        Self(AtomicU16::new(0))
    }

    pub fn set(&self, m: u16) {
        self.store(m, Ordering::Relaxed)
    }

    pub fn get(&self) -> u16 {
        self.load(Ordering::Relaxed)
    }
}
