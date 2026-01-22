//! This crate contains all shared UI for the workspace.

mod hero;
pub use hero::Hero;

mod navbar;
pub use navbar::Navbar;

// Player components
mod search_bar;
pub use search_bar::SearchBar;

mod series_card;
pub use series_card::{SeriesCard, SeriesGrid};

mod episode_list;
pub use episode_list::{EpisodeItem, EpisodeList};

mod video_player;
pub use video_player::VideoPlayer;

mod components;
pub use components::select::*;

mod types;
pub use types::AppState;

mod api_trait;
pub use api_trait::{ApiClient, ApiClientContext};
