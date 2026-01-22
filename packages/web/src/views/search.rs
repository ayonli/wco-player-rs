//! Search page - entry point for finding series

use dioxus::prelude::*;
use ui::{SearchBar, SeriesGrid};
use wco::Series;

/// Search page component
#[component]
pub fn Search() -> Element {
    let search_results = use_signal(Vec::<Series>::new);
    let is_searching = use_signal(|| false);
    let search_error = use_signal(|| Option::<String>::None);
    let has_searched = use_signal(|| false);

    // Get router and current series context
    let router = router();
    let mut current_series = use_context::<Signal<Option<Series>>>();

    let handle_search = {
        let mut is_searching_clone = is_searching;
        let mut search_error_clone = search_error;
        let mut search_results_clone = search_results;
        let mut has_searched_clone = has_searched;
        move |query: String| {
            spawn(async move {
                is_searching_clone.set(true);
                search_error_clone.set(None);

                let result = async {
                    #[cfg(feature = "desktop")]
                    {
                        wco::search_series(&query, None).await
                    }
                    #[cfg(not(feature = "desktop"))]
                    {
                        match api::search_series(query).await {
                            Ok(results) => Ok(results),
                            Err(e) => Err(wco::WcoError::Other(e.to_string())),
                        }
                    }
                }
                .await;

                match result {
                    Ok(results) => {
                        search_results_clone.set(results);
                        has_searched_clone.set(true);
                    }
                    Err(e) => {
                        search_error_clone.set(Some(e.to_string()));
                        search_results_clone.set(vec![]);
                    }
                }

                is_searching_clone.set(false);
            });
        }
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
                                use crate::video_js::setLastRoute;
                                let _: Result<(), _> = setLastRoute("/player").await;
                            });

                            // Navigate to player route
                            // Route is defined in web's main.rs, accessed via web::Route
                            router.push(crate::route::Route::Player {});
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
