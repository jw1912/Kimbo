fn index_to_uci(idx: u16) -> String {
    let rank = idx >> 3;
    let file = idx & 7;
    let srank = (rank+1).to_string();
    let sfile = match file {
        0 => "a",
        1 => "b",
        2 => "c",
        3 => "d",
        4 => "e",
        5 => "f",
        6 => "g",
        7 => "h",
        _ => panic!("")
    };
    format!("{sfile}{srank}")
}
/// u16 move format to uci move format
pub fn u16_to_uci(m: &u16) -> String {
    format!("{}{}", index_to_uci(m & 0b111111), index_to_uci((m>>6) & 0b111111))
}