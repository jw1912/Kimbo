use super::{consts::*, eval::eval, Engine};
use crate::state::MoveType;

fn score_move(engine: &Engine, r#move: u16) -> i16 {
    let occ = engine.position.occ();
    let victim = engine.position.get_piece(1 << (r#move & 63), occ);
    let attacker = engine.position.get_piece(1 << ((r#move >> 6) & 63), occ);
    MoveScore::MVV_LVA[victim][attacker]
}

pub fn qsearch(engine: &mut Engine, mut alpha: i16, beta: i16) -> i16 {
    // count all quiescent nodes towards total
    engine.qnodes += 1;

    // calculate static eval
    let mut eval = eval(&engine.position);

    // quick beta prune?
    if eval >= beta {
        return eval;
    }

    // improve alpha?
    alpha = alpha.max(eval);

    // generate and score captures only
    let mut captures = engine.position.generate::<{ MoveType::CAPTURES }>();
    captures.score(|r#move| score_move(engine, r#move));

    // go through moves
    while let Some(r#move) = captures.pick() {
        // make move and skip if illegal
        if engine.position.r#do(r#move.r#move()) {
            continue;
        }

        // search move
        eval = eval.max(-qsearch(engine, -beta, -alpha));

        // undo move
        engine.position.undo();

        // beta pruning?
        if eval >= beta {
            break;
        }

        // improve alpha?
        alpha = alpha.max(eval);
    }

    eval
}
