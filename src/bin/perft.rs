use kimbo::engine::EnginePosition;
use kimbo::perft::PerftSearch;
use kimbo::hash::perft::PerftTT;
use std::sync::Arc;
use std::time::Instant;

pub const NUM_TESTS: usize = 7;

pub const TESTS: [&str; NUM_TESTS] = [
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 1 1",
    "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 1 1",
    "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
    "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
    "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
    "1Q6/8/8/8/2k2P2/1p6/1B4K1/8 w - - 3 63"
];

pub const TESTS_EXPECTED: [&[u64];NUM_TESTS] = [
    &[20,400,8902,197281,4865609,119060324],
    &[48,2039,97862,4085603,193690690],
    &[14,191,2812,43238,674624,11030083],
    &[6,264,9467,422333,15833292],
    &[44,1486,62379,2103487,89941194],
    &[46,2079,89890,3894594],
    &[34, 115, 3907, 18736, 626398, 3307800, 106824961,703134803]
];

fn _perft_tests() {
    let tt = Arc::new(PerftTT::new(1024 * 1024));
    let start = Instant::now();
    for (i, expected) in TESTS_EXPECTED.iter().enumerate() {
        let len: usize = expected.len();
        let now = Instant::now();
        let mut total = 0;
        let mut results = Vec::new();
        let mut search: PerftSearch = PerftSearch::new(
            EnginePosition::from_fen(TESTS[i]),
            tt.clone(),
        );
        for j in 0..len {
            let new_tt = Arc::new(PerftTT::new(32 * 1024 * 1024));
            search.ttable = new_tt;
            let count = search.perft::<true>((j + 1) as u8);
            results.push(count);
            total += count;
        }
        let time = now.elapsed().as_millis() as u64;
        println!(" ");
        println!("Test {} complete in {}ms.", i+1, time);
        println!("Nodes: {}, NPS: {}", total, total * 1000 / time);
        search.stats.report();
        search.stats.reset();
        
        assert_eq!(results, *expected, "Failed at test {}.", i+1);
    }
    println!("Took {}ms", start.elapsed().as_millis());
}

fn _root() {
    let now = Instant::now();
    let tt = Arc::new(PerftTT::new(32 * 1024 * 1024));
    let mut search: PerftSearch = PerftSearch::new(
        EnginePosition::default(),
        tt,
    );
    let count = search.perft::<true>(7);
    println!("Nodes: {count}, Time: {}ms", now.elapsed().as_millis());
}

fn main() {
    _perft_tests()
}