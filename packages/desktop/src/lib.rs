use axum::{Router, routing::get};
use tokio::sync::oneshot;
use tower_http::cors::{Any, CorsLayer};
use web::streaming_video;

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

/// Start the web server for video streaming.
/// Returns the port number once the server is ready.
pub fn start_video_server() -> u16 {
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
