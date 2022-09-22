use kimbo::search::Engine;
use kimbo::tables::search::HashTable;
use kimbo::tables::pawn::PawnHashTable;
use kimbo::position::zobrist::ZobristVals;
use kimbo::io::outputs::display_board;
use kimbo::position::Position;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Instant;

pub const _POSITIONS: [&str; 12] = [
    // Start Position
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 
    // Lasker-Reichhelm Position
    "8/k7/3p4/p2P1p2/P2P1P2/8/8/K7 w - - 0 1",
    // Kiwipete Position
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
    // Standard low depth mate puzzles
    "rn5r/pp3kpp/2p1R3/5p2/3P4/2B2N2/PPP3PP/2K4n w - - 1 17",
    "4r1rk/pp4pp/2n5/8/6Q1/7R/1qPK1P1P/3R4 w - - 0 28",
    "2r1rbk1/1R3R1N/p3p1p1/3pP3/8/q7/P1Q3PP/7K b - - 0 25",
    // Positions that catch pruning methods out
    "8/2krR3/1pp3bp/6p1/PPNp4/3P1PKP/8/8 w - - 0 1",
    "1Q6/8/8/8/2k2P2/1p6/1B4K1/8 w - - 3 63",
    "3r2k1/pp3ppp/4p3/8/QP6/P1P5/5KPP/7q w - - 0 27",
    "1q1r3k/3P1pp1/ppBR1n1p/4Q2P/P4P2/8/5PK1/8 w - - 0 1",
    "1n3r2/3k2pp/pp1P4/1p4b1/1q3B2/5Q2/PPP2PP1/R4RK1 w - - 0 1",
    "7K/8/k1P5/7p/8/8/8/8 w - - 0 1"
];

fn _search_all() {
    // params
    let max_time = 1000;
    let max_depth = i8::MAX;
    let tt = Arc::new(HashTable::new(32 * 1024 * 1024));
    let pt = Arc::new(PawnHashTable::new(1024 * 1024));
    let zvals = Arc::new(ZobristVals::default());
    let now = Instant::now();
    for (i, pos ) in _POSITIONS.iter().enumerate() {
        let position =  Position::from_fen(*pos, zvals.clone()).unwrap();
        let mut search= Engine::new(
            position,
            Arc::new(AtomicBool::new(false)),
            max_time,
            max_depth,
            u64::MAX,
            tt.clone(),
            pt.clone(),
            i as u8,
        );
        assert_eq!(String::from(*pos), search.board.to_fen());
        display_board::<true>(&search.board);
        println!("fen: {}", pos);
        search.go::<true, true>();
        println!(" ");
    }
    println!("Total time: {}ms", now.elapsed().as_millis());
}

fn _search_one(pos: usize) {
    // params
    let max_time = 5000;
    let max_depth = i8::MAX;
    let tt = Arc::new(HashTable::new(32 * 1024 * 1024));
    let pt = Arc::new(PawnHashTable::new(1024 * 1024));
    let zvals = Arc::new(ZobristVals::default());
    let position = Position::from_fen(_POSITIONS[pos], zvals).unwrap();
    let mut search = Engine::new(
        position,
        Arc::new(AtomicBool::new(false)),
        max_time,
        max_depth,
        u64::MAX,
        tt,
        pt,
        0,
    );
    display_board::<true>(&search.board);
    println!("fen: {}", _POSITIONS[pos]);
    search.go::<true, true>();
}

fn main() {
    _search_one(0)
}