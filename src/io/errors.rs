use std::{fmt, num::ParseIntError};
use kimbo_state::fen::FenError;

#[derive(Debug)]
pub enum UciError {
    Value,
    Display,
    SetOption,
    Go,
    Position,
    Move,
    Fen
}
impl fmt::Display for UciError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Value => write!(f, "could not parse a value as an integer"),
            Self::Display => write!(f, "error parsing 'display' command"),
            Self::SetOption => write!(f, "error parsing 'setoption' command"),
            Self::Go => write!(f, "error parsing 'go' command"),
            Self::Position => write!(f, "error parsing 'position' command"),
            Self::Move => write!(f, "illegal move in moves list"),
            Self::Fen => write!(f, "error parsing fen string")
        }
    }
}

impl From<ParseIntError> for UciError {
    fn from(_: ParseIntError) -> Self {
        Self::Value
    }
}
impl From<FenError> for UciError {
    fn from(_: FenError) -> Self {
        Self::Fen
    }
}
