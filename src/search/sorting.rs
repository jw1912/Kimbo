use super::EnginePosition;

const MVV_LVA: [[i8; 7]; 7] = [
    [15, 14, 13, 12, 11, 10, 0], // victim PAWN
    [25, 24, 23, 22, 21, 20, 0], // victim KNIGHT
    [35, 34, 33, 32, 31, 30, 0], // victim BISHOP
    [45, 44, 43, 42, 41, 40, 0], // victim ROOK
    [55, 54, 53, 52, 51, 50, 0], // victim QUEEN
    [0, 0, 0, 0, 0, 0, 0],       // victim KING (should not be referenced)
    [5, 0, 0, 0, 0, -1, 0],      // empty
];

impl EnginePosition {
    /// Calculates MVV-LVA score for a move
    pub fn mvv_lva(&self, m: &u16) -> i8 {
        let from_idx = m & 0b111111;
        let to_idx = (m >> 6) & 0b111111;
        let moved_pc = self.board.squares[from_idx as usize] as usize;
        let captured_pc = self.board.squares[to_idx as usize] as usize;
        -MVV_LVA[captured_pc][moved_pc]
    }

    pub fn score_move(&self, m: &u16, hash_move: u16, move_hit: &mut bool) -> i8 {
        if *m == hash_move {
            *move_hit = true;
            -100
        } else {
            self.mvv_lva(m)
        }
    }
}
