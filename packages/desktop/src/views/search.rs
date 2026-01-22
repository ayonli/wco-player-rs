//! Search page - entry point for finding series

use crate::{api_client, ServerConfig};
use dioxus::prelude::*;
use ui::{SearchBar, SeriesGrid};
use wco::Series;

/// Search page component
#[component]
pub fn Search() -> Element {
    let mut search_results = use_signal(Vec::<Series>::new);
    let mut is_searching = use_signal(|| false);
    let mut search_error = use_signal(|| Option::<String>::None);
    let mut has_searched = use_signal(|| false);

    // Get server config from context
    let server_config = use_context::<ServerConfig>();

    // Get router and current series context
    let router = router();
    let mut current_series = use_context::<Signal<Option<Series>>>();

    let handle_search = move |query: String| {
        spawn(async move {
            is_searching.set(true);
            search_error.set(None);

            match api_client::search_series(&query, server_config.port).await {
                Ok(results) => {
                    search_results.set(results);
                    has_searched.set(true);
                }
                Err(e) => {
                    search_error.set(Some(e.to_string()));
                    search_results.set(vec![]);
                }
            }

            is_searching.set(false);
        });
    };

    rsx! {
        div { class: "search-page",
            // Header with search bar
            div { class: "search-header",
                h1 { "🎬 WCO Player" }
                SearchBar { on_search: handle_search, loading: is_searching() }
            }
            // Results area
            div { class: "search-results",
                if is_searching() {
                    div { class: "loading-overlay",
                        div { class: "spinner large", "" }
                    }
                }
                if let Some(ref error) = search_error() {
                    div { class: "no-results",
                        p { "❌ Error: {error}" }
                    }
                } else if has_searched() && search_results.read().is_empty() {
                    div { class: "no-results",
                        p { "No results found. Try a different search term." }
                    }
                } else if !search_results.read().is_empty() {
                    div { class: "search-results-header",
                        h2 { "Found {search_results.read().len()} series" }
                    }
                    SeriesGrid {
                        series_list: search_results(),
                        on_select: move |s: Series| {
                            // Set series in context and navigate to player
                            current_series.set(Some(s.clone()));

                            // Save route to localStorage
                            spawn(async move {
                                use crate::video_js::updateRoute;
                                let _: Result<(), _> = updateRoute("/player".to_string()).await;
                            });

                            router.push(crate::Route::Player {});
                        },
                    }
                } else {
                    div { class: "no-results",
                        p { "Search for your favorite anime series to get started!" }
                    }
                }
            }
        }
    }
}
