mod consts;
mod movegen;
mod position;

pub use consts::{Fens, MoveFlag, MoveType};
pub use position::Position;
use std::{mem::MaybeUninit, ops::Index};

/// Move Encoding
/// ```
/// 0b 0000 000000 000000
///    flag   to    from
/// ```
#[derive(Clone, Copy)]
pub struct Move {
    r#move: u16,
    score: i16,
}

impl Move {
    #[inline]
    pub fn r#move(&self) -> u16 {
        self.r#move
    }

    #[inline]
    pub fn score(&self) -> i16 {
        self.score
    }

    #[inline]
    pub fn new(r#move: u16) -> Self {
        Self { r#move, score: 0 }
    }
}

pub fn square_str_to_index(sq: &str) -> Result<u16, String> {
    let chars: Vec<char> = sq.chars().collect();
    Ok(8 * chars[1].to_string().parse::<u16>().unwrap() + chars[0] as u16 - 105)
}

pub struct MoveList {
    list: [Move; 252],
    length: usize,
}

impl Index<usize> for MoveList {
    type Output = Move;

    fn index(&self, index: usize) -> &Self::Output {
        &self.list[index]
    }
}

impl MoveList {
    #[inline]
    pub fn len(&self) -> usize {
        self.length
    }

    /// Creates a new instance with list uninitialised.
    /// # Safety
    /// List accessed only by 'add', 'score' and 'pick' methods,
    /// which guarantee safety.
    pub fn uninit() -> Self {
        Self {
            list: unsafe {
                #[allow(clippy::uninit_assumed_init, invalid_value)]
                MaybeUninit::uninit().assume_init()
            },
            length: 0,
        }
    }

    /// Adds a move to the list.
    #[inline]
    pub fn add(&mut self, mov: u16) {
        self.list[self.length] = Move::new(mov);
        self.length += 1;
    }

    /// Scores move list based on given closure.
    pub fn score<F>(&mut self, score_move: F)
    where F: Fn(u16) -> i16
    {
        for i in 0..self.length {
            self.list[i].score = score_move(self.list[i].r#move);
        }
    }

    /// Picks the best move remaining in the list.
    pub fn pick(&mut self) -> Option<Move> {
        // end of move list
        if self.length == 0 {
            return None;
        }

        // find move with highest score
        let mut idx = 0;
        let mut best = i16::MIN;
        for i in 0..self.length {
            let mov = self.list[i];
            if mov.score > best {
                best = mov.score;
                idx = i;
            }
        }

        // move best move out of list
        self.length -= 1;
        self.list.swap(idx, self.length);
        Some(self.list[self.length])
    }
}
