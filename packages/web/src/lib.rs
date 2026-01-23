//! Web package - shared views for web and desktop platforms

pub mod route;
#[cfg(feature = "desktop")]
pub mod streaming;
mod utils;
mod video_js;
mod views;

pub use utils::{format_time, parse_time};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ServerPort(pub u16);
