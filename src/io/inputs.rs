use kimbo_state::{MoveType, MoveList, Position};
use super::errors::UciError;
use super::FILES;

fn sq_to_idx(sq: &str) -> Result<u16, UciError> {
    let chs: Vec<char> = sq.chars().collect();
    let file: u16 = match FILES.iter().position(|&ch| ch == chs[0]) {
        Some(res) => res as u16,
        None => return Err(UciError::Move),
    };
    let rank = chs[1].to_string().parse::<u16>()? - 1;
    Ok(8 * rank + file)
}

const TWELVE: u16 = 0b0000_1111_1111_1111;

pub fn uci_to_u16(pos: &Position, m: &str) -> Result<u16, UciError> {
    let l = m.len();
    if !(l == 4 || l == 5) {
        return Err(UciError::Move)
    }
    let from = sq_to_idx(&m[0..2])?;
    let to = sq_to_idx(&m[2..4])?;
    let mut no_flags = from | (to << 6);
    if l == 5 {
        no_flags |= match m.chars().nth(4).unwrap() {
            'n' => 0b1000_0000_0000_0000,
            'b' => 0b1001_0000_0000_0000,
            'r' => 0b1010_0000_0000_0000,
            'q' => 0b1011_0000_0000_0000,
            _ => return Err(UciError::Move),
        }
    }
    let mut possible_moves = MoveList::default();
    pos.gen_moves::<{ MoveType::ALL }>(&mut kimbo_state::Check::None, &mut possible_moves);
    for m_idx in 0..possible_moves.len() {
        let um = possible_moves[m_idx];
        if no_flags & TWELVE == um & TWELVE {
            if l < 5 {
                return Ok(um);
            }
            if no_flags & !TWELVE == um & 0b1011_0000_0000_0000 {
                return Ok(um);
            }
        }
    }
    Err(UciError::Move)
}
