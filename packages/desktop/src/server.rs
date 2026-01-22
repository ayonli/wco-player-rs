//! Combined HTTP server for video proxy and API endpoints

use axum::{
    body::Body,
    extract::{Json, Query},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;
use tower_http::cors::{Any, CorsLayer};
use wco::{fetch_video, get_video_info, list_episodes, search_series};

/// Start the combined server (video proxy + API) synchronously
/// Returns the port number once the server is ready
pub fn start_server_sync() -> u16 {
    let (tx, rx) = oneshot::channel();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async move {
            // Build router with all routes
            let app = Router::new()
                // Health check
                .route("/", get(health_check))
                // API endpoints
                .route("/api/search", post(api_search))
                .route("/api/episodes", post(api_episodes))
                .route("/api/video_info", post(api_video_info))
                // Video proxy
                .route("/video", get(proxy_video))
                // Add CORS
                .layer(
                    CorsLayer::new()
                        .allow_origin(Any)
                        .allow_methods(Any)
                        .allow_headers(Any),
                );

            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let addr = listener.local_addr().unwrap();
            let port = addr.port();

            // Send port back to main thread
            tx.send(port).unwrap();

            let _ = axum::serve(listener, app).await;
        });
    });

    // Wait for server to be ready
    rx.blocking_recv().unwrap()
}

// ============================================================================
// Health Check
// ============================================================================

async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "message": "Server is running"
    }))
}

// ============================================================================
// API Endpoints
// ============================================================================

#[derive(Deserialize)]
struct SearchRequest {
    query: String,
}

#[derive(Serialize)]
struct SearchResponse {
    results: Vec<wco::Series>,
}

async fn api_search(Json(req): Json<SearchRequest>) -> impl IntoResponse {
    match search_series(&req.query, None).await {
        Ok(results) => (StatusCode::OK, Json(SearchResponse { results })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": e.to_string()
            })),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
struct EpisodesRequest {
    series_url: String,
}

#[derive(Serialize)]
struct EpisodesResponse {
    episodes: Vec<wco::Episode>,
}

async fn api_episodes(Json(req): Json<EpisodesRequest>) -> impl IntoResponse {
    match list_episodes(&req.series_url).await {
        Ok(episodes) => (StatusCode::OK, Json(EpisodesResponse { episodes })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": e.to_string()
            })),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
struct VideoInfoRequest {
    episode_url: String,
}

#[derive(Serialize)]
struct VideoInfoResponse {
    video_info: wco::VideoInfo,
}

async fn api_video_info(Json(req): Json<VideoInfoRequest>) -> impl IntoResponse {
    match get_video_info(&req.episode_url, None).await {
        Ok(video_info) => (StatusCode::OK, Json(VideoInfoResponse { video_info })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": e.to_string()
            })),
        )
            .into_response(),
    }
}

// ============================================================================
// Video Proxy
// ============================================================================

#[derive(Deserialize)]
struct VideoQuery {
    url: String,
}

async fn proxy_video(Query(query): Query<VideoQuery>, headers: HeaderMap) -> impl IntoResponse {
    // Prepare extra headers (especially Range for seeking)
    let mut extra_headers = reqwest::header::HeaderMap::new();
    if let Some(range) = headers.get(header::RANGE) {
        extra_headers.insert(header::RANGE, range.clone());
    }

    // Fetch video
    match fetch_video(&query.url, Some(&extra_headers)).await {
        Ok(response) => {
            let status = response.status();
            if !status.is_success() && status != reqwest::StatusCode::PARTIAL_CONTENT {
                return (StatusCode::BAD_GATEWAY, "Failed to fetch video").into_response();
            }

            // Build response headers
            let mut response_headers = HeaderMap::new();
            if let Some(ct) = response.headers().get(header::CONTENT_TYPE) {
                response_headers.insert(header::CONTENT_TYPE, ct.clone());
            }
            if let Some(cl) = response.headers().get(header::CONTENT_LENGTH) {
                response_headers.insert(header::CONTENT_LENGTH, cl.clone());
            }
            if let Some(cr) = response.headers().get(header::CONTENT_RANGE) {
                response_headers.insert(header::CONTENT_RANGE, cr.clone());
            }
            response_headers.insert(header::ACCEPT_RANGES, "bytes".parse().unwrap());

            let stream = response.bytes_stream();
            let body = Body::from_stream(stream);

            (
                StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::OK),
                response_headers,
                body,
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            format!("Failed to fetch video: {}", e),
        )
            .into_response(),
    }
}
