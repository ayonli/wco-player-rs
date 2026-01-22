//! Trait for abstracting API client calls across platforms

use async_trait::async_trait;
use std::sync::Arc;
use wco::{Episode, Series, VideoInfo, WcoError};

/// Trait for platform-specific API client implementations
#[async_trait]
pub trait ApiClient: Send + Sync {
    /// Search for series
    async fn search_series(&self, keyword: &str) -> Result<Vec<Series>, WcoError>;

    /// List episodes for a series
    async fn list_episodes(&self, series_url: &str) -> Result<Vec<Episode>, WcoError>;

    /// Get video info for an episode
    async fn get_video_info(&self, episode_url: &str) -> Result<VideoInfo, WcoError>;

    /// Build a proxy URL for a video
    fn build_video_url(&self, video_url: &str) -> String;
}

/// Wrapper type for ApiClient that can be used in Dioxus context
#[derive(Clone)]
pub struct ApiClientContext {
    inner: Arc<dyn ApiClient>,
}

impl ApiClientContext {
    pub fn new(client: impl ApiClient + 'static) -> Self {
        Self {
            inner: Arc::new(client),
        }
    }

    pub async fn search_series(&self, query: &str) -> Result<Vec<Series>, WcoError> {
        self.inner.search_series(query).await
    }

    pub async fn list_episodes(&self, series_url: &str) -> Result<Vec<Episode>, WcoError> {
        self.inner.list_episodes(series_url).await
    }

    pub async fn get_video_info(&self, episode_url: &str) -> Result<VideoInfo, WcoError> {
        self.inner.get_video_info(episode_url).await
    }

    pub fn build_video_url(&self, video_url: &str) -> String {
        self.inner.build_video_url(video_url)
    }
}

impl PartialEq for ApiClientContext {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}
