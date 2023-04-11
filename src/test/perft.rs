use std::time::Instant;

use crate::{state::Fens, uci::perft};

/// Mix of FRC and normal fens.
const FENS: [(&str, u8, u64); 12] = [
    (Fens::STARTPOS, 6, 119_060_324),
    (Fens::KIWIPETE, 5, 193_690_690),
    ("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -", 7, 178_633_661),
    ("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq -", 6, 706_045_033),
    ("r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ -", 6, 706_045_033),
    ("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8", 5, 89_941_194),
    ("bqnb1rkr/pp3ppp/3ppn2/2p5/5P2/P2P4/NPP1P1PP/BQ1BNRKR w HFhf - 2 9", 5, 8_146_062),
    ("2nnrbkr/p1qppppp/8/1ppb4/6PP/3PP3/PPP2P2/BQNNRBKR w HEhe - 1 9", 5, 16_253_601),
    ("b1q1rrkb/pppppppp/3nn3/8/P7/1PPP4/4PPPP/BQNNRKRB w GE - 1 9", 6, 177_654_692),
    ("qbbnnrkr/2pp2pp/p7/1p2pp2/8/P3PP2/1PPP1KPP/QBBNNR1R w hf - 0 9", 5, 9_183_776),
    ("1nbbnrkr/p1p1ppp1/3p4/1p3P1p/3Pq2P/8/PPP1P1P1/QNBBNRKR w HFhf - 0 9", 5, 34_030_312),
    ("qnbnr1kr/ppp1b1pp/4p3/3p1p2/8/2NPP3/PPP1BPPP/QNB1R1KR w HEhe - 1 9", 5, 24_851_983),
];

#[test]
fn test_perft() {
    let mut pos;
    let time = Instant::now();
    for (i, &(fen, depth, expected_count)) in FENS.iter().enumerate() {
        pos = fen.parse().expect("hard coded");
        let count = perft::<false>(&mut pos, depth);
        assert_eq!(count, expected_count, "FEN {i}: {fen}");
    }
    println!("NPS: {:.0}",
        FENS.iter().map(|(_, _, count)| count).sum::<u64>() as f64
            / time.elapsed().as_secs_f64(),
    );
}