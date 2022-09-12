// Shamelessly copied from Inanis
// Inanis: https://github.com/Tearth/Inanis/blob/master/src/cache/perft.rs

use std::sync::atomic::{AtomicU64, Ordering};
const ENTRIES_PER_BUCKET: usize = 8;

#[derive(Default)]
pub struct PerftTTEntry {
    key: AtomicU64,
    data: AtomicU64,
}
impl Clone for PerftTTEntry {
    fn clone(&self) -> Self {
        Self { 
            key: AtomicU64::new(self.key.load(Ordering::Relaxed)),
            data: AtomicU64::new(self.data.load(Ordering::Relaxed)),
        }
    }
}

#[derive(Clone, Default)]
#[repr(align(64))]
pub struct PerftTTBucket {
    entries: [PerftTTEntry; ENTRIES_PER_BUCKET],
}
const BUCKET_SIZE: usize = std::mem::size_of::<PerftTTBucket>();

pub struct PerftTT {
    table: Vec<PerftTTBucket>,
    num_buckets: usize,
    pub num_entries: usize,
    pub filled: AtomicU64,
}

impl PerftTT {
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

    pub fn push(&self, zobrist: u64, count: u64, depth: u8) {
        let idx = (zobrist as usize) % self.num_buckets;
        let bucket = &self.table[idx];

        let mut smallest_depth = u8::MAX;
        let mut desired_idx = 0;
        for (entry_idx, entry) in bucket.entries.iter().enumerate() {
            let entry_key = entry.key.load(Ordering::Relaxed);
            let entry_data = entry.data.load(Ordering::Relaxed);
            let entry_depth = ((entry_key ^ entry_data) as u8) & 0xf;

            if entry_depth < smallest_depth {
                smallest_depth = entry_depth;
                desired_idx = entry_idx;
            }
        }
        let key = (zobrist & !0xf) | (depth as u64);
        let data = count;
        if smallest_depth == 0 {
            self.filled.fetch_add(1, Ordering::Relaxed);
        }
        bucket.entries[desired_idx].key.store(key ^ data, Ordering::Relaxed);
        bucket.entries[desired_idx].data.store(data, Ordering::Relaxed);
    }

    pub fn get(&self, zobrist: u64, depth: u8) -> Option<u64> {
        let idx = (zobrist as usize) % self.num_buckets;
        let bucket = &self.table[idx];

        for entry in &bucket.entries {
            let entry_key = entry.key.load(Ordering::Relaxed);
            let entry_data = entry.data.load(Ordering::Relaxed);
            let key = (zobrist & !0xf) | (depth as u64);

            if (entry_key ^ entry_data) == key {
                return Some(entry_data);
            }
        }
        None
    }

    pub fn report(&self) {
        println!("Hashtable: {} / {} entries filled", self.filled.load(Ordering::Relaxed), self.num_entries)
    }
}
