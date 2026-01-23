//! Web package - shared views for web and desktop platforms

mod route;
mod utils;
mod video_js;
mod views;
pub use views::App;

#[cfg(feature = "desktop")]
mod streaming;
#[cfg(feature = "desktop")]
pub use streaming::streaming_video;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ServerPort(pub u16);
