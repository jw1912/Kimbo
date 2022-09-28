use super::{Engine, sorting::{MoveScores, get_next_move}};
use crate::position::{MoveType, MoveList};

/// Comments:
/// UCI: implemented for the uci protocol / debug stats

impl Engine {
    /// Quiescence search
    /// 
    /// Constant parameters:
    ///  - STATS - are debug stats required?
    pub fn quiesce<const STATS: bool>(&mut self, mut alpha: i16, beta: i16) -> i16 {
        // static eval
        let stand_pat = self.board.static_eval(&self.ptable);

        // beta pruning
        if stand_pat >= beta {
            return beta;
        }

        // delta pruning
        if stand_pat < alpha - 850 {
            return alpha;
        }

        // improving alpha bound
        if alpha < stand_pat {
            alpha = stand_pat;
        }
        
        // UCI: count qnodes now, as no early prune
        self.stats.node_count += 1;
        if STATS { self.stats.qnode_count += 1; }

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
            let score = -self.quiesce::<STATS>(-beta, -alpha);

            // unmaking move
            self.board.unmake_move();

            // beta pruning
            if score >= beta {
                return beta;
            }

            // improve alpha bound
            if score > alpha {
                alpha = score;
            }
        }
        alpha
    }
}
