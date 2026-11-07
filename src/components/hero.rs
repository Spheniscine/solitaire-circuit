use dioxus::prelude::*;
use glam::Vec2;

use crate::{components::CardComponent, game::{Card, Skin}};

#[component]
pub fn Hero() -> Element {
    let test_card = Card { rank: 13, suit: crate::game::Suit::Spades };
    let is_powered = true;
    let skin = Skin::default();

    rsx! {
        div {
            id: "hero",
            class: "select-none",

            CardComponent {
                position: Vec2::new(10., 10.),
                width: 12.,
                card: test_card,
                skin,
                is_powered,
            }
        }
    }
}