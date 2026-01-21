use dioxus::prelude::*;
use views::{Player, Search};
use wco::Series;

mod api_client;
mod server;
mod video_js;
mod views;

const PLAYER_CSS: Asset = asset!("/assets/player.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind-output.css");
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
    eprintln!("🚀 Starting combined server...");
    let server_port = server::start_server_sync();
    eprintln!("✅ Server ready on port {}", server_port);

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

/// Application state
#[derive(Clone, PartialEq)]
enum AppView {
    /// Search page
    Search,
    /// Player page with selected series
    Player(Series),
}

#[component]
fn App() -> Element {
    let mut current_view = use_signal(|| AppView::Search);

    rsx! {
        // Load styles (theme first for CSS variables, then player.css to override body styles)
        document::Link { rel: "stylesheet", href: DX_COMPONENTS_THEME }
        document::Stylesheet { href: TAILWIND_CSS }
        document::Link { rel: "stylesheet", href: PLAYER_CSS }

        // Render current view
        match current_view() {
            AppView::Search => rsx! {
                Search {
                    on_select: move |series: Series| {
                        current_view.set(AppView::Player(series));
                    },
                }
            },
            AppView::Player(series) => rsx! {
                Player {
                    series,
                    on_back: move |_| {
                        current_view.set(AppView::Search);
                    },
                }
            },
        }
    }
}
