use super::PerftSearch;
use crate::{engine::EngineMoveContext, io::outputs::u16_to_uci};
use kimbo_state::MoveType;

impl PerftSearch {
    pub fn perft<const TT_ACTIVE: bool>(&mut self, depth_left: u8) -> u64 {
        if depth_left == 0 {
            return 1;
        }
        // probing transposition table
        let zobrist = self.position.zobrist;
        if TT_ACTIVE {
            if let Some(res) = self.ttable.get(zobrist, depth_left) {
                self.stats.tt_hits += 1;
                return res.count;
            }
        }

        // bulk counting on depth 1
        let moves = self.position.board.gen_moves::<{ MoveType::ALL }>();
        if depth_left == 1 {
            return moves.len() as u64;
        }

        // calculate number of positions
        let mut positions: u64 = 0;
        let mut ctx: EngineMoveContext;
        for m in moves {
            ctx = self.position.make_move(m);
            positions += self.perft::<TT_ACTIVE>(depth_left - 1);
            self.position.unmake_move(ctx);
        }

        // push position info to tt
        if TT_ACTIVE {
            self.ttable.push(zobrist, positions, depth_left);
        }

        positions
    }

    pub fn go(&mut self, depth: u8) -> u64 {
        // works like stockfish's perft function
        if depth == 0 {
            return 1;
        }
        let mut new_move_list: Vec<(u16, u64)> = Vec::new();
        let move_list = self.position.board.gen_moves::<{ MoveType::ALL }>();
        let mut ctx: EngineMoveContext;
        let mut score: u64;
        for mo in move_list {
            ctx = self.position.make_move(mo);
            score = self.perft::<true>(depth - 1);
            self.position.unmake_move(ctx);
            new_move_list.push((mo, score));
            println!("{}: {}", u16_to_uci(&mo), score);
        }
        let mut positions: u64 = 0;
        for sm in new_move_list {
            positions += sm.1;
        }
        positions
    }
}