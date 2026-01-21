//! This crate contains all shared UI for the workspace.

#[cfg(not(feature = "desktop"))]
mod hero;
#[cfg(not(feature = "desktop"))]
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
