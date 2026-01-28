//! Search bar component for searching series

use dioxus::prelude::*;
use dioxus_free_icons::{Icon, icons::ld_icons::LdSearch};

/// Search bar component
#[component]
pub fn SearchBar(
    /// Callback when search is submitted
    on_search: EventHandler<String>,
    /// Placeholder text
    #[props(default = "Search for anime series...".to_string())]
    placeholder: String,
    /// Whether search is in progress
    #[props(default = false)]
    loading: bool,
) -> Element {
    let mut query = use_signal(String::new);

    let handle_submit = move |e: Event<FormData>| {
        e.prevent_default();
        let q = query.read().trim().to_string();
        if !q.is_empty() {
            on_search.call(q);
        }
    };

    rsx! {
        form { class: "search-bar", onsubmit: handle_submit,
            input {
                r#type: "text",
                class: "search-input",
                placeholder: "{placeholder}",
                value: "{query}",
                disabled: loading,
                oninput: move |e| query.set(e.value()),
            }
            button {
                r#type: "submit",
                class: "search-button",
                disabled: loading || query.read().trim().is_empty(),
                title: "Search",
                if loading {
                    span { class: "spinner", "" }
                } else {
                    Icon {
                        icon: LdSearch,
                        width: Some(20),
                        height: Some(20),
                    }
                }
            }
        }
    }
}
