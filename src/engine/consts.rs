pub const MAX_PLY: i16 = 96;

pub struct Score;
impl Score {
    pub const MAX: i16 = 30_000;
    pub const MATE: i16 = Self::MAX - 256;
    pub const DRAW: i16 = 0;
    pub const ABORT: i16 = 0;
}

pub struct MoveScore;
impl MoveScore {
    pub const HASH: i16 = Score::MAX;
    pub const QUIET: i16 = 0;
    pub const MVV_LVA: [[i16; 6]; 7] = [
        [1500, 1400, 1300, 1200, 1100, 1000],
        [2500, 2400, 2300, 2200, 2100, 2000],
        [3500, 3400, 3300, 3200, 3100, 3000],
        [4500, 4400, 4300, 4200, 4100, 4000],
        [5500, 5400, 5300, 5200, 5100, 5000],
        [   0,    0,    0,    0,    0,    0],
        [1500,    0,    0,    0,    0,    0],
    ];
}