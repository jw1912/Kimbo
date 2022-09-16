pub mod timings;
mod go;
#[rustfmt::skip]
mod negamax;
mod qsearch;
pub mod sorting;
mod pruning;

pub const MAX_PLY: u8 = u8::MAX;

/// Checkmate stuff
pub const MAX_SCORE: i16 = 30000;
pub const MATE_THRESHOLD: i16 = MAX_SCORE - u8::MAX as i16;
#[inline(always)]
pub fn is_mate_score(score: i16) -> bool {
    score >= MATE_THRESHOLD || score <= -MATE_THRESHOLD
}

// useful functions
fn update_pv(pv: &mut Vec<u16>, m: u16, sub_pv: &mut Vec<u16>) {
    pv.clear();
    pv.push(m);
    pv.append(sub_pv); 
}

fn is_capture(m: u16) -> bool {
    m & 0b0100_0000_0000_0000 > 0
}

fn is_promotion(m: u16) -> bool {
    m & 0b1000_0000_0000_0000 > 0
}

fn is_castling(m: u16) -> bool {
    let flags = m & 0b1111_0000_0000_0000;
    flags == 0b0011_0000_0000_0000 || flags == 0b0010_0000_0000_0000
}
