use kimbo_state::Position;
use crate::search::{MAX_SCORE, is_mate_score};
use super::FILES;

/// board idx to square
fn idx_to_sq(idx: u16) -> String {
    let rank = idx >> 3;
    let file = idx & 7;
    let srank = (rank + 1).to_string();
    let sfile = FILES[file as usize];
    format!("{sfile}{srank}")
}

/// u16 move format to uci move format
const PROMOS: [&str; 4] = ["n","b","r","q"];
const PROMO_BIT: u16 = 0b1000_0000_0000_0000;
pub fn u16_to_uci(m: &u16) -> String {
    let mut promo = "";
    if m & PROMO_BIT > 0 {
        promo = PROMOS[((m >> 12) & 0b11) as usize];
    }
    format!("{}{}{} ", idx_to_sq(m & 0b111111), idx_to_sq((m >> 6) & 0b111111), promo)
}

/// returns info on the search
pub fn uci_info(depth: u8, seldepth: u8, nodes: u64, time: u128, pv: Vec<u16>, eval: i16, hashfull: u64) {
    let pv_str: String = pv.iter().map(u16_to_uci).collect();
    let mut score_type = "cp";
    let mut score = eval;
    if is_mate_score(eval) {
        score_type = "mate";
        score = if eval < 0 { eval.abs() - MAX_SCORE } else { MAX_SCORE - eval + 1 } / 2;
    }
    let nps = ((nodes as f64) / ((time as f64) / 1000.0)) as u32;
    println!(
        "info depth {} seldepth {} score {} {} time {} nodes {} nps {} hashfull {} pv {}",
        depth, seldepth, score_type, score, time, nodes, nps, hashfull, pv_str
    );
}

// getting symbols for pieces
const PIECE_SYMBOLS: [&str; 13] = [" ", "P", "N", "B", "R", "Q", "K", "p", "n", "b", "r", "q", "k"];
const PIECE_SYMBOLS_FANCY: [&str; 13] = [" ", "♟", "♞", "♝", "♜", "♛", "♚", "♙", "♘", "♗", "♖", "♕", "♔"];
fn symbol_at_idx<const FANCY: bool>(idx: usize, pos: &Position) -> &str {
    let indx = match pos.squares[idx] {
        6 => 0,
        _ => ((pos.squares[idx] + 1) as u64 + 6 * ((pos.sides[1] >> idx) & 1)) as usize,
    };
    match FANCY {
        true => PIECE_SYMBOLS_FANCY[indx],
        false => PIECE_SYMBOLS[indx],
    }
}

/// Prints the current board in pretty format
pub fn display_board<const FANCY: bool>(pos: &Position) {
    println!("+---+---+---+---+---+---+---+---+");
    for i in 1..9 {
        let mut line: String = String::from("| ");
        for j in 0..8 {
            let idx = 64 - i * 8 + j;
            line.push_str(symbol_at_idx::<FANCY>(idx, pos));
            line.push_str(" | ");
        }
        println!("{}", line);
        println!("+---+---+---+---+---+---+---+---+");
    }
}
