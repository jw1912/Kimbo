use kimbo_state::{MoveType, movelist::MoveList};
use crate::engine::EnginePosition;
use super::errors::UciError;

fn square_to_index(sq: &str) -> Result<u16, UciError> {
    let chs: Vec<char> = sq.chars().collect();
    let file = match chs[0] {
        'a' => 0,
        'b' => 1,
        'c' => 2,
        'd' => 3,
        'e' => 4,
        'f' => 5,
        'g' => 6,
        'h' => 7,
        _ => return Err(UciError::Move),
    };
    let rank = chs[1].to_string().parse::<u16>()? - 1;
    Ok(8 * rank + file)
}

const TWELVE: u16 = 0b1111_1111_1111;

pub fn uci_to_u16(pos: &EnginePosition, m: &str) -> Result<u16, UciError> {
    let l = m.len();
    if !(l == 4 || l == 5) {
        return Err(UciError::Move)
    }
    let from = square_to_index(&m[0..2])?;
    let to = square_to_index(&m[2..4])?;
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
    pos.board.gen_moves::<{ MoveType::ALL }>(&mut kimbo_state::Check::None, &mut possible_moves);
    for m_idx in 0..possible_moves.len() {
        let um = possible_moves[m_idx];
        if no_flags & TWELVE == um & TWELVE {
            if l < 5 {
                return Ok(um);
            }
            if no_flags & !TWELVE == um & !TWELVE & 0b1011_0000_0000_0000 {
                return Ok(um);
            }
        }
    }
    Err(UciError::Move)
}
