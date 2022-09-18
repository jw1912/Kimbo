use std::{fmt, num::ParseIntError};
use super::fen::FenError;

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
            Self::Value => write!(f, "error parsing value as integer"),
            Self::Display => write!(f, "error parsing 'display' command"),
            Self::SetOption => write!(f, "error parsing 'setoption' command"),
            Self::Go => write!(f, "error parsing 'go' command"),
            Self::Position => write!(f, "error parsing 'position' command"),
            Self::Move => write!(f, "error parsing 'moves' list"),
            Self::Fen => write!(f, "error parsing 'fen' string")
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
