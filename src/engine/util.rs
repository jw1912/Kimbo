use crate::state::{MoveFlag, MoveType, Position};

pub fn u16_to_uci(pos: &Position, m: u16) -> String {
    let index_to_square = |i| format!("{}{}", ((i & 7) as u8 + b'a') as char, (i / 8) + 1);

    // extract move info
    let from_idx = (m >> 6) & 63;
    let from = index_to_square(from_idx);
    let to = index_to_square(m & 63);
    let flag = m & MoveFlag::ALL;

    // chess960 castle or promotion?
    if pos.chess960() && (flag == MoveFlag::KS_CASTLE || flag == MoveFlag::QS_CASTLE) {
        let rook = pos.castling_rooks()[usize::from(flag == MoveFlag::KS_CASTLE)] as u16
            + 56 * (from_idx / 56);
        format!("{from}{}", index_to_square(rook))
    } else {
        let promo = if flag >= MoveFlag::KNIGHT_PROMO {
            ["n", "b", "r", "q"][usize::from((flag >> 12) & 0b11)]
        } else {
            ""
        };
        format!("{from}{to}{promo} ")
    }
}

#[derive(Default)]
pub struct PvLine(Vec<u16>);

impl PvLine {
    pub fn with_capacity(capacity: i8) -> Self {
        Self(Vec::with_capacity(capacity as usize))
    }

    #[inline]
    pub fn first(&self) -> u16 {
        *self.0.first().unwrap_or(&0)
    }

    pub fn to_string(&self, pos: &Position) -> String {
        self.0.iter().map(|&m| u16_to_uci(pos, m)).collect()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.0.clear();
    }

    #[inline]
    pub fn update(&mut self, m: u16, sub_pv: &mut PvLine) {
        self.clear();
        self.0.push(m);
        self.0.append(&mut sub_pv.0);
    }
}

pub fn perft<const SPLIT: bool>(pos: &mut Position, depth: u8) -> u64 {
    let moves = pos.generate::<{ MoveType::ALL }>();
    let mut positions = 0;
    for i in 0..moves.len() {
        let m = moves[i].r#move();
        if pos.r#do(m) {
            continue;
        }

        let count = if depth > 1 {
            perft::<false>(pos, depth - 1)
        } else {
            1
        };

        pos.undo();

        positions += count;
        if SPLIT {
            println!("{}: {count}", u16_to_uci(pos, m));
        }
    }
    positions
}
