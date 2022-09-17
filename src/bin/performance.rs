use kimbo::engine::{Engine, GameState};
use kimbo::engine::zobrist::{ZobristVals, initialise_zobrist};
use kimbo::tables::search::HashTable;
use kimbo::tables::pawn::PawnHashTable;
use kimbo_state::Position;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Instant;

pub const _POSITIONS: [&str; 8] = [
    // Start Position
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 
    // Lasker-Reichhelm Position
    "8/k7/3p4/p2P1p2/P2P1P2/8/8/K7 w - -",
    // Standard low depth mate puzzles
    "rn5r/pp3kpp/2p1R3/5p2/3P4/2B2N2/PPP3PP/2K4n w - - 1 17",
    "4r1rk/pp4pp/2n5/8/6Q1/7R/1qPK1P1P/3R4 w - - 0 28",
    "2r1rbk1/1R3R1N/p3p1p1/3pP3/8/q7/P1Q3PP/7K b - - 0 25",
    // Positions that catch pruning methods out
    "8/2krR3/1pp3bp/42p1/PPNp4/3P1PKP/8/8 w - - 0 1",
    "1Q6/8/8/8/2k2P2/1p6/1B4K1/8 w - - 3 63",
    "3r2k1/pp3ppp/4p3/8/QP6/P1P5/5KPP/7q w - - 0 27"
];

fn main() {
    // params
    let max_time = 1000;
    let max_depth = u8::MAX;
    let tt = Arc::new(HashTable::new(32 * 1024 * 1024));
    let pt = Arc::new(PawnHashTable::new(4 * 1024 * 1024));
    let zvals = Arc::new(ZobristVals::default());
    let now = Instant::now();
    for (i, position ) in _POSITIONS.iter().enumerate() {
        let pos =  Position::from_fen(*position).unwrap();
        let mut search = Engine::new(
            pos,
            Arc::new(AtomicBool::new(false)),
            max_time,
            max_depth,
            u64::MAX,
            tt.clone(),
            pt.clone(),
            zvals.clone(),
            vec![GameState::new(initialise_zobrist(&pos, &zvals))],
            i as u8,
        );
        println!("===Search Report===");
        println!("fen: {}", position);
        search.go::<false, true>();
        println!(" ");
    }
    println!("Total time: {}ms", now.elapsed().as_millis());
}