use std::{
    sync::{
        atomic::{AtomicBool, Ordering::Relaxed},
        Arc,
    },
    time::{Duration, Instant},
};

pub struct Limits {
    time: Instant,
    abort_signal: Arc<AtomicBool>,
    max_time: u128,
    max_depth: i8,
    max_nodes: u64,
}

impl Limits {
    pub fn new(abort_signal: Arc<AtomicBool>) -> Self {
        Self {
            time: Instant::now(),
            abort_signal,
            max_time: 1000,
            max_depth: 64,
            max_nodes: u64::MAX,
        }
    }

    pub fn depth(&self) -> i8 {
        self.max_depth
    }

    pub fn elapsed(&self) -> Duration {
        self.time.elapsed()
    }

    //pub fn set_depth(&mut self, depth: i8) {
    //    self.max_depth = depth;
    //}

    //pub fn set_nodes(&mut self, nodes: u64) {
    //    self.max_nodes = nodes;
    //}

    pub fn set_time(&mut self, time: u128) {
        self.max_time = time;
    }

    pub fn aborting(&self) -> bool {
        self.abort_signal.load(Relaxed)
    }

    pub fn abort_handle(&self) -> Arc<AtomicBool> {
        self.abort_signal.clone()
    }

    pub fn reset(&mut self) {
        self.abort_signal.store(false, Relaxed);
        self.time = Instant::now();
    }

    pub fn should_abort(&mut self, nodes: u64) -> bool {
        if nodes & 1023 == 0
            && (self.elapsed().as_millis() > self.max_time || nodes >= self.max_nodes)
        {
            self.abort_signal.store(true, Relaxed);
            return true;
        }
        false
    }
}
