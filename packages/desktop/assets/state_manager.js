// Application state management using localStorage
const STATE_KEY = "wco-player-state";
/**
 * Save application state to localStorage
 */
export function saveAppState(state) {
    try {
        const json = JSON.stringify(state);
        localStorage.setItem(STATE_KEY, json);
    }
    catch {
        // Silently fail
    }
}
/**
 * Load application state from localStorage
 */
export function loadAppState() {
    try {
        const json = localStorage.getItem(STATE_KEY);
        if (!json) {
            return null;
        }
        const state = JSON.parse(json);
        return state;
    }
    catch {
        return null;
    }
}
/**
 * Clear application state from localStorage
 */
export function clearAppState() {
    try {
        localStorage.removeItem(STATE_KEY);
    }
    catch {
        // Silently fail
    }
}
/**
 * Set the playback position in saved state
 */
export function setPlaybackPosition(position) {
    // Always ensure we have a state object, even if it doesn't exist yet
    const state = loadAppState() || { route: location.pathname || "/" };
    if (position !== null && position > 0) {
        state.playback_position = position;
    }
    else {
        delete state.playback_position;
    }
    saveAppState(state);
}
/**
 * Set the last route in saved state
 */
export function setLastRoute(route) {
    const state = loadAppState() || { route: "/" };
    state.route = route;
    saveAppState(state);
}
/**
 * Set series and episode in saved state
 * This also clears playback position when series/episode changes
 */
export function setSeriesAndEpisode(series, episode) {
    const state = loadAppState() || { route: "/player" };
    const oldEpisodeUrl = state.episode?.url;
    // Clear playback position if episode changed
    if (oldEpisodeUrl && episode?.url && oldEpisodeUrl !== episode.url) {
        delete state.playback_position;
    }
    else if (!oldEpisodeUrl && episode?.url) {
        // New episode selected, clear any existing position
        delete state.playback_position;
    }
    state.series = series;
    state.episode = episode;
    saveAppState(state);
}
/**
 * Restore route from localStorage on app startup
 * This function is no longer used - route restoration is handled in Rust
 * @deprecated Route restoration is now handled in Rust using router().push()
 */
export function restoreRouteFromState() {
    // This function is kept for backwards compatibility but does nothing
    // Route restoration is now handled in Rust in the Search component
}
const SETTINGS_KEY = "wco-player-settings";
/**
 * Load settings from localStorage
 */
export function loadSettings() {
    try {
        const json = localStorage.getItem(SETTINGS_KEY);
        if (!json) {
            return {};
        }
        return JSON.parse(json);
    }
    catch {
        return {};
    }
}
/**
 * Save settings to localStorage
 */
export function saveSettings(settings) {
    try {
        const json = JSON.stringify(settings);
        localStorage.setItem(SETTINGS_KEY, json);
    }
    catch {
        // Silently fail
    }
}
/**
 * Set a specific setting value
 */
export function setSetting(key, value) {
    const settings = loadSettings();
    settings[key] = value;
    saveSettings(settings);
}
/**
 * Get a specific setting value
 */
export function getSetting(key, defaultValue) {
    const settings = loadSettings();
    return settings[key] ?? defaultValue;
}
