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
    catch (error) {
        console.error("Error saving app state:", error);
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
        return JSON.parse(json);
    }
    catch (error) {
        console.error("Error loading app state:", error);
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
    catch (error) {
        console.error("Error clearing app state:", error);
    }
}
/**
 * Set the last route in saved state
 */
export function setLastRoute(route) {
    const state = loadAppState() || { route: "/" };
    state.route = route;
    saveAppState(state);
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
    catch (error) {
        console.error("Error loading settings:", error);
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
/**
 * Get a specific URL query parameter
 */
export function getUrlParam(key) {
    const urlParams = new URLSearchParams(globalThis.location.search);
    return urlParams.get(key);
}
/**
 * Update URL query parameters without reloading the page
 */
export function setUrlParams(params) {
    const url = new URL(globalThis.location.href);
    for (const [key, value] of Object.entries(params)) {
        if (value === null || value === "") {
            url.searchParams.delete(key);
        }
        else {
            url.searchParams.set(key, value);
        }
    }
    globalThis.history.replaceState({}, "", url.toString());
}
/**
 * Get URL hash (without the #)
 */
export function getUrlHash() {
    const hash = globalThis.location.hash;
    return hash ? hash.slice(1) : null;
}
/**
 * Update URL hash without reloading the page
 */
export function setUrlHash(hash) {
    const url = new URL(globalThis.location.href);
    if (hash === null || hash === "") {
        url.hash = "";
    }
    else {
        url.hash = hash;
    }
    globalThis.history.replaceState({}, "", url.toString());
}
