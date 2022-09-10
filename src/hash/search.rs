use std::sync::atomic::{AtomicU64, Ordering};
use crate::engine::sorting::is_score_near_mate;

const ENTRIES_PER_BUCKET: usize = 8;

#[derive(Default)]
pub struct TTEntry {
    pub data: AtomicU64,
}
impl Clone for TTEntry {
    fn clone(&self) -> Self {
        Self {
            data: AtomicU64::new(self.data.load(Ordering::Relaxed)),
        }
    }
}

#[derive(Clone, Default)]
#[repr(align(64))]
pub struct TTBucket {
    pub entries: [TTEntry; ENTRIES_PER_BUCKET],
}
const BUCKET_SIZE: usize = std::mem::size_of::<TTBucket>();

pub struct TT {
    pub table: Vec<TTBucket>,
    pub num_buckets: usize,
    pub num_entries: usize,
    pub filled: AtomicU64,
}

#[derive(Default)]
pub struct TTResult {
    pub key: u16,
    pub best_move: u16,
    pub score: i16,
    pub depth: u8,
    pub age: u8,
    pub cutoff_type: u8,
}

#[derive(Default)]
pub struct TTPerft {
    pub key: u16,
    pub count: u64,
    pub depth: u8,
    pub age: u8,
}

pub struct CutoffType;
impl CutoffType {
    pub const INVALID: u8 = 0;
    pub const EXACT: u8 = 1;
    pub const ALPHA: u8 = 2;
    pub const BETA: u8 = 3;
}

impl TT {
    pub fn new(size: usize) -> Self {
        let num_buckets = size / BUCKET_SIZE;
        let num_entries = num_buckets * ENTRIES_PER_BUCKET;
        let mut table = Self {
            table: Vec::with_capacity(num_buckets),
            num_buckets,
            num_entries,
            filled: AtomicU64::new(0),
        };
        if size != 0 {
            table
                .table
                .resize(table.table.capacity(), Default::default())
        }
        table
    }
    #[allow(clippy::too_many_arguments)]
    pub fn push(
        &self,
        zobrist: u64,
        orig_score: i16,
        best_move: u16,
        depth: u8,
        ply: u8,
        age: u8,
        cutoff_type: u8,
    ) {
        let key = (zobrist >> 48) as u16;
        let index = (zobrist as usize) % self.num_buckets;
        let bucket = &self.table[index];
        let mut smallest_depth = u8::MAX;
        let mut desired_index = usize::MAX;
        let mut found_old_entry = false;
        let mut score = orig_score;

        for (entry_index, entry) in bucket.entries.iter().enumerate() {
            let entry_data = entry.get_data();
            if entry_data.depth == 0 {
                self.filled.fetch_add(1, Ordering::Relaxed);
                desired_index = entry_index;
                break;
            }

            if entry_data.key == key && depth > entry_data.depth {
                desired_index = entry_index;
                break;
            }
            if entry_data.age != age {
                if found_old_entry {
                    if entry_data.depth < smallest_depth {
                        desired_index = entry_index;
                        smallest_depth = entry_data.depth;
                    }
                } else {
                    desired_index = entry_index;
                    smallest_depth = entry_data.depth;
                    found_old_entry = true;
                }

                continue;
            }

            if !found_old_entry && entry_data.depth < smallest_depth {
                smallest_depth = entry_data.depth;
                desired_index = entry_index;
                continue;
            }
        }
        if is_score_near_mate(score) {
            if score > 0 {
                score += ply as i16 - 1;
            } else {
                score -= ply as i16 + 1;
            }
        }

        bucket.entries[desired_index].set_data(key, score, best_move, depth, age, cutoff_type);
    }

    pub fn get(&self, zobrist: u64, ply: u8) -> Option<TTResult> {
        let key = (zobrist >> 48) as u16;
        let index = (zobrist as usize) % self.num_buckets;
        let bucket = &self.table[index];

        for entry in &bucket.entries {
            let entry_key = entry.get_key();
            if entry_key == key {
                let mut entry_data = entry.get_data();
                if is_score_near_mate(entry_data.score) {
                    if entry_data.score > 0 {
                        entry_data.score -= ply as i16 + 1
                    } else {
                        entry_data.score += ply as i16 - 1
                    }
                }
                return Some(entry_data);
            } 
        }
        None
    }
}

impl TTEntry {
    pub fn set_data(
        &self,
        key: u16,
        score: i16,
        best_move: u16,
        depth: u8,
        age: u8,
        cutoff_type: u8,
    ) {
        let data = (key as u64)
            | (((score as u16) as u64) << 16)
            | ((best_move as u64) << 32)
            | ((depth as u64) << 48)
            | ((cutoff_type as u64) << 56)
            | ((age as u64) << 58);

        self.data.store(data, Ordering::Relaxed);
    }

    pub fn get_key(&self) -> u16 {
        let data = self.data.load(Ordering::Relaxed);
        data as u16
    }

    pub fn get_data(&self) -> TTResult {
        let data = self.data.load(Ordering::Relaxed);
        TTResult {
            key: data as u16,
            best_move: (data >> 32) as u16,
            score: ((data >> 16) as u16) as i16,
            depth: (data >> 48) as u8,
            age: (data >> 58) as u8,
            cutoff_type: ((data >> 56) & 3) as u8,
        }
    }
}
