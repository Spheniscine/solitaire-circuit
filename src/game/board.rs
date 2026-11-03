use std::ops::Range;

use serde::{Deserialize, Serialize};
use serde_tuple::{Deserialize_tuple, Serialize_tuple};
use strum::{IntoEnumIterator, VariantArray};
use strum_macros::{EnumIter, VariantArray};

use crate::game::{Card, DECK_SIZE, NUM_SUITS, RANK_KING, Suit};

#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq, Eq, EnumIter, VariantArray)]
#[repr(u8)]
pub enum DepotRole {
    Source,
    Grid,
    Foundation,
}

pub const NUM_DEPOTS: usize = {
    let mut sum = 0;
    let mut index = 0;
    while index < DepotRole::VARIANTS.len() {
        sum += DepotRole::VARIANTS[index].number_of();
        index += 1;
    }
    sum
};

impl DepotRole {
    pub const fn number_of(&self) -> usize {
        match self {
            DepotRole::Source => NUM_SUITS,
            DepotRole::Grid => DECK_SIZE - NUM_SUITS,
            DepotRole::Foundation => NUM_SUITS,
        }
    }

    pub const fn offset(self) -> usize {
        let mut sum = 0;
        let mut index = 0;
        loop {
            if index == self as usize { return sum; }
            sum += DepotRole::VARIANTS[index].number_of();
            index += 1;
        }
    }

    pub const fn range(self) -> Range<usize> {
        self.offset() .. self.offset() + self.number_of()
    }

    pub fn role_and_subindex(i: usize) -> Option<(DepotRole, usize)> {
        for role in Self::iter() {
            if role.range().contains(&i) {
                return Some((role, i - role.offset()))
            }
        }
        None
    }

    pub fn role(i: usize) -> Option<DepotRole> {
        Self::role_and_subindex(i).map(|x| x.0)
    }

    pub fn id(self, i: usize) -> usize {
        self.offset() + i
    }
}

pub const GRID_HEIGHT: usize = 8;
pub const GRID_WIDTH: usize = 6;
const _: () = assert!(GRID_HEIGHT * GRID_WIDTH == DepotRole::Grid.number_of());

pub type GridPos = [usize; 2];
pub fn grid_pos_to_depot([r, c]: GridPos) -> Option<usize> {
    if c >= GRID_WIDTH { return None; }
    if r >= 1 && r <= GRID_HEIGHT {
        return Some(DepotRole::Grid.id((r-1) * GRID_WIDTH + c));
    }
    let a = if r == 0 {0} 
        else if r == GRID_HEIGHT + 1 {1}
        else {return None};
    let b = if c == 0 {0}
        else if c == GRID_WIDTH - 1 {1}
        else {return None};
    
    Some(DepotRole::Source.id(a * 2 + b))
}

pub fn depot_to_grid_pos(depot: usize) -> Option<GridPos> {
    let (role, index) = DepotRole::role_and_subindex(depot)?;
    match role {
        DepotRole::Source => {
            let a = index / 2;
            let b = index % 2;

            Some([a * (GRID_HEIGHT + 1), b * (GRID_WIDTH - 1)])
        },
        DepotRole::Grid => {
            let r = index / GRID_WIDTH;
            let c = index % GRID_WIDTH;
            Some([r + 1, c])
        },
        DepotRole::Foundation => None,
    }
}

#[derive(Copy, Clone, Serialize_tuple, Deserialize_tuple, Debug, PartialEq, Eq)]
pub struct BoardPos {
    pub depot_index: usize,
    pub card_index: usize,
}

impl BoardPos {
    pub fn new(depot_index: usize, card_index: usize) -> Self {
        Self { depot_index, card_index }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum AnimationAct {
    Move(Card, BoardPos, BoardPos),
}

pub const MAX_ENERGY: u8 = 3;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Board {
    pub depots: Vec<Vec<Card>>,
    pub selected: Option<BoardPos>,
    pub num_energy: u8,
    pub animation_acts: Vec<AnimationAct>,

    // todo: cache powered state of cards / grid
}

impl Board {
    pub fn empty() -> Self {
        Self {
            depots: vec![vec![]; NUM_DEPOTS],
            selected: None,
            num_energy: 0,
            animation_acts: vec![],
        }
    }

    pub fn from_deal(deal: &[Card]) -> Self {
        use DepotRole::*;
        assert_eq!(deal.len(), DepotRole::Grid.number_of());

        let mut res = Self::empty();
        for (&card, depot) in deal.iter().zip(DepotRole::Grid.range()) {
            res.depots[depot].push(card);
        }

        let king_suits = [
            Suit::Hearts, Suit::Diamonds, Suit::Clubs, Suit::Spades
        ];

        for (suit, depot) in king_suits.into_iter().zip(DepotRole::Source.range()) {
            let card = Card { rank: RANK_KING, suit };
            res.depots[depot].push(card);
        }

        res.num_energy = MAX_ENERGY;

        res
    }
}