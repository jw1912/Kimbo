use crate::search::sorting::HISTORY_MAX;

#[derive(Clone, Copy)]
pub struct HistoryTable {
    pub table: [[[u32; 64]; 64]; 2],
    max: u32,
}
impl Default for HistoryTable {
    fn default() -> Self {
        Self { 
            table: [[[0; 64]; 64]; 2],
            max: 1,
        }
    }
}
impl HistoryTable {
    pub fn set(&mut self, side: usize, m: u16, depth: u8) {
        let locale = &mut self.table[side][(m & 63) as usize][((m >> 6) & 63) as usize];
        let new = *locale + (depth as u32) * (depth as u32);
        self.max = std::cmp::max(self.max, new);
        *locale = new;
    }
    pub fn get(&self, side: usize, m: u16) -> i16 {
        let val = self.table[side][(m & 63) as usize][((m >> 6) & 63) as usize];
        ((val * HISTORY_MAX as u32 + self.max - 1) / self.max) as i16
    }
}