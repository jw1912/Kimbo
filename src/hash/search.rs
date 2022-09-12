// This implementation is heavily inspired by Rustic and Inanis
// Rustic: https://github.com/mvanthoor/rustic/blob/master/src/engine/transposition.rs
// Inanis: https://github.com/Tearth/Inanis/blob/master/src/cache/search.rs
// however the replacement scheme is my own and optimisations have been made

use std::sync::atomic::{AtomicU64, Ordering};

pub struct Bound;
impl Bound {
    pub const INVALID: u8 = 0;
    pub const LOWER: u8 = 1;
    pub const UPPER: u8 = 2;
    pub const EXACT: u8 = 3;
}

const ENTRIES_PER_BUCKET: usize = 8;

#[derive(Default)]
pub struct HashEntry {
    data: AtomicU64,
}
impl Clone for HashEntry {
    fn clone(&self) -> Self {
        Self {
            data: AtomicU64::new(self.data.load(Ordering::Relaxed)),
        }
    }
}

#[derive(Clone, Default)]
#[repr(align(64))]
pub struct HashBucket {
    entries: [HashEntry; ENTRIES_PER_BUCKET],
}
// should be 64 bytes
const BUCKET_SIZE: usize = std::mem::size_of::<HashBucket>();

pub struct HashTable {
    table: Vec<HashBucket>,
    num_buckets: usize,
    num_entries: usize,
    filled: AtomicU64,
}

#[derive(Default)]
pub struct HashResult {
    pub key: u16,
    pub best_move: u16,
    pub score: i16,
    pub depth: u8,
    pub age: u8,
    pub bound: u8,
}

impl HashTable {
    pub fn new(size: usize) -> Self {
        let num_buckets = size / BUCKET_SIZE;
        let num_entries = num_buckets * ENTRIES_PER_BUCKET;
        Self {
            table: vec![Default::default(); num_buckets],
            num_buckets,
            num_entries,
            filled: AtomicU64::new(0),
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub fn push(&self, zobrist: u64, best_move: u16, depth: u8, age: u8, bound: u8, score: i16, _ply: u8) {
        let key = (zobrist >> 48) as u16;
        let idx = (zobrist as usize) % self.num_buckets;
        let bucket = &self.table[idx];
        let mut desired_idx = usize::MAX;
        let mut smallest_depth = u8::MAX;
        for (entry_idx, entry) in bucket.entries.iter().enumerate() {
            let data = entry.data.load(Ordering::Relaxed);
            let entry_data = HashEntry::load(data);
            // ignoring entries from previous searches anyway, so they are first to be replaced
            if entry_data.age != age {
                desired_idx = entry_idx;
                break;
            }
            // then replace lower depth entries with this key
            if entry_data.key == key && depth > entry_data.depth {
                desired_idx = entry_idx;
                break;
            }
            // then fill remaining empty entries
            if entry_data.depth == 0 {
                self.filled.fetch_add(1, Ordering::Relaxed);
                desired_idx = entry_idx;
                break;
            }
            // if all else fails, replace the entry with lowest search depth
            if entry_data.depth < smallest_depth {
                smallest_depth = entry_data.depth;
                desired_idx = entry_idx;
                continue;
            }
        }
        bucket.entries[desired_idx].store(key, best_move, depth, age, bound, score);
    }

    pub fn get(&self, zobrist: u64, _ply: u8, search_age: u8) -> Option<HashResult> {
        let key = (zobrist >> 48) as u16;
        let idx = (zobrist as usize) % self.num_buckets;
        let bucket = &self.table[idx];
        for entry in &bucket.entries {
            let data = entry.data.load(Ordering::Relaxed);
            let entry_key = HashEntry::get_key(data);
            // require that the key matches AND that the result is from this search
            if entry_key == key && search_age == HashEntry::get_age(data) {
                let entry_data = HashEntry::load(data);
                return Some(entry_data);
            } 
        }
        None
    }

    pub fn hashfull(&self) -> u64 {
        self.filled.load(Ordering::Relaxed) * 1000 / self.num_entries as u64
    }
}

impl HashEntry {
    fn store(&self, key: u16, best_move: u16, depth: u8, age: u8, bound: u8, score: i16) {
        let data = (key as u64)
            | ((best_move as u64) << 16)
            | (((score as u16) as u64) << 32)
            | ((depth as u64) << 48)
            | ((bound as u64) << 56)
            | ((age as u64) << 58);
        self.data.store(data, Ordering::Relaxed);
    }

    fn get_key(data: u64) -> u16 {
        data as u16
    }

    fn get_age(data: u64) -> u8 {
        (data >> 58) as u8
    }

    fn load(data: u64) -> HashResult {
        HashResult {
            key: data as u16,
            best_move: (data >> 16) as u16,
            score: (data >> 32) as i16,
            depth: (data >> 48) as u8,
            age: (data >> 58) as u8,
            bound: ((data >> 56) & 3) as u8,
        }
    }
}
