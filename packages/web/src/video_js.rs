//! Video player JavaScript bindings for web platform

use dioxus::prelude::*;
use dioxus_use_js::use_js;

// Import TypeScript functions using dioxus-use-js
// The macro needs both TS source file (for type parsing) and compiled JS file (for runtime)
// Build with: tsc assets/video_player.ts --outDir assets
use_js!("assets/video_player.ts", "assets/video_player.js"::{
    initVideoPlayerControls,
    updateFullscreenStyles,
    checkWindowFullscreen,
    setupFullscreenHover,
    isPlayerPageFullscreen,
});

/// Initialize video player controls (web platform)
pub async fn init_video_player_controls() -> Result<(), String> {
    initVideoPlayerControls().await.map_err(|e| e.to_string())
}
