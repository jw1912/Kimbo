use super::Times;

impl Times {
    /// Checks if equal to default
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }
    /// Calculates a movetime
    pub fn to_movetime(&self, side: usize) -> u64 {
        let available = match side {
            0 => self.wtime,
            1 => self.btime,
            _ => panic!("Invalid side!")
        };
        let increment = match side {
            0 => self.winc,
            1 => self.binc,
            _ => panic!("Invalid side!")
        };

        if self.moves_to_go.is_some() {
            return available / self.moves_to_go.unwrap() as u64
        }
        available / 32 + increment
    }
}