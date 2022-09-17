use crate::io::outputs::u16_to_uci;
use kimbo_state::{MoveType, MoveList, Position};

pub fn perft<const ROOT: bool>(position: &mut Position, depth_left: u8) -> u64 {
    // leaf node, count 1
    if depth_left == 0 {
        return 1;
    }

    // generate moves
    let mut moves = MoveList::default();
    position.gen_moves::<{ MoveType::ALL }>(&mut kimbo_state::Check::None, &mut moves);

    // bulk counting on depth 1
    if depth_left == 1 {
        return moves.len() as u64;
    }

    // calculate number of positions
    let mut positions: u64 = 0;
    for m_idx in 0..moves.len() {

        let m = moves[m_idx];

        // make move
        let ctx = position.make_move(m);

        // find number of positions
        let score = perft::<false>(position, depth_left - 1);
        positions += score;

        // unmake move
        position.unmake_move(ctx);

        // print positions from this move if root
        if ROOT {
            println!("{}: {}", u16_to_uci(&m), score);
        }
    }
    positions
}
