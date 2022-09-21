/// Move timing info
#[derive(Default, PartialEq, Eq)]
pub struct Times {
    /// White time on clock
    pub wtime: u64,
    /// Black time on clock
    pub btime: u64,
    /// White time increment
    pub winc: u64,
    /// Black time increment
    pub binc: u64,
    /// Moves until next time control
    pub moves_to_go: Option<u8>,
}

impl Times {
    /// Checks if equal to default
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }
    /// Calculates a movetime
    pub fn to_movetime(&self, side: usize, phase: i16) -> u64 {
        let available = match side {
            0 => self.wtime,
            1 => self.btime,
            _ => panic!("Invalid side!"),
        };

        if self.moves_to_go.is_some() {
            return available / self.moves_to_go.unwrap() as u64;
        }
        available / (2 * (phase as u64 + 1))
    }
}
