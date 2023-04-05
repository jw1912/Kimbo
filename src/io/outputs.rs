use super::FILES;
use crate::position::{MoveList, Position};
use crate::search::{is_mate_score, MAX_SCORE};

/// board idx to square
pub fn idx_to_sq(idx: u16) -> String {
    let rank = idx >> 3;
    let file = idx & 7;
    let srank = (rank + 1).to_string();
    let sfile = FILES[file as usize];
    format!("{sfile}{srank}")
}

/// u16 move format to uci move format
const PROMOS: [&str; 4] = ["n", "b", "r", "q"];
const PROMO_BIT: u16 = 0b1000_0000_0000_0000;
pub fn u16_to_uci(m: &u16) -> String {
    let mut promo = "";
    if m & PROMO_BIT > 0 {
        promo = PROMOS[((m >> 12) & 0b11) as usize];
    }
    format!(
        "{}{}{} ",
        idx_to_sq(m & 0b111111),
        idx_to_sq((m >> 6) & 0b111111),
        promo
    )
}

/// returns info on the search
pub fn uci_info(
    depth: i8,
    seldepth: i8,
    nodes: u64,
    time: u128,
    pv: Vec<u16>,
    eval: i16,
    hashfull: u64,
) {
    let pv_str: String = pv.iter().map(u16_to_uci).collect();
    let mut score_type = "cp";
    let mut score = eval;
    if is_mate_score(eval) {
        score_type = "mate";
        score = if eval < 0 {
            eval.abs() - MAX_SCORE
        } else {
            MAX_SCORE - eval + 1
        } / 2;
    }
    let nps = if time != 0 {
        ((nodes as f64) / ((time as f64) / 1000.0)) as u32
    } else {
        nodes as u32 * 1000
    };
    println!(
        "info depth {} seldepth {} score {} {} time {} nodes {} nps {} hashfull {} pv {}",
        depth, seldepth, score_type, score, time, nodes, nps, hashfull, pv_str
    );
}

// getting symbols for pieces
const PIECE_SYMBOLS: [&str; 13] = [
    " ", "P", "N", "B", "R", "Q", "K", "p", "n", "b", "r", "q", "k",
];
fn symbol_at_idx(idx: usize, pos: &Position) -> &str {
    let indx = match pos.squares[idx] {
        6 => 0,
        _ => ((pos.squares[idx] + 1) as u64 + 6 * ((pos.sides[1] >> idx) & 1)) as usize,
    };
    PIECE_SYMBOLS[indx]
}

/// Prints the current board in pretty format
pub fn display_board(pos: &Position) {
    println!("+---+---+---+---+---+---+---+---+");
    for i in 1..9 {
        let mut line: String = String::from("| ");
        for j in 0..8 {
            let idx = 64 - i * 8 + j;
            line.push_str(symbol_at_idx(idx, pos));
            line.push_str(" | ");
        }
        println!("{}", line);
        println!("+---+---+---+---+---+---+---+---+");
    }
}

pub fn display_movelist(moves: &MoveList) {
    for i in 0..moves.len() {
        println!("{}", u16_to_uci(&moves[i]))
    }
}

pub fn report_stats(pos: &Position) {
    println!("fen: {}", pos.to_fen());
    println!("halfmove counter: {}", pos.halfmove_clock);
    println!("fullmove counter: {}", pos.fullmove_counter);
    println!("state stack length: {}", pos.state_stack.len());
}

pub fn output_move_and_score(m: u16, s: i16, eval: i16) {
    let score_type = match s {
        30000 => "hash move",
        1000..=5500 => "capture",
        600..=900 => "promotion",
        500 => "killer move",
        400 => "counter move",
        300 => "castle",
        0 => "quiet",
        _ => "history",
    };
    println!("{}: {} ({}), eval: {}", u16_to_uci(&m), s, score_type, eval)
}
