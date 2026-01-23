//! Video player JavaScript bindings for web platform
//! This module only contains use_js! macro declarations.
//! The generated functions are used directly in components.

use dioxus::prelude::*;
use dioxus_use_js::use_js;

// Import state management functions
// Note: state_manager.js should be compiled from state_manager.ts
use_js!("assets/state_manager.ts", "assets/state_manager.js"::{
    loadAppState,
    saveAppState,
    setLastRoute,
    getSetting,
    getUrlParam,
    getUrlHash,
    setUrlParams,
});

// Import TypeScript functions using dioxus-use-js
// The macro needs both TS source file (for type parsing) and compiled JS file (for runtime)
use_js!("assets/video_player.ts", "assets/video_player.js"::{
    savePlayerState,
    initVideoPlayerControls,
    isPlayerPageFullscreen,
    setFullscreenMode,
    setupPlaybackTracking,
    scrollToEpisode,
    restorePlaybackEpisode,
    setAutoPlayNext,
});
