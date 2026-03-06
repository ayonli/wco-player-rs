// Application state management using localStorage

export interface Series {
    title: string
    url: string
    thumbnail?: string
}

export interface Episode {
    title: string
    url: string
}

export interface AppState {
    route: string
    series?: Series
    episode?: Episode
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
    } catch (error) {
        console.error("Error saving app state:", error)
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
        return JSON.parse(json) as AppState
    } catch (error) {
        console.error("Error loading app state:", error)
        return null
    }
}

/**
 * Clear application state from localStorage
 */
export function clearAppState(): void {
    try {
        localStorage.removeItem(STATE_KEY)
    } catch (error) {
        console.error("Error clearing app state:", error)
    }
}

/**
 * Set the last route in saved state
 */
export function setLastRoute(route: string): void {
    const state = loadAppState() || { route: "/" }
    state.route = route
    saveAppState(state)
}

export type ThemePreference = "light" | "dark" | "system"

/**
 * Settings interface for application preferences
 */
export interface Settings {
    auto_play_next?: boolean
    theme_preference?: ThemePreference
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
    } catch (error) {
        console.error("Error loading settings:", error)
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
 * Get a specific URL query parameter
 */
export function getUrlParam(key: string): string | null {
    const urlParams = new URLSearchParams(globalThis.location.search)
    return urlParams.get(key)
}

/**
 * Update URL query parameters without reloading the page
 */
export function setUrlParams(params: Record<string, string | null>): void {
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
 * Get system color scheme (for theme preference "system")
 */
export function getSystemColorScheme(): "light" | "dark" {
    if (
        typeof globalThis.matchMedia !== "undefined" &&
        globalThis.matchMedia("(prefers-color-scheme: dark)").matches
    ) {
        return "dark"
    }
    return "light"
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
export function setUrlHash(hash: string | null): void {
    const url = new URL(globalThis.location.href)
    if (hash === null || hash === "") {
        url.hash = ""
    } else {
        url.hash = hash
    }
    globalThis.history.replaceState({}, "", url.toString())
}
