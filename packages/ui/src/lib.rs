//! This crate contains all shared UI for the workspace.

// Player components
mod search_bar;
pub use search_bar::SearchBar;

mod series_card;
pub use series_card::{SeriesCard, SeriesGrid};

mod episode_list;
pub use episode_list::{EpisodeItem, EpisodeList};

mod series_description;
pub use series_description::SeriesDescription;

mod video_player;
pub use video_player::VideoPlayer;

mod components;
pub use components::select::*;

mod types;
pub use types::AppState;
