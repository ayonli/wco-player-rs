// localStorage helper functions for state persistence

/**
 * Save application state to localStorage
 * @param {string} json - JSON string of the state
 */
export function saveStateToLocalStorage(json) {
    try {
        localStorage.setItem("wco-player-state", json)
        return { success: true }
    } catch (e) {
        console.error("Failed to save state to localStorage:", e)
        return { success: false, error: e.message }
    }
}

/**
 * Load application state from localStorage
 * @returns {string | null} - JSON string of the state, or null if not found
 */
export function loadStateFromLocalStorage() {
    try {
        return localStorage.getItem("wco-player-state")
    } catch (e) {
        console.error("Failed to load state from localStorage:", e)
        return null
    }
}
