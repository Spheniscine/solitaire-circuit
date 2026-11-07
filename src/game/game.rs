use std::time::Duration;

use rand::{Rng, seq::SliceRandom};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use crate::game::{Board, BoardPos, Card, DepotRole, MAX_ENERGY, NUM_RANKS, RANK_KING, RANK_MIN, Skin, Suit};

pub const ANIMATION_DURATION: Duration = Duration::from_millis(200);
pub type AnimationKey = u16;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum ActionRecord {
    Move(BoardPos, BoardPos),
    Energy(u8),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ScreenState {
    #[default] Game, 
    Settings, Help,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct GameState {
    pub board: Board,
    pub deal: Vec<Card>,
    #[serde(skip)]
    pub animation_key: AnimationKey, // used for syncing and to provide animator components with cycling keys
    pub history: Vec<ActionRecord>,
    pub undo_stack: Vec<usize>,
    pub already_won: bool,
    pub num_wins: i32,

    pub screen_state: ScreenState,

    pub allow_undo: bool,
    pub skin: Skin,
}

impl GameState {
    pub fn new_deal(rng: &mut impl Rng) -> Vec<Card> {
        // kings excluded from deal
        let mut deck = Vec::with_capacity(DepotRole::Grid.number_of());
        for rank in RANK_MIN .. RANK_KING {
            for suit in Suit::iter() {
                deck.push(Card { rank, suit });
            }
        }

        deck.shuffle(rng);
        deck
    }

    pub fn init() -> Self {
        let mut res = Self {
            board: Board::empty(),
            deal: vec![],
            animation_key: 0,
            history: vec![],
            undo_stack: vec![],
            already_won: false,
            num_wins: 0,
            screen_state: ScreenState::Game,
            allow_undo: true,
            skin: Skin::default(),
        };

        res.new_game();
        res
    }

    pub fn new_game(&mut self) {
        let deal = Self::new_deal(&mut rand::rng());
        self.board = Board::from_deal(&deal);
        self.deal = deal;
        self.history.clear();
        self.undo_stack.clear();
        self.already_won = false;

        // if !self.is_busy() { LocalStorage.save_game_state(&self); }
    }

    pub fn can_select(&mut self, pos: BoardPos) -> bool {
        let depot = pos.depot_index;
        let ord = pos.card_index;

        if ord >= self.board.depots[depot].len() {
            return false;
        }
        // let slice = &self.board.depots[depot][ord..];

        let Some(role) = DepotRole::role(depot) else { return false };
        match role {
            DepotRole::Grid => {
                self.board.num_energy == MAX_ENERGY || self.board.is_powered(depot)
            }
            _ => false,
        }
    }

    pub fn is_won(&self) -> bool {
        DepotRole::Foundation.range().all(|d| self.board.depots[d].len() == NUM_RANKS - 1)
    }

    pub fn is_busy(&self) -> bool {
        self.is_acting()
    }

    pub fn is_acting(&self) -> bool {
        !self.board.animation_acts.is_empty()
    }

    pub fn undo_possible(&self) -> bool {
        self.allow_undo && !self.undo_stack.is_empty()
    }

    fn do_move_raw(&mut self, pos1: BoardPos, pos2: BoardPos) {
        self.board.do_move(pos1, pos2);
        self.history.push(ActionRecord::Move(pos1, pos2));
    }

    fn change_energy_raw(&mut self, value: u8) {
        let old = self.board.num_energy;
        self.board.num_energy = value;
        self.history.push(ActionRecord::Energy(old));
    }

    fn move_intent(&mut self, pos1: BoardPos, mut pos2: BoardPos) -> bool {
        if pos1.depot_index == pos2.depot_index { return false; }
        let depot1 = &self.board.depots[pos1.depot_index];
        let depot2 = &self.board.depots[pos2.depot_index];

        let card = depot1[pos1.card_index];
        let Some(role) = DepotRole::role(pos2.depot_index) else { return false };
        let history_len = self.history.len();

        match role {
            DepotRole::Source => return false,
            DepotRole::Grid => {
                let power_count = [pos1, pos2].iter().filter(|&&pos| {
                    self.board.is_powered(pos.depot_index)
                }).count();
                if power_count == 0 { return false; }
                if power_count == 1 {
                    if self.board.num_energy < MAX_ENERGY { return false; }
                    self.change_energy_raw(0);
                }
                pos2.card_index = 0;
                self.do_move_raw(pos1, pos2);
            },
            DepotRole::Foundation => {
                let ok = if let Some(&c) = depot2.last() {
                    card.suit == c.suit && card.rank == c.rank + 1
                } else {
                    card.rank == 1
                };

                if !ok { return false; }
                // update to energy only done after animation is finalized
                self.do_move_raw(pos1, pos2);
            },
        }
        
        self.undo_stack.push(history_len);
        true
    }

    pub fn onclick(&mut self, pos: BoardPos) {
        if self.is_busy() { return; }

        if let Some(src) = self.board.selected {
            if pos == src { 
                self.board.selected = None; 
                return;
            }
            if src.depot_index == pos.depot_index && self.can_select(pos) {
                self.board.selected = Some(pos);
                return;
            }

            let dest = BoardPos { depot_index: pos.depot_index, card_index: pos.card_index.wrapping_add(1) };
            self.move_intent(src, dest);
        } else {
            if self.can_select(pos) {
                self.board.selected = Some(pos);
            }
        }
    }

    pub fn ondoubleclick(&mut self, pos: BoardPos) {
        if self.is_busy() { return; }
        if !self.can_select(pos) { return; } // needed, or illegal stacks can still be moved this way!

        for d in DepotRole::Foundation.range() {
            let dest = self.board.top_pos(d);
            if self.move_intent(pos, dest) {
                return;
            }
        }
    }

    pub fn advance_animations(&mut self, key: AnimationKey) {
        if key != self.animation_key { return; }
        self.animation_key = self.animation_key.wrapping_add(1);
        
        let energy = self.board.num_energy;
        if energy < MAX_ENERGY && self.board.is_card_moving_to_foundation() {
            self.change_energy_raw(energy + 1);
        }

        self.board.advance_actions();

        if self.is_won() {
            if !self.already_won {
                self.num_wins += 1;
                self.already_won = true;
            }
        } else {
            // self.check_auto_moves();
        }

        // if !self.is_busy() { LocalStorage.save_game_state(&self); }
    }
}