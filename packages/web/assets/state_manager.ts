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
 * Update only the playback position in saved state
 */
export function updatePlaybackPosition(position: number | null): void {
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
 * Update route in saved state
 */
export function updateRoute(route: string): void {
    const state = loadAppState() || { route: "/" }
    state.route = route
    saveAppState(state)
}

/**
 * Update series and episode in saved state
 * This also clears playback position when series/episode changes
 */
export function updateSeriesAndEpisode(
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
