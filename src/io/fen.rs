// Converting position to fen
// other way is handled in kimbo_state

use super::outputs::idx_to_sq;
use crate::eval::{calc_material, calc_pst, calculate_phase};
use crate::position::{
    zobrist::{initialise_pawnhash, initialise_zobrist, ZobristVals},
    *,
};
use std::sync::Arc;
use std::{fmt, num::ParseIntError};

const PIECES: [char; 12] = ['P', 'N', 'B', 'R', 'Q', 'K', 'p', 'n', 'b', 'r', 'q', 'k'];
const RIGHTS: [u8; 4] = [4, 8, 1, 2];
const RIGHTS_CHAR: [char; 4] = ['K', 'Q', 'k', 'q'];
const SIDES: [char; 2] = ['w', 'b'];

fn piece_out(pc: (usize, usize)) -> char {
    PIECES[pc.1 + pc.0 * 6]
}

fn board(sqs: [u8; 64], sides: [u64; 2]) -> String {
    let mut fen = String::from("");
    for i in 0..8 {
        let row = &sqs[(7 - i) * 8..(8 - i) * 8];
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
                fen.push(piece_out((side, *sq as usize)));
            }
        }
        if empty {
            fen.push_str(&empty_count.to_string());
        }
        if i != 7 {
            fen.push('/');
        }
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

impl Position {
    pub fn to_fen(&self) -> String {
        format!(
            "{} {} {} {} {} {}",
            board(self.squares, self.sides),
            SIDES[self.side_to_move],
            castle_rights(self.castle_rights),
            match self.en_passant_sq {
                0 => String::from("-"),
                _ => idx_to_sq(self.en_passant_sq),
            },
            self.halfmove_clock,
            self.fullmove_counter,
        )
    }
}

#[derive(Debug)]
pub struct FenError;
impl fmt::Display for FenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error parsing 'fen' string")
    }
}
impl From<ParseIntError> for FenError {
    fn from(_: ParseIntError) -> Self {
        Self
    }
}

fn pieces_to_squares(pieces: [[u64; 6]; 2]) -> [u8; 64] {
    let mut squares = [Piece::NONE as u8; 64];
    for side in &pieces {
        for (pc, &bb) in side.iter().enumerate() {
            let mut curr_bb = bb;
            while curr_bb > 0 {
                let ls1b = curr_bb & curr_bb.wrapping_neg();
                let idx = ls1b_scan(ls1b) as usize;
                squares[idx] = pc as u8;
                curr_bb &= curr_bb - 1;
            }
        }
    }
    squares
}

fn piece_in(ch: char) -> Result<(usize, usize), FenError> {
    if let Some(idx) = PIECES.iter().position(|&element| element == ch) {
        return Ok(((idx > 5) as usize, idx - 6 * ((idx > 5) as usize)));
    };
    Err(FenError)
}

fn piece_idxs(s: &str) -> Result<[[Vec<usize>; 6]; 2], FenError> {
    let mut piece_idxs = [
        [
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ],
        [
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ],
    ];
    let mut idx: usize = 63;
    let rows: Vec<&str> = s.split('/').collect();
    if rows.len() != 8 {
        return Err(FenError);
    }
    for row in rows {
        let mut count = 0;
        for ch in row.chars().rev() {
            // if piece, insert it
            if !ch.is_numeric() {
                count += 1;
                let (col, pc) = piece_in(ch)?;
                piece_idxs[col][pc].push(idx);
                if idx > 0 {
                    idx -= 1;
                }
            }
            // skip empty squares
            else {
                let len = ch.to_string().parse::<usize>()?;
                count += len;
                if idx >= len {
                    idx -= len;
                }
            }
        }
        if count != 8 {
            return Err(FenError);
        }
    }
    Ok(piece_idxs)
}

fn get_pieces(s: &str) -> Result<[[u64; 6]; 2], FenError> {
    let indxs = piece_idxs(s)?;
    let mut pieces = [[0; 6]; 2];
    for side in 0..2 {
        for pc in 0..6 {
            for idx in indxs[side][pc].clone() {
                pieces[side][pc] |= 1 << idx;
            }
        }
    }
    Ok(pieces)
}

fn get_sides(pieces: [[u64; 6]; 2]) -> [u64; 2] {
    let mut sides = [0; 2];
    for side in 0..2 {
        for pc in 0..6 {
            sides[side] |= pieces[side][pc];
        }
    }
    sides
}

fn get_castling_rights(s: &str) -> Result<u8, FenError> {
    if s == "-" {
        return Ok(CastleRights::NONE);
    }
    let mut castle = CastleRights::NONE;
    for ch in s.chars() {
        let right = match ch {
            'Q' => CastleRights::WHITE_QS,
            'K' => CastleRights::WHITE_KS,
            'q' => CastleRights::BLACK_QS,
            'k' => CastleRights::BLACK_KS,
            _ => return Err(FenError),
        };
        castle |= right;
    }
    Ok(castle)
}

fn get_square(s: &str) -> Result<u16, FenError> {
    if s == "-" {
        return Ok(0);
    }
    let arr = s.as_bytes();
    if arr.len() != 2 {
        return Err(FenError);
    }
    let x = arr[0] as char;
    let y = arr[1] as char;
    let file = match x {
        'A' | 'a' => 0,
        'B' | 'b' => 1,
        'C' | 'c' => 2,
        'D' | 'd' => 3,
        'E' | 'e' => 4,
        'F' | 'f' => 5,
        'G' | 'g' => 6,
        'H' | 'h' => 7,
        _ => return Err(FenError),
    };
    let rank: u16 = y.to_string().parse::<u16>()? - 1;
    Ok((8 * rank + file) as u16)
}

impl Position {
    /// initialises a position from a fen string
    pub fn from_fen(s: &str, zobrist_vals: Arc<ZobristVals>) -> Result<Self, FenError> {
        // splits fen string by whitespace
        let vec: Vec<&str> = s.split_whitespace().collect();
        if vec.len() < 4 {
            return Err(FenError);
        }
        let pieces = get_pieces(vec[0])?;
        let sides = get_sides(pieces);
        let occupied = sides[0] | sides[1];
        let squares = pieces_to_squares(pieces);
        let side_to_move = match vec[1] {
            "w" => Side::WHITE,
            "b" => Side::BLACK,
            _ => return Err(FenError),
        };
        let castle_rights = get_castling_rights(vec[2])?;
        let en_passant_sq = get_square(vec[3])?;
        let mut halfmove_clock = 0;
        let mut fullmove_counter = 1;
        let l = vec.len();
        if l >= 5 {
            halfmove_clock = vec[4].parse::<u8>()?;
        }
        if l >= 6 {
            fullmove_counter = vec[5].parse::<u16>()?;
        }
        let mut pos = Self {
            pieces,
            sides,
            occupied,
            squares,
            side_to_move,
            castle_rights,
            en_passant_sq,
            halfmove_clock,
            fullmove_counter,
            zobrist_vals,
            state_stack: Vec::new(),
            null_counter: 0,
            pawnhash: 0,
            zobrist: 0,
            phase: 0,
            pst_eg: [0, 0],
            pst_mg: [0, 0],
            mat_eg: [0, 0],
            mat_mg: [0, 0],
        };
        pos.pawnhash = initialise_pawnhash(&pos);
        pos.zobrist = initialise_zobrist(&pos);
        pos.phase = calculate_phase(&pos);
        pos.pst_eg = calc_pst::<false>(&pos);
        pos.pst_mg = calc_pst::<true>(&pos);
        pos.mat_mg = calc_material::<true>(&pos);
        pos.mat_eg = calc_material::<false>(&pos);
        Ok(pos)
    }
}
