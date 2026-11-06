use dioxus::prelude::*;

#[component]
pub fn Hero() -> Element {
    rsx! {
        div {
            id: "hero",
            class: "select-none",
        }
    }
}