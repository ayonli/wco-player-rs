//! Web package - shared views for web and desktop platforms

mod route;
mod utils;
mod video_js;
mod views;
pub use views::App;

#[cfg(any(feature = "desktop", feature = "mobile"))]
mod streaming;

#[cfg(any(feature = "desktop", feature = "mobile"))]
mod server;
#[cfg(any(feature = "desktop", feature = "mobile"))]
pub use server::start_video_server;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ServerPort(pub u16);
