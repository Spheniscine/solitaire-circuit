use std::ops::{Add, Not};

use crate::game::Card;

use super::Suit;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    North, East, South, West,
}

impl Not for Direction {
    type Output = Self;

    fn not(self) -> Self::Output {
        use Direction::*;
        match self {
            North => South,
            East => West,
            South => North,
            West => East,
        }
    }
}

/// Array is interpreted as row-major index. Uses wrapping arithmetic. Bounds-checking is done later in the pipeline.
impl Add<Direction> for [usize; 2] {
    type Output = Self;

    fn add(self, rhs: Direction) -> Self::Output {
        let [y, x] = self;
        match rhs {
            Direction::North => [y.wrapping_sub(1), x],
            Direction::East => [y, x.wrapping_add(1)],
            Direction::South => [y.wrapping_add(1), x],
            Direction::West => [y, x.wrapping_sub(1)],
        }
    }
}

impl Card {
    /// Board is rotated 90 degress counterclockwise, compared to ASM's original
    pub fn connections(self) -> &'static [Direction] {
        use Direction::*;
        match self.rank {
            1 => match self.suit {
                Suit::Clubs => &[North],
                Suit::Diamonds => &[East],
                Suit::Hearts => &[South],
                Suit::Spades => &[West],
            }
            2 => &[North, West],
            3 => &[West, South],
            4 => &[South, East],
            5 => &[East, North],
            6 => &[North, South],
            7 => &[West, East],
            8 => &[North, West, South],
            9 => &[West, South, East],
            10 => &[South, East, North],
            11 => &[East, North, West],
            12 => &[North, East, South, West],
            13 => match self.suit {
                Suit::Clubs | Suit::Spades => &[North],
                Suit::Diamonds | Suit::Hearts => &[South],
            }
            _ => panic!("Invalid rank"),
        }
    }
}