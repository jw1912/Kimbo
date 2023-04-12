use super::{consts::*, qsearch::qsearch, Engine, PvLine, Score, MAX_PLY};
use crate::{
    state::{MoveFlag, MoveType},
    tables::Bound,
};

fn score_move(engine: &Engine, r#move: u16, hash_move: u16) -> i16 {
    if r#move == hash_move {
        MoveScore::HASH
    } else if r#move & MoveFlag::CAPTURE > 0 {
        let occ = engine.position.occ();
        let victim = engine.position.get_piece(1 << (r#move & 63), occ);
        let attacker = engine.position.get_piece(1 << ((r#move >> 6) & 63), occ);
        MoveScore::MVV_LVA[victim][attacker]
    } else {
        MoveScore::QUIET
    }
}

pub fn search(
    engine: &mut Engine,
    mut alpha: i16,
    beta: i16,
    depth: i8,
    in_check: bool,
    pv_line: &mut PvLine,
) -> i16 {
    // check if need to end the search,
    // or if search is already ending
    if engine.limits.aborting() || engine.limits.should_abort(engine.nodes) {
        return Score::ABORT;
    }

    // clear pv line for new node
    pv_line.clear();

    // draw detection
    if engine.position.is_draw(engine.ply) {
        return Score::DRAW;
    }

    // drop into quiescence search if depth is 0
    // or if maximum ply is reached
    if depth <= 0 || engine.ply == MAX_PLY {
        return qsearch(engine, alpha, beta);
    }

    // count node as this node is not a quiescent node
    engine.nodes += 1;

    // necessary information to track for this node
    let pv_node = beta > alpha + 1;
    let zobrist = engine.position.hash();
    let mut hash_move = 0;
    let mut write_to_hash = true;

    // probing hash table
    if let Some(entry) = engine.hash_table.probe(zobrist, engine.ply) {
        write_to_hash = depth > entry.depth;
        hash_move = entry.r#move;

        // hash score pruning
        if !pv_node
            && entry.depth >= depth
            && match entry.bound {
                Bound::LOWER => entry.score >= beta,
                Bound::UPPER => entry.score <= alpha,
                _ => true,
            }
        {
            return entry.score;
        }
    }

    // generate and score moves
    let mut moves = engine.position.generate::<{ MoveType::ALL }>();
    moves.score(|r#move| score_move(engine, r#move, hash_move));

    // necessary information to track for going through moves
    let mut best_move = hash_move;
    let mut best_score = -Score::MAX;
    let mut bound = Bound::UPPER;
    let mut legal_moves = 0;
    let sub_pv = &mut PvLine::default();
    let do_lmr = depth > 1 && engine.ply > 0 && !in_check;

    // increment ply for next depth
    engine.ply += 1;

    // go through moves
    while let Some(r#move) = moves.pick() {
        // make move and skip if illegal
        if engine.position.r#do(r#move.r#move()) {
            continue;
        }

        legal_moves += 1;
        let check = engine.position.in_check();
        let reduce = i8::from(do_lmr && !check && r#move.score() == MoveScore::QUIET);

        // principle variation search of move
        #[rustfmt::skip]
        let score = if legal_moves == 1 {
            -search(engine, -beta, -alpha, depth - 1, check, sub_pv)
        } else {
            let null_window_score =
                -search(engine, -alpha - 1, -alpha, depth - 1 - reduce, check, sub_pv);
            if pv_node && null_window_score > alpha {
                -search(engine, -beta, -alpha, depth - 1, check, sub_pv)
            } else {
                null_window_score
            }
        };

        // undo move
        engine.position.undo();

        // best move so far?
        if score > best_score {
            best_score = score;
            best_move = r#move.r#move();

            // improve alpha?
            if score > alpha {
                alpha = score;
                bound = Bound::EXACT;

                // update pv
                pv_line.update(best_move, sub_pv);

                // beta pruning?
                if score >= beta {
                    bound = Bound::LOWER;
                    break;
                }
            }
        }
    }

    // restore ply
    engine.ply -= 1;

    // no legal moves? (stale)mate
    if legal_moves == 0 {
        return i16::from(in_check) * (engine.ply - Score::MAX);
    }

    // writing to hash table
    if write_to_hash && !engine.limits.aborting() {
        engine
            .hash_table
            .push(zobrist, best_move, depth, bound, best_score, engine.ply);
    }

    best_score
}
