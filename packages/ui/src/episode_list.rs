//! Episode list component for the sidebar

use dioxus::prelude::*;
use wco::Episode;

/// Single episode item in the list
#[component]
pub fn EpisodeItem(
    /// Episode data
    episode: Episode,
    /// Whether this episode is currently selected
    #[props(default = false)]
    selected: bool,
    /// Callback when episode is clicked
    on_select: EventHandler<Episode>,
) -> Element {
    let episode_clone = episode.clone();

    rsx! {
        div {
            class: if selected { "episode-item selected" } else { "episode-item" },
            "data-episode-url": "{episode.url}",
            onclick: move |_| on_select.call(episode_clone.clone()),

            span {
                class: "episode-title",
                "{episode.title}"
            }
        }
    }
}

/// Episode list sidebar component
#[component]
pub fn EpisodeList(
    /// List of episodes
    episodes: Vec<Episode>,
    /// Currently selected episode URL (if any)
    selected_url: Option<String>,
    /// Callback when an episode is selected
    on_select: EventHandler<Episode>,
    /// Whether episodes are loading
    #[props(default = false)]
    loading: bool,
) -> Element {
    rsx! {
        div {
            class: "episode-list",

            div {
                class: "episode-list-header",
                h2 { "Episodes" }
                span {
                    class: "episode-count",
                    if loading {
                        "Loading..."
                    } else {
                        "{episodes.len()}"
                    }
                }
            }

            div {
                class: "episode-list-content",

                if loading {
                    div {
                        class: "episode-loading",
                        "Loading episodes..."
                    }
                } else if episodes.is_empty() {
                    div {
                        class: "episode-empty",
                        "No episodes found"
                    }
                } else {
                    for episode in episodes {
                        EpisodeItem {
                            key: "{episode.url}",
                            episode: episode.clone(),
                            selected: selected_url.as_ref() == Some(&episode.url),
                            on_select: move |e| on_select.call(e),
                        }
                    }
                }
            }
        }
    }
}
