use dioxus::prelude::*;
use ui::AppState;

use crate::views::PlayerQuery;

// Home route component (default route, handles route restoration)
#[component]
pub fn Home() -> Element {
    // Route restoration on app startup
    let router = router();
    let mut initialized = use_signal(|| false);

    use_effect(move || {
        if !initialized() {
            initialized.set(true);
            spawn(async move {
                // Load state from localStorage
                let result = crate::video_js::loadAppState::<Option<AppState>>().await;
                if let Ok(Some(state)) = result {
                    // Match the saved route path and construct the corresponding Route enum
                    match state.route.as_str() {
                        "/player" => {
                            // Build PlayerQuery from saved state
                            let query = PlayerQuery {
                                series_url: state
                                    .series
                                    .as_ref()
                                    .map(|s| s.url.clone())
                                    .unwrap_or_default(),
                                episode_url: state.episode.as_ref().map(|e| e.url.clone()),
                            };
                            router.replace(crate::route::Route::Player { query });
                        }
                        "/search" => {
                            // Saved route is Search, navigate to it
                            router.replace(crate::route::Route::Search {});
                        }
                        "/" => {
                            // Saved route is Home (default), go to Search
                            router.replace(crate::route::Route::Search {});
                        }
                        _ => {
                            // Invalid route path, fallback to Search route
                            router.replace(crate::route::Route::Search {});
                        }
                    }
                } else {
                    // No saved state, go to Search route
                    router.replace(crate::route::Route::Search {});
                }
            });
        }
    });

    // Return empty content while determining route
    rsx! {}
}
