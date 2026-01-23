use dioxus::prelude::*;

use crate::views::{Home, Player, PlayerQuery, Search};

/// Route definitions (shared with desktop)
#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[route("/")]
    Home {},
    #[route("/search")]
    Search {},
    #[route("/player?:..query")]
    Player { query: PlayerQuery },
}
