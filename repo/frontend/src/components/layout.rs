use dioxus::prelude::*;
use super::nav::NavBar;

#[component]
pub fn Layout(children: Element) -> Element {
    rsx! {
        div { class: "app-layout",
            NavBar {}
            main { class: "main-content",
                {children}
            }
            footer { class: "app-footer",
                p { "Meridian Academic Publishing & Fulfillment Portal" }
            }
        }
    }
}
