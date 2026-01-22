use dioxus::prelude::*;
use ui::AppState;
use wco::Series;

// Home route component (default route, handles route restoration)
#[component]
pub fn Home() -> Element {
    // Route restoration on app startup
    let router = router();
    let mut current_series = use_context::<Signal<Option<Series>>>();
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
                            // Restore series if available
                            if let Some(series_data) = state.series {
                                // Reconstruct Series from saved data
                                let series = Series {
                                    title: series_data.title.clone(),
                                    url: series_data.url.clone(),
                                    thumbnail: series_data.thumbnail.clone(),
                                };
                                current_series.set(Some(series));
                                router.replace(crate::route::Route::Player {});
                            } else {
                                // No series data, fallback to Search route
                                router.replace(crate::route::Route::Search {});
                            }
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
