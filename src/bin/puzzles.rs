/// Puzzles used for testing
pub const _PUZZLES: [&str; 7] = ["8/2krR3/1pp3bp/42p1/PPNp4/3P1PKP/8/8 w - - 0 1",
                                "rn5r/pp3kpp/2p1R3/5p2/3P4/2B2N2/PPP3PP/2K4n w - - 1 17",
                                "4r1rk/pp4pp/2n5/8/6Q1/7R/1qPK1P1P/3R4 w - - 0 28",
                                "2r1rbk1/1R3R1N/p3p1p1/3pP3/8/q7/P1Q3PP/7K b - - 0 25",
                                "8/1k6/1pp5/7K/8/8/8/8 w - - 0 2",
                                "8/8/8/rrk1K3/8/8/8/8 b - - 0 2",
                                "8/k7/3p4/p2P1p2/P2P1P2/8/8/K7 w - -"]; 

use std::time::Instant;
use kimbo::{engine::EnginePosition, io::outputs::u16_to_uci};
use kimbo_state::*;

fn main() {
    // initialise board with fen string
    let mut game = EnginePosition::from_fen(_PUZZLES[1]);
    // display initial board config
    // move counter
    let now = Instant::now();
    for _ in 0..50 {
        let m = game.analyse(5);
        println!("playing {}", u16_to_uci(&m));
        game.make_move(m);
        // check if game has ended
        let legal_moves = game.board.gen_moves::<{MoveType::ALL}>();
        if legal_moves.is_empty() {
            let side = game.board.side_to_move;
            let idx = ls1b_scan(game.board.pieces[side][5]) as usize;
            // checkmate
            if game.board.is_square_attacked(idx, side, game.board.occupied) {
                println!("Checkmate!");
                break
            }
            // stalemate
            println!("Stalemate!");
            break
        }
    } 
    println!("Took {}ms.", now.elapsed().as_millis());
}