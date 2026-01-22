use axum::{
    body::Body,
    extract::Query,
    http::{HeaderMap, StatusCode, header},
    response::IntoResponse,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct VideoQuery {
    url: String,
}

pub async fn streaming_video(
    Query(query): Query<VideoQuery>,
    headers: HeaderMap,
) -> impl IntoResponse {
    // Prepare extra headers (especially Range for seeking)
    let mut extra_headers = reqwest::header::HeaderMap::new();
    if let Some(range) = headers.get(header::RANGE) {
        extra_headers.insert(header::RANGE, range.clone());
    }

    // Fetch video
    match wco::fetch_video(&query.url, Some(&extra_headers)).await {
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
