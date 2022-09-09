use std::sync::atomic::{AtomicU64, Ordering};

const ENTRIES_PER_BUCKET: usize = 8;

#[derive(Default)]
pub struct PerftTTEntry {
    pub key: AtomicU64,
    pub data: AtomicU64,
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
    pub entries: [PerftTTEntry; ENTRIES_PER_BUCKET],
}

pub struct PerftTT {
    pub table: Vec<PerftTTBucket>,
    pub num_buckets: usize,
}

#[derive(Default)]
pub struct PerftTTResult {
    pub count: u64,
}
impl PerftTTResult {
    fn new(count: u64) -> Self {
        Self {
            count
        }
    }
}

impl PerftTT {
    pub fn new(size: usize) -> Self {
        let bucket_size = std::mem::size_of::<PerftTTBucket>();
        let num_buckets = size / bucket_size;
        let mut table = Self {
            table: Vec::with_capacity(num_buckets),
            num_buckets,
        };
        if size != 0 {
            table
                .table
                .resize(table.table.capacity(), Default::default())
        }
        table
    }

    pub fn push(&self, zobrist: u64, count: u64, depth: u8) {
        let index = (zobrist as usize) % self.num_buckets;
        let bucket = &self.table[index];

        let mut smallest_depth = u8::MAX;
        let mut desired_index = 0;
        for (entry_index, entry) in bucket.entries.iter().enumerate() {
            let entry_key = entry.key.load(Ordering::Relaxed);
            let entry_data = entry.data.load(Ordering::Relaxed);
            let entry_depth = ((entry_key ^ entry_data) as u8) & 0xf;

            if entry_depth < smallest_depth {
                smallest_depth = entry_depth;
                desired_index = entry_index;
            }
        }
        let key = (zobrist & !0xf) | (depth as u64);
        let data = count;

        bucket.entries[desired_index].key.store(key ^ data, Ordering::Relaxed);
        bucket.entries[desired_index].data.store(data, Ordering::Relaxed);
    }

    pub fn get(&self, zobrist: u64, depth: u8) -> Option<PerftTTResult> {
        let index = (zobrist as usize) % self.num_buckets;
        let bucket = &self.table[index];

        for entry in &bucket.entries {
            let entry_key = entry.key.load(Ordering::Relaxed);
            let entry_data = entry.data.load(Ordering::Relaxed);
            let key = (zobrist & !0xf) | (depth as u64);

            if (entry_key ^ entry_data) == key {
                return Some(PerftTTResult::new(entry_data));
            }
        }
        None
    }
}
