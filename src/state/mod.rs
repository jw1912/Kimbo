mod consts;
mod movegen;
mod position;

pub use position::Position;
use std::mem::MaybeUninit;

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

pub struct MoveList {
    list: [Move; 252],
    length: usize,
}

impl MoveList {
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

    pub fn score<F: Fn(u16) -> i16>(&mut self, score_move: F) {
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
