use std::{
    sync::{
        atomic::{AtomicBool, Ordering::Relaxed},
        Arc,
    },
    time::Instant,
};

pub struct Limits {
    time: Instant,
    abort_signal: Arc<AtomicBool>,
    max_time: u64,
    max_depth: i8,
    max_nodes: u64,
}

impl Limits {
    pub fn new(abort_signal: Arc<AtomicBool>) -> Self {
        Self {
            time: Instant::now(),
            abort_signal,
            max_time: 1000,
            max_depth: 0,
            max_nodes: 0,
        }
    }

    pub fn set_depth(&mut self, depth: i8) {
        self.max_depth = depth;
    }

    pub fn set_nodes(&mut self, nodes: u64) {
        self.max_nodes = nodes;
    }

    pub fn set_time(&mut self, time: u64) {
        self.max_time = time;
    }

    pub fn aborting(&self) -> bool {
        self.abort_signal.load(Relaxed)
    }

    pub fn reset(&mut self) {
        self.abort_signal.store(false, Relaxed);
        self.time = Instant::now();
    }

    pub fn should_abort(&mut self, nodes: u64) -> bool {
        if self.time.elapsed().as_millis() as u64 > self.max_time || nodes >= self.max_nodes {
            self.abort_signal.store(true, Relaxed);
            return true;
        }
        false
    }

    pub fn allocate_time(&mut self, remaining: u64, increment: u64, moves_to_go: Option<u64>) {
        self.max_time = remaining / moves_to_go.unwrap_or(25) + 3 * increment / 4 - 10;
    }
}
