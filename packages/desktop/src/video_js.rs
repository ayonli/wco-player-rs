//! Video player JavaScript bindings for desktop platform

use dioxus::prelude::*;
use dioxus_use_js::use_js;

// Import TypeScript functions using dioxus-use-js
// The macro needs both TS source file (for type parsing) and compiled JS file (for runtime)
// TS source is shared from web package, JS is compiled to desktop assets
use_js!("../web/assets/video_player.ts", "assets/video_player.js"::{
    initVideoPlayerControls,
    updateFullscreenStyles,
    checkWindowFullscreen,
    setupFullscreenHover,
    isPlayerPageFullscreen,
});

/// Initialize video player controls (desktop platform)
pub async fn init_video_player_controls() -> Result<(), String> {
    initVideoPlayerControls().await.map_err(|e| e.to_string())
}
