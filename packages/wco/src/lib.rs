//! WCO Scraper Library
//!
//! A Rust library for scraping anime series and episodes from WCO streaming sites.
//! This is a port of the original TypeScript library using `scraper` instead of `cheerio`.
//!
//! ## Features
//! - Search for anime series
//! - List episodes for a series
//! - Get video download links
//! - Download videos
//!
//! ## Example
//! ```no_run
//! use wco::{search_series, list_episodes, get_video_info};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Search for a series
//!     let series = search_series("naruto", None).await?;
//!     println!("Found {} series", series.len());
//!     
//!     if let Some(first_series) = series.first() {
//!         // List episodes
//!         let episodes = list_episodes(&first_series.url).await?;
//!         println!("Found {} episodes", episodes.len());
//!         
//!         if let Some(first_episode) = episodes.first() {
//!             // Get video info
//!             let video_info = get_video_info(&first_episode.url, None).await?;
//!             println!("Video URL: {}", video_info.url);
//!         }
//!     }
//!     
//!     Ok(())
//! }
//! ```

mod error;
mod user_agent;

pub mod detail;
pub mod list;
pub mod search;

// Re-export commonly used types and functions
pub use error::{Result, WcoError};
pub use user_agent::UserAgent;

pub use detail::{fetch_video, get_video_info, VideoInfo};
pub use list::{list_episodes, Episode};
pub use search::{search_series, Series};
