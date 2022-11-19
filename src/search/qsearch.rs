use super::{Engine, sorting::{MoveScores, get_next_move}};
use crate::position::{MoveType, MoveList};

impl Engine {
    /// Quiescence search
    ///
    /// fail-hard
    pub fn quiesce(&mut self, mut alpha: i16, beta: i16) -> i16 {
        // UCI: count qnodes now, as no early prune
        self.stats.node_count += 1;

        // static eval
        let mut stand_pat = self.board.static_eval(&self.ptable);

        // beta pruning
        if stand_pat >= beta {
            return stand_pat;
        }

        // delta pruning
        if stand_pat < alpha - 850 {
            return stand_pat + 850;
        }

        // improving alpha bound
        if alpha < stand_pat {
            alpha = stand_pat;
        }

        // generating captures
        let mut captures = MoveList::default();
        self.board.gen_moves::<{ MoveType::CAPTURES }>(&mut captures);

        // scoring captures
        let mut move_scores = MoveScores::default();
        self.score_captures(&captures, &mut move_scores);

        // going through captures
        while let Some((m, _, _)) = get_next_move(&mut captures, &mut move_scores) {
            // making move
            self.board.make_move(m);

            // getting score
            let score = -self.quiesce(-beta, -alpha);

            // unmaking move
            self.board.unmake_move();

            if score > stand_pat {
                stand_pat = score;
                if score > alpha {
                    alpha = score;
                    if score >= beta { return score }
                }
            }
        }
        stand_pat
    }
}
