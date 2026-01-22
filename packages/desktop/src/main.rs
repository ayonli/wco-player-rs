use axum::{Router, routing::get};
use dioxus::prelude::*;
use tokio::sync::oneshot;
use tower_http::cors::{Any, CorsLayer};
use wco::Series;
use web::{ServerPort, route::Route, streaming::streaming_video};

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
    let server_port = start_server_sync();

    dioxus::LaunchBuilder::new()
        .with_cfg(Config::new().with_window(window))
        .with_context(ServerPort(server_port))
        .launch(App);
}

/// Build the API router with all routes
fn build_router() -> Router {
    Router::new()
        .route("/streaming", get(streaming_video))
        // Add CORS
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
}

/// Start the combined server (video proxy + API) synchronously
/// Returns the port number once the server is ready
fn start_server_sync() -> u16 {
    let (tx, rx) = oneshot::channel();

    std::thread::spawn(move || {
        let rt = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(e) => {
                eprintln!("❌ Failed to create tokio runtime: {}", e);
                let _ = tx.send(0);
                return;
            }
        };

        rt.block_on(async move {
            let app = build_router();

            let listener = match tokio::net::TcpListener::bind("127.0.0.1:0").await {
                Ok(listener) => listener,
                Err(e) => {
                    eprintln!("❌ Failed to bind TCP listener: {}", e);
                    let _ = tx.send(0);
                    return;
                }
            };

            let addr = match listener.local_addr() {
                Ok(addr) => addr,
                Err(e) => {
                    eprintln!("❌ Failed to get local address: {}", e);
                    let _ = tx.send(0);
                    return;
                }
            };

            let port = addr.port();

            // Send port back to main thread
            if tx.send(port).is_err() {
                eprintln!("❌ Failed to send port to main thread");
                return;
            }

            eprintln!("🚀 API server started at http://127.0.0.1:{}", port);

            if let Err(e) = axum::serve(listener, app).await {
                eprintln!("❌ Server error: {}", e);
            }
        });
    });

    // Wait for server to be ready
    match rx.blocking_recv() {
        Ok(port) => {
            if port == 0 {
                eprintln!("❌ Server failed to start");
                std::process::exit(1);
            }
            eprintln!("✅ API server ready on port {}", port);
            port
        }
        Err(e) => {
            eprintln!("❌ Server startup channel error: {}", e);
            std::process::exit(1);
        }
    }
}

#[component]
fn App() -> Element {
    // Current series for player page (stored in context)
    let current_series = use_signal(|| Option::<Series>::None);
    use_context_provider(|| current_series);

    rsx! {
        // Load styles (theme first for CSS variables, then player.css to override body styles)
        document::Link { rel: "stylesheet", href: DX_COMPONENTS_THEME }
        document::Link { rel: "stylesheet", href: PLAYER_CSS }

        Router::<Route> {}
    }
}
