// Converting position to fen
// other way is handled in kimbo_state

use kimbo_state::Position;

use crate::engine::Engine;

use super::outputs::idx_to_sq;

const PIECES: [char; 12] = ['P','N','B','R','Q','K','p','n','b','r','q','k'];
const RIGHTS: [u8; 4] = [4,8,1,2];
const RIGHTS_CHAR: [char; 4] = ['K','Q','k','q'];
const SIDES: [char; 2] = ['w', 'b'];

fn piece(pc: (usize, usize)) -> char {
    PIECES[pc.1 + pc.0 * 6]
}

fn board(sqs: [u8; 64], sides: [u64; 2]) -> String {
    let mut fen = String::from("");
    for i in 0..8 {
        let row = &sqs[(7-i)*8 .. (8-i)*8];
        let mut empty_count = 0;
        let mut empty = false;
        for (j, sq) in row.iter().enumerate() {
            if *sq == 6 {
                empty = true;
                empty_count += 1;
            } else {
                if empty {
                    fen.push_str(&empty_count.to_string());
                }
                empty_count = 0;
                empty = false;
                let idx = j + 8 * (7 - i);
                let side = ((sides[1] & (1 << idx)) > 0) as usize;
                fen.push(piece((side, *sq as usize)));
            }           
        }
        if empty {
            fen.push_str(&empty_count.to_string());
        }
        if i != 7 { fen.push('/'); }
    }
    fen
}

fn castle_rights(rights: u8) -> String {
    if rights == 0 {
        return String::from("-");
    }
    let mut s = String::from("");
    for i in 0..4 {
        if rights & RIGHTS[i] > 0 {
            s.push(RIGHTS_CHAR[i]);
        }
    }
    s
}

impl Engine {
    pub fn to_fen(&self) -> String {
        format!("{} {} {} {} {} {}",
            board(self.board.squares, self.board.sides),
            SIDES[self.board.side_to_move],
            castle_rights(self.board.castle_rights),
            match self.board.en_passant_sq {
                0 => String::from("-"),
                _ => idx_to_sq(self.board.en_passant_sq),
            },
            self.board.halfmove_clock,
            self.board.fullmove_counter,
        )
    }
}

pub fn to_fen(pos: &Position) -> String {
    format!("{} {} {} {} {} {}",
        board(pos.squares, pos.sides),
        SIDES[pos.side_to_move],
        castle_rights(pos.castle_rights),
        match pos.en_passant_sq {
            0 => String::from("-"),
            _ => idx_to_sq(pos.en_passant_sq),
        },
        pos.halfmove_clock,
        pos.fullmove_counter,
    )
}
