use crate::state::Position;

mod eval;
mod limits;
mod search;

use limits::Limits;

pub struct Score;
impl Score {
    pub const MAX: i16 = 30_000;
    pub const MATE: i16 = Self::MAX - 256;
    pub const DRAW: i16 = 0;
    pub const ABORT: i16 = 0;
}

pub struct Engine {
    pub position: Position,
    pub limits: Limits,
}
