// Application state management using localStorage

export interface AppState {
    route: string
    series?: {
        title: string
        url: string
        thumbnail?: string
    }
    episode?: {
        title: string
        url: string
    }
    playback_position?: number
}

const STATE_KEY = "wco-player-state"

/**
 * Save application state to localStorage
 */
export function saveAppState(state: AppState): void {
    try {
        const json = JSON.stringify(state)
        localStorage.setItem(STATE_KEY, json)
    } catch {
        // Silently fail
    }
}

/**
 * Load application state from localStorage
 */
export function loadAppState(): AppState | null {
    try {
        const json = localStorage.getItem(STATE_KEY)
        if (!json) {
            return null
        }
        const state = JSON.parse(json) as AppState
        return state
    } catch {
        return null
    }
}

/**
 * Clear application state from localStorage
 */
export function clearAppState(): void {
    try {
        localStorage.removeItem(STATE_KEY)
    } catch {
        // Silently fail
    }
}

/**
 * Set the playback position in saved state
 */
export function setPlaybackPosition(position: number | null): void {
    // Always ensure we have a state object, even if it doesn't exist yet
    const state = loadAppState() || { route: location.pathname || "/" }
    if (position !== null && position > 0) {
        state.playback_position = position
    } else {
        delete state.playback_position
    }
    saveAppState(state)
}

/**
 * Set the last route in saved state
 */
export function setLastRoute(route: string): void {
    const state = loadAppState() || { route: "/" }
    state.route = route
    saveAppState(state)
}

/**
 * Set series and episode in saved state
 * This also clears playback position when series/episode changes
 */
export function setSeriesAndEpisode(
    series: AppState["series"],
    episode: AppState["episode"],
): void {
    const state = loadAppState() || { route: "/player" }
    const oldEpisodeUrl = state.episode?.url

    // Clear playback position if episode changed
    if (oldEpisodeUrl && episode?.url && oldEpisodeUrl !== episode.url) {
        delete state.playback_position
    } else if (!oldEpisodeUrl && episode?.url) {
        // New episode selected, clear any existing position
        delete state.playback_position
    }

    state.series = series
    state.episode = episode
    saveAppState(state)
}

/**
 * Restore route from localStorage on app startup
 * This function is no longer used - route restoration is handled in Rust
 * @deprecated Route restoration is now handled in Rust using router().push()
 */
export function restoreRouteFromState(): void {
    // This function is kept for backwards compatibility but does nothing
    // Route restoration is now handled in Rust in the Search component
}

/**
 * Settings interface for application preferences
 */
export interface Settings {
    auto_play_next?: boolean
    // Future settings can be added here
}

const SETTINGS_KEY = "wco-player-settings"

/**
 * Load settings from localStorage
 */
export function loadSettings(): Settings {
    try {
        const json = localStorage.getItem(SETTINGS_KEY)
        if (!json) {
            return {}
        }
        return JSON.parse(json) as Settings
    } catch {
        return {}
    }
}

/**
 * Save settings to localStorage
 */
export function saveSettings(settings: Settings): void {
    try {
        const json = JSON.stringify(settings)
        localStorage.setItem(SETTINGS_KEY, json)
    } catch {
        // Silently fail
    }
}

/**
 * Set a specific setting value
 */
export function setSetting<K extends keyof Settings>(
    key: K,
    value: Settings[K],
): void {
    const settings = loadSettings()
    settings[key] = value
    saveSettings(settings)
}

/**
 * Get a specific setting value
 */
export function getSetting<K extends keyof Settings>(
    key: K,
    defaultValue: Settings[K],
): Settings[K] {
    const settings = loadSettings()
    return settings[key] ?? defaultValue
}

/**
 * URL parameter management for player page
 */

/**
 * Get URL query parameters
 */
export function getUrlParams(): Record<string, string> {
    const params: Record<string, string> = {}
    const urlParams = new URLSearchParams(globalThis.location.search)
    for (const [key, value] of urlParams.entries()) {
        params[key] = value
    }
    return params
}

/**
 * Get a specific URL query parameter
 */
export function getUrlParam(key: string): string | null {
    const urlParams = new URLSearchParams(globalThis.location.search)
    return urlParams.get(key)
}

/**
 * Update URL query parameters without reloading the page
 */
export function updateUrlParams(params: Record<string, string | null>): void {
    const url = new URL(globalThis.location.href)
    for (const [key, value] of Object.entries(params)) {
        if (value === null || value === "") {
            url.searchParams.delete(key)
        } else {
            url.searchParams.set(key, value)
        }
    }
    globalThis.history.replaceState({}, "", url.toString())
}

/**
 * Get URL hash (without the #)
 */
export function getUrlHash(): string | null {
    const hash = globalThis.location.hash
    return hash ? hash.slice(1) : null
}

/**
 * Update URL hash without reloading the page
 */
export function updateUrlHash(hash: string | null): void {
    const url = new URL(globalThis.location.href)
    if (hash === null || hash === "") {
        url.hash = ""
    } else {
        url.hash = hash
    }
    globalThis.history.replaceState({}, "", url.toString())
}

/**
 * Format seconds to HH:mm:ss format
 */
function formatTime(seconds: number): string {
    const hours = Math.floor(seconds / 3600)
    const minutes = Math.floor((seconds % 3600) / 60)
    const secs = Math.floor(seconds % 60)
    return `${hours.toString().padStart(2, "0")}:${minutes.toString().padStart(2, "0")}:${
        secs.toString().padStart(2, "0")
    }`
}

/**
 * Update series, episode, and playback position in both URL and localStorage
 * This ensures they stay in sync
 */
export function updateSeriesEpisodeAndPosition(
    series: AppState["series"] | null,
    episode: AppState["episode"] | null,
    playbackPosition: number | null,
): void {
    // Update localStorage (convert null to undefined for compatibility)
    setSeriesAndEpisode(series ?? undefined, episode ?? undefined)

    // Update playback position in localStorage
    if (playbackPosition !== null && playbackPosition > 0) {
        setPlaybackPosition(playbackPosition)
    } else {
        setPlaybackPosition(null)
    }

    // Update URL query parameters
    if (series && episode) {
        updateUrlParams({
            series_url: series.url,
            episode_url: episode.url,
        })
    } else {
        // Clear URL params if no series/episode
        updateUrlParams({
            series_url: null,
            episode_url: null,
        })
    }

    // Update URL hash with playback position (or clear it if null)
    if (playbackPosition !== null && playbackPosition > 0) {
        updateUrlHash(formatTime(playbackPosition))
    } else {
        updateUrlHash(null)
    }
}
