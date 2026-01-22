use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use views::Search;
use wco::Series;

mod api_client;
mod server;
mod video_js;
mod views;

const PLAYER_CSS: Asset = asset!("/assets/player.css");
const DX_COMPONENTS_THEME: Asset = asset!("/assets/dx-components-theme.css");

fn main() {
    use dioxus::desktop::tao::dpi::LogicalSize;
    use dioxus::desktop::{Config, WindowBuilder};

    let window = WindowBuilder::new()
        .with_title("WCO Player")
        .with_always_on_top(false)
        .with_resizable(true)
        .with_inner_size(LogicalSize::new(1280.0, 800.0));

    // Start combined server (video proxy + API)
    let server_port = server::start_server_sync();

    dioxus::LaunchBuilder::new()
        .with_cfg(Config::new().with_window(window))
        .with_context(ServerConfig { port: server_port })
        .launch(App);
}

/// Server configuration shared via context
#[derive(Clone, Copy)]
pub struct ServerConfig {
    pub port: u16,
}

/// Current series context (for player page)
#[derive(Clone, PartialEq)]
pub struct CurrentSeries(pub Series);

/// Application state loaded from localStorage (matches TypeScript AppState)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub route: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series: Option<SeriesData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub episode: Option<EpisodeData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub playback_position: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesData {
    pub title: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeData {
    pub title: String,
    pub url: String,
}

/// Route definitions
#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[route("/")]
    Home {},
    #[route("/search")]
    Search {},
    #[route("/player")]
    Player {},
}

#[component]
fn App() -> Element {
    // Current series for player page (stored in context)
    let current_series = use_signal(|| Option::<Series>::None);

    // Provide current series context
    use_context_provider(|| current_series);

    rsx! {
        // Load styles (theme first for CSS variables, then player.css to override body styles)
        document::Link { rel: "stylesheet", href: DX_COMPONENTS_THEME }
        document::Link { rel: "stylesheet", href: PLAYER_CSS }

        Router::<Route> {}
    }
}

// Home route component (default route, handles route restoration)
#[component]
fn Home() -> Element {
    // Route restoration on app startup
    let router = router();
    let mut current_series = use_context::<Signal<Option<Series>>>();
    let mut initialized = use_signal(|| false);

    use_effect(move || {
        if !initialized() {
            initialized.set(true);
            spawn(async move {
                // Wait a bit for everything to initialize
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

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
                                router.replace(Route::Player {});
                            } else {
                                // No series data, fallback to Search route
                                router.replace(Route::Search {});
                            }
                        }
                        "/search" => {
                            // Saved route is Search, navigate to it
                            router.replace(Route::Search {});
                        }
                        "/" => {
                            // Saved route is Home (default), go to Search
                            router.replace(Route::Search {});
                        }
                        _ => {
                            // Invalid route path, fallback to Search route
                            router.replace(Route::Search {});
                        }
                    }
                } else {
                    // No saved state, go to Search route
                    router.replace(Route::Search {});
                }
            });
        }
    });

    // Return empty content while determining route
    rsx! {}
}

// Player route component - needs to get series from context
#[component]
fn Player() -> Element {
    let current_series = use_context::<Signal<Option<Series>>>();

    match current_series() {
        Some(series) => rsx! {
            views::Player { series }
        },
        None => rsx! {
            div {
                class: "error-page",
                style: "display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100vh;",
                h1 { "No series selected" }
                Link { to: Route::Search {}, "Go to Search" }
            }
        },
    }
}
