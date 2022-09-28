use crate::{io::outputs::u16_to_uci, position::{zobrist::{initialise_zobrist, initialise_pawnhash}}, eval::{calc_pst, calc_material}};
use super::{MoveType, MoveList, Position};

pub fn perft<const ROOT: bool, const DEBUG: bool>(position: &mut Position, depth_left: u8) -> u64 {
    if DEBUG {
        assert_eq!(position.zobrist, initialise_zobrist(position));
        assert_eq!(position.pawnhash, initialise_pawnhash(position));
        assert_eq!(position.pst_mg, calc_pst::<true>(position));
        assert_eq!(position.pst_eg, calc_pst::<false>(position));
        assert_eq!(position.mat_mg, calc_material::<true>(position));
        assert_eq!(position.mat_eg, calc_material::<false>(position));
    }
    // leaf node, count 1
    if depth_left == 0 {
        return 1;
    }

    // generate moves
    let mut moves = MoveList::default();
    position.gen_moves::<{ MoveType::ALL }>(&mut moves);

    // bulk counting on depth 1
    if depth_left == 1 {
        return moves.len() as u64;
    }

    // calculate number of positions
    let mut positions: u64 = 0;
    for m_idx in 0..moves.len() {

        let m = moves[m_idx];

        // make move
        position.make_move(m);

        // find number of positions
        let score = perft::<false, DEBUG>(position, depth_left - 1);
        positions += score;

        // unmake move
        position.unmake_move();

        // print positions from this move if root
        if ROOT {
            println!("{}: {}", u16_to_uci(&m), score);
        }
    }
    positions
}
