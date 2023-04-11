use std::{
    mem::size_of,
    sync::atomic::{AtomicU64, AtomicUsize, Ordering::Relaxed},
};

use crate::engine::Score;

/// Public facing.
pub struct HashResult {
    pub key: u16,
    pub r#move: u16,
    pub score: i16,
    pub depth: i8,
    pub bound: u8,
}

/*  Hash Entry Encoding:
    0x 00     00     0000   0000  0000
       bound  depth  score  move  key
*/
#[derive(Default)]
struct HashEntry(AtomicU64);

impl Clone for HashEntry {
    fn clone(&self) -> Self {
        Self(AtomicU64::new(self.0.load(Relaxed)))
    }
}

impl HashEntry {
    #[inline]
    fn new(key: u16, m: u16, score: i16, depth: i8, bound: u8) -> Self {
        Self(AtomicU64::new(
            (key as u64)
                | ((m as u64) << 16)
                | ((score as u64) << 32)
                | ((depth as u64) << 48)
                | ((bound as u64) << 56),
        ))
    }

    #[inline]
    fn load(&self) -> u64 {
        self.0.load(Relaxed)
    }

    #[inline]
    fn key(&self) -> u16 {
        self.load() as u16
    }

    #[inline]
    fn depth(&self) -> i8 {
        ((self.load() >> 16) as u8) as i8
    }
}

impl From<&HashEntry> for HashResult {
    fn from(value: &HashEntry) -> Self {
        let raw = value.load();
        Self {
            key: raw as u16,
            r#move: (raw >> 16) as u16,
            score: (raw >> 32) as i16,
            depth: (raw >> 48) as i8,
            bound: (raw >> 56) as u8,
        }
    }
}

#[derive(Default)]
pub struct HashTable {
    table: Vec<HashEntry>,
    capacity: AtomicUsize,
    filled: AtomicUsize,
}

impl HashTable {
    pub fn capacity(&self) -> usize {
        self.capacity.load(Relaxed)
    }

    pub fn filled(&self) -> usize {
        self.filled.load(Relaxed)
    }

    pub fn resize(&mut self, mut size: usize) {
        size = 2usize.pow((size as f64).log2().floor() as u32);
        self.capacity
            .store(size * 1024 * 1024 / size_of::<AtomicU64>(), Relaxed);
        self.filled.store(0, Relaxed);
        self.table = vec![Default::default(); self.capacity()];
    }

    pub fn clear(&mut self) {
        self.filled.store(0, Relaxed);
        self.table
            .iter_mut()
            .for_each(|bucket| *bucket = Default::default());
    }

    pub fn push(&mut self, hash: u64, m: u16, depth: i8, bound: u8, mut score: i16, ply: i16) {
        let key = (hash >> 48) as u16;
        let idx = (hash as usize) & (self.capacity() - 1);
        let old = HashResult::from(&self.table[idx]);

        if key != old.key || depth >= old.depth {
            score += adjust(score, ply);
            if self.table[idx].load() == 0 {
                self.filled.fetch_add(1, Relaxed);
            }
            self.table[idx] = HashEntry::new(key, m, score, depth, bound);
        }
    }

    pub fn probe(&self, zobrist: u64, ply: i16) -> Option<HashResult> {
        let idx = (zobrist as usize) & (self.capacity() - 1);
        let mut entry = HashResult::from(&self.table[idx]);

        if entry.key == (zobrist >> 48) as u16 {
            entry.score -= adjust(entry.score, ply);
            return Some(entry);
        }
        None
    }
}

#[inline]
fn adjust(score: i16, ply: i16) -> i16 {
    if score > Score::MATE {
        ply
    } else if score < -Score::MATE {
        -ply
    } else {
        0
    }
}
