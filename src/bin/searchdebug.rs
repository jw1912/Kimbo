use kimbo::engine::EnginePosition;
use kimbo::hash::search::TT;
use kimbo::io::outputs::{display_board, u16_to_uci};
use kimbo::search::Search;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Instant;

pub const _POSITIONS: [&str; 9] = [
    "8/2krR3/1pp3bp/42p1/PPNp4/3P1PKP/8/8 w - - 0 1",
    "rn5r/pp3kpp/2p1R3/5p2/3P4/2B2N2/PPP3PP/2K4n w - - 1 17",
    "4r1rk/pp4pp/2n5/8/6Q1/7R/1qPK1P1P/3R4 w - - 0 28",
    "2r1rbk1/1R3R1N/p3p1p1/3pP3/8/q7/P1Q3PP/7K b - - 0 25",
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", // Start
    "8/k7/3p4/p2P1p2/P2P1P2/8/8/K7 w - -", // Lasker-Reichhelm 
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -", // Kiwipete
    "3rkbr1/2Q2ppp/4p3/8/b2B4/P3P3/2P2PPP/R3KB1R w KQ - 0 17",
    "1Q6/8/8/8/2k2P2/1p6/1B4K1/8 w - - 3 63",
];

fn _search_all() {
    // params
    let max_time = 1000;
    let max_depth = u8::MAX;
    let tt = Arc::new(TT::new(32 * 1024 * 1024));
    let now = Instant::now();
    for (i, position ) in _POSITIONS.iter().enumerate() {
        let mut search: Search = Search::new(
            EnginePosition::from_fen(position),
            Arc::new(AtomicBool::new(false)),
            max_time,
            max_depth,
            u64::MAX,
            tt.clone(),
            i as u8,
        );
        display_board::<true>(&search.position.board);
        search.go::<true>();
        println!("best move {}", u16_to_uci(&search.best_move));
    }
    println!("Total time: {}ms", now.elapsed().as_millis());
}

fn _search_one(pos: usize) {
    // params
    let max_time = 10000;
    let max_depth = u8::MAX;
    let tt = Arc::new(TT::new(32 * 1024 * 1024));
    let mut search: Search = Search::new(
        EnginePosition::from_fen(_POSITIONS[pos]),
        Arc::new(AtomicBool::new(false)),
        max_time,
        max_depth,
        u64::MAX,
        tt,
        0,
    );
    display_board::<true>(&search.position.board);
    search.go::<true>();
    println!("best move {}", u16_to_uci(&search.best_move));
}

fn main() {
    _search_one(8)
}
