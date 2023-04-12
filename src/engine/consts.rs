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
}