use std::{cell::RefCell, ops::Range};

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

    #[serde(skip)]
    is_powered_cache: RefCell<Vec<bool>>,
}

impl Board {
    pub fn empty() -> Self {
        Self {
            depots: vec![vec![]; NUM_DEPOTS],
            selected: None,
            num_energy: 0,
            animation_acts: vec![],
            is_powered_cache: RefCell::new(vec![]),
        }
    }

    pub fn from_deal(deal: &[Card]) -> Self {
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

    fn populate_cache(&self) {
        let mut vis = vec![false; NUM_DEPOTS];
        let mut stk = vec![];
        for id in DepotRole::Source.range() {
            vis[id] = true;
            stk.push(id);
        }

        while let Some(u) = stk.pop() {
            let Some(pos) = depot_to_grid_pos(u) else {continue};
            let Some(&cu) = self.depots[u].first() else {continue};

            for &dir in cu.connections() {
                let Some(v) = grid_pos_to_depot(pos + dir) else {continue};
                if vis[v] { continue; }
                let Some(&cv) = self.depots[v].first() else {continue};
                if !cv.connections().contains(&!dir) { continue; }
                vis[v] = true;
                stk.push(v);
            }
        }

        *self.is_powered_cache.borrow_mut() = vis;
    }

    pub fn invalidate_cache(&mut self) {
        *self.is_powered_cache.borrow_mut() = vec![];
    }

    pub fn is_powered(&self, depot: usize) -> bool {
        if self.is_powered_cache.borrow().is_empty() {
            self.populate_cache();
        }

        self.is_powered_cache.borrow()[depot]
    }

    // NOTE: works differently than most games; moves only one card, but may swap with another card
    pub fn do_move(&mut self, pos1: BoardPos, pos2: BoardPos) {
        self.selected = None;

        let Some(card) = self.depots[pos1.depot_index].drain(pos1.card_index ..).next() else {return};
        let card2 = self.depots[pos2.depot_index].drain(pos2.card_index ..).next();
        self.animation_acts.push(AnimationAct::Move(card, pos1, pos2));
        if let Some(card2) = card2 {
            self.animation_acts.push(AnimationAct::Move(card2, pos2, pos1));
        }
        self.invalidate_cache();
    }

    pub fn advance_actions(&mut self) {
        for act in self.animation_acts.drain(..) {
            match act {
                AnimationAct::Move (card, _, pos2) => {
                    self.depots[pos2.depot_index].push(card);
                },
            }
        }
        self.invalidate_cache();
    }

    pub fn top_pos(&self, depot: usize) -> BoardPos {
        BoardPos::new(depot, self.depots[depot].len())
    }

    pub fn last_pos(&self, depot: usize) -> BoardPos {
        BoardPos::new(depot, self.depots[depot].len().wrapping_sub(1))
    }

    pub fn is_card_moving_to_foundation(&self) -> bool {
        self.animation_acts.iter().any(|act| {
            match act {
                &AnimationAct::Move(_card, _pos1, pos2) => 
                    DepotRole::role(pos2.depot_index) == Some(DepotRole::Foundation),
            }
        })
    }
}