use dioxus::prelude::*;

use crate::{components::{Emoji, SkinTrait, rem}, game::{Card, DECK_SIZE, GameState, NUM_RANKS, NUM_SUITS, RANK_ACE, RANK_KING, RANK_QUEEN, RankSkin, ScreenState, Suit,}};

#[component]
fn Emph(children: Element) -> Element {
    rsx! {
        strong {
            color: "#ff0",
            {children}
        }
    }
}

#[component]
pub fn Help(mut game_state: Signal<GameState>) -> Element {
    let st = game_state.read();
    let skin = st.skin;

    let render_rank = |rank: u8| rsx! {
        span {
            font_size: "1.2em",
            line_height: 1.2,
            {skin.render_rank(&Card { rank, suit: Suit::Spades })}
        }
    };

    let ace_node = match skin.ranks {
        RankSkin::Numbers => render_rank(RANK_ACE),
        RankSkin::Traditional => rsx!{"Ace"},
    };

    let queen_node = match skin.ranks {
        RankSkin::Numbers => render_rank(RANK_QUEEN),
        RankSkin::Traditional => rsx!{"Queen"},
    };

    let kings_desc = match skin.ranks {
        RankSkin::Numbers => rsx!{"Cards of rank " {render_rank(RANK_KING)}},
        RankSkin::Traditional => rsx!{"Kings"},
    };

    rsx! {
        div {
            style: "display: flex; flex-direction: column; align-items: center; font-size: 3.9rem; color: #fff; padding: 4rem;",
            class: "help",

            div {
                text_align: "left",

                p {
                    margin_top: "0",
                    "The deck is a standard {DECK_SIZE}-card deck, with {NUM_RANKS} ranks and {NUM_SUITS} suits."
                }

                p {
                    "To " Emph{"win the game"} ", stack cards of each suit, incrementing from " {ace_node}
                    " to " {queen_node} ", to the " Emph{"foundations"} "."
                }

                p {
                    {kings_desc} " are " Emph{"power sources"} " and " Emph{"cannot"} " be moved. A card whose " Emph{"wires"}
                    " connect to a power source or another powered card becomes " Emph{"powered"} "."
                }

                p {
                    "You start with 3 " Emph{"energy"} ", indicated by the colored " Emoji { text: "⚡" }
                    " symbols on the top-right."
                }

                p {
                    "The following moves " Emph{"cost 3 energy"} ":"

                    ul {
                        li {
                            "Swapping a " Emph{"powered"} " card with an " Emph{"unpowered"} " card in the grid."
                        }
                        li {
                            "Moving a " Emph{"powered"} " card to an " Emph{"empty"} " spot in the grid."
                        }
                    }
                }

                p {
                    "The following moves are " Emph{"free"} ":"

                    ul {
                        li {
                            "Swapping two " Emph{"powered"} " cards in the grid."
                        }
                        li {
                            "Moving a " Emph{"powered"} " card to the " Emph{"foundations"} ". If you have less than 3 energy, you "
                            Emph{"regain"} " 1 energy."
                        }
                    }
                }

                p {
                    Emph{"Shortcut note: "} "Double-clicking a card will send it to the foundations if possible."
                }

                p {
                    "(The deal guarantees that 3 or more power sources each connect to a card.)"
                }

                div {
                    position: "absolute",
                    bottom: rem(2.),
                    width: "92rem",
                    display: "flex",
                    justify_content: "center",

                    a {
                        // href: VIDEO_GAMEPLAY,
                        // target: "_blank",
                        text_decoration: "none",
                        margin_right: rem(4.),
                        div {
                            width: rem(30.),
                            position: "relative",
                            class: "game-button-disabled",
                            "Example video"
                        }
                    }

                    div {
                        width: rem(30.),
                        position: "relative",
                        class: "game-button",
                        onclick: move |_| game_state.write().screen_state = ScreenState::Game,
                        "Back to game"
                    }
                }
            }
        }
    }
}