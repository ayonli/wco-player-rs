use dioxus::prelude::*;

use crate::views::{Home, Player, Search};

/// Route definitions (shared with desktop)
#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[route("/")]
    Home {},
    #[route("/search")]
    Search {},
    #[route("/player")]
    Player {},
}
