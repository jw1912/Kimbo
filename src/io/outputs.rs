use std::sync::atomic::{AtomicUsize, Ordering};

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
pub fn uci_info(depth: u8, nodes: &AtomicUsize, time: u32, pv: Vec<u16>, eval: i16) {
    let pv_str: String = pv.iter().map(u16_to_uci).collect();
    // need to add mate score possibility
    println!("info depth {} score cp {} time {} nodes {} pv {}", depth, eval, time, nodes.load(Ordering::SeqCst), pv_str);
}