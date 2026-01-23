//! Search page - entry point for finding series

use crate::views::PlayerQuery;
use dioxus::prelude::*;
use ui::{AppState, SearchBar, SeriesGrid};
use wco::Series;

/// Search page component
#[component]
pub fn Search() -> Element {
    let search_results = use_signal(Vec::<Series>::new);
    let is_searching = use_signal(|| false);
    let search_error = use_signal(|| Option::<String>::None);
    let has_searched = use_signal(|| false);

    // Get router
    let router = router();

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
                            let series_url = s.url.clone();
                            // Save route to localStorage
                            spawn(async move {
                                use crate::video_js::{setLastRoute, saveAppState};

                                let _: Result<(), _> = setLastRoute("/player").await;

                                // Save series to localStorage
                                let state_result = crate::video_js::loadAppState::<Option<AppState>>().await;
                                let mut state = if let Ok(Some(s)) = state_result {
                                    s
                                } else {
                                    AppState {
                                        route: "/player".to_string(),
                                        series: None,
                                        episode: None,
                                        playback_position: None,
                                    }
                                };
                                state.series = Some(Series {
                                    title: s.title.clone(),
                                    url: s.url.clone(),
                                    thumbnail: s.thumbnail.clone(),
                                });
                                state.route = "/player".to_string();
                                let _: Result<(), _> = saveAppState(state).await;
                            });

                            // Navigate to player route with query parameter
                            let query = PlayerQuery {
                                series_url,
                                episode_url: None,
                            };
                            router
                                .push(crate::route::Route::Player {
                                    query,
                                });
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
