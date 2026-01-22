//! Video player JavaScript bindings for desktop platform
//! This module only contains use_js! macro declarations.
//! The generated functions are used directly in components.

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
    setupPlaybackTracking,
    getCurrentPlaybackPosition,
    scrollToEpisode,
    restorePlaybackEpisode,
    setAutoPlayNext,
    playNextEpisode,
});

// Import state management functions
use_js!("assets/state_manager.js"::{
    loadAppState,
    updateRoute,
    updateSeriesAndEpisode,
    loadSettings,
    saveSettings,
    updateSetting,
    getSetting,
});
