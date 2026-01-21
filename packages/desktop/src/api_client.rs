//! HTTP API client for calling the local server

use serde::Deserialize;
use wco::{Episode, Series, VideoInfo, WcoError};

/// Search for series
pub async fn search_series(query: &str, port: u16) -> Result<Vec<Series>, WcoError> {
    let url = format!("http://127.0.0.1:{}/api/search", port);

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .json(&serde_json::json!({ "query": query }))
        .send()
        .await
        .map_err(|e| WcoError::Other(format!("Request failed: {}", e)))?;

    let status = response.status();
    if !status.is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(WcoError::Other(format!("HTTP {}: {}", status, error_text)));
    }

    #[derive(Deserialize)]
    struct Response {
        results: Vec<Series>,
    }
    let data: Response = response
        .json()
        .await
        .map_err(|e| WcoError::Other(format!("Failed to parse response: {}", e)))?;

    Ok(data.results)
}

/// List episodes for a series
pub async fn list_episodes(series_url: &str, port: u16) -> Result<Vec<Episode>, WcoError> {
    let url = format!("http://127.0.0.1:{}/api/episodes", port);

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .json(&serde_json::json!({ "series_url": series_url }))
        .send()
        .await
        .map_err(|e| WcoError::Other(format!("Request failed: {}", e)))?;

    let status = response.status();
    if !status.is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(WcoError::Other(format!("HTTP {}: {}", status, error_text)));
    }

    #[derive(Deserialize)]
    struct Response {
        episodes: Vec<Episode>,
    }

    let data: Response = response
        .json()
        .await
        .map_err(|e| WcoError::Other(format!("Failed to parse response: {}", e)))?;

    Ok(data.episodes)
}

/// Get video info for an episode
pub async fn get_video_info(episode_url: &str, port: u16) -> Result<VideoInfo, WcoError> {
    let url = format!("http://127.0.0.1:{}/api/video_info", port);

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .json(&serde_json::json!({ "episode_url": episode_url }))
        .send()
        .await
        .map_err(|e| WcoError::Other(format!("Request failed: {}", e)))?;

    let status = response.status();
    if !status.is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(WcoError::Other(format!("HTTP {}: {}", status, error_text)));
    }

    #[derive(Deserialize)]
    struct Response {
        video_info: VideoInfo,
    }
    let data: Response = response
        .json()
        .await
        .map_err(|e| WcoError::Other(format!("Failed to parse response: {}", e)))?;

    Ok(data.video_info)
}
