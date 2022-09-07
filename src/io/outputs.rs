use kimbo_state::Position;

/// board index to square
fn index_to_square(idx: u16) -> String {
    let rank = idx >> 3;
    let file = idx & 7;
    let srank = (rank + 1).to_string();
    let sfile = match file {
        0 => "a",
        1 => "b",
        2 => "c",
        3 => "d",
        4 => "e",
        5 => "f",
        6 => "g",
        7 => "h",
        _ => panic!(""),
    };
    format!("{sfile}{srank}")
}

/// u16 move format to uci move format
pub fn u16_to_uci(m: &u16) -> String {
    format!(
        "{}{} ",
        index_to_square(m & 0b111111),
        index_to_square((m >> 6) & 0b111111)
    )
}

/// returns info on the search
pub fn uci_info(depth: u8, nodes: u64, time: u64, pv: Vec<u16>, eval: i16) {
    let pv_str: String = pv.iter().map(u16_to_uci).collect();
    // need to add mate score possibility
    println!("info depth {} score cp {} time {} nodes {} nps {} pv {}", depth, eval, time, nodes, ((nodes as f64) / ( (time as f64) / 1000.0 )) as u32, pv_str);
}

const PIECE_SYMBOLS: [&str; 13] = [" ", "P", "N", "B", "R", "Q", "K", "p", "n", "b", "r", "q", "k"];
const PIECE_SYMBOLS_FANCY: [&str; 13] = [" ", "♟", "♞", "♝", "♜", "♛", "♚", "♙", "♘", "♗", "♖", "♕", "♔"];

fn symbol_at_idx<const FANCY: bool>(idx: usize, pos: &Position) -> &str {
    let idx = match pos.squares[idx] {
        7 => 0,
        _ => (  (pos.squares[idx] + 1) as u64 + 6 * ((pos.sides[1] >> idx) & 1) ) as usize
    };
    match FANCY {
        true => PIECE_SYMBOLS_FANCY[idx],
        false => PIECE_SYMBOLS[idx]
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