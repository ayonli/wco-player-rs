//! Shared types for application state

use serde::{Deserialize, Serialize};
use wco::{Episode, Series};

/// Application state loaded from localStorage (matches TypeScript AppState)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub route: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series: Option<Series>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub episode: Option<Episode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub playback_position: Option<f64>,
}
