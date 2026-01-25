import {
    type Episode,
    getSetting,
    loadAppState,
    saveAppState,
    type Series,
    setSetting,
    setUrlHash,
    setUrlParams,
} from "./state_manager"

function setSeriesAndEpisode(
    series: Series,
    episode: Episode | null,
): void {
    const state = loadAppState() || { route: "/player" }
    const oldEpisodeUrl = state.episode?.url

    // Clear playback position if episode changed
    if (
        (oldEpisodeUrl && episode?.url && oldEpisodeUrl !== episode.url) ||
        (!oldEpisodeUrl && episode?.url)
    ) {
        delete state.playback_position
    }

    state.series = series
    state.episode = episode ?? undefined
    saveAppState(state)
}

function setPlaybackPosition(position: number | null): void {
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
 * Format seconds to HH:mm:ss format
 */
function formatDuration(seconds: number): string {
    const hours = Math.floor(seconds / 3600).toString().padStart(2, "0")
    const minutes = Math.floor((seconds % 3600) / 60).toString().padStart(2, "0")
    const secs = Math.floor(seconds % 60).toString().padStart(2, "0")
    return `${hours}:${minutes}:${secs}`
}

/**
 * Update series, episode, and playback position in both URL and localStorage
 * This ensures they stay in sync
 */
export function savePlayerState(
    series: Series,
    episode: Episode,
    playbackPosition: number | null = 0,
): void {
    setSeriesAndEpisode(series, episode)
    setUrlParams({
        series_url: series.url,
        episode_url: episode.url,
    })

    if (playbackPosition && playbackPosition > 0) {
        setPlaybackPosition(playbackPosition)
        setUrlHash(formatDuration(playbackPosition))
    } else {
        setPlaybackPosition(null)
        setUrlHash(null)
    }
}

/**
 * Check if an element is fully visible within its scrollable container
 */
function isElementVisible(
    element: HTMLElement,
    container: HTMLElement,
): boolean {
    const containerRect = container.getBoundingClientRect()
    const elementRect = element.getBoundingClientRect()

    // Check if element is within container's visible area
    return (
        elementRect.top >= containerRect.top &&
        elementRect.bottom <= containerRect.bottom
    )
}

function getEpisodeListContainer(): HTMLElement | null {
    return document.querySelector(".episode-list-content") as HTMLElement | null
}

function getSelectedEpisodeElement(): HTMLElement | null {
    return document.querySelector(".episode-item.selected") as HTMLElement | null
}

function getEpisodeItemElement(episodeUrl: string): HTMLElement | null {
    return document.querySelector(
        `.episode-item[data-episode-url="${CSS.escape(episodeUrl)}"]`,
    ) as HTMLElement | null
}

function getVideoElement(): HTMLVideoElement | null {
    return document.querySelector("video.video-element") as HTMLVideoElement | null
}

/**
 * Scroll to a specific episode in the episode list
 * Only scrolls if the episode is not fully visible in the viewport
 * @param episodeUrl - URL of the episode to scroll to
 */
export function scrollToEpisode(episodeUrl: string): void {
    // Find the episode item by data attribute
    const episodeItem = getEpisodeItemElement(episodeUrl)
    if (!episodeItem) {
        return
    }

    // Get the episode list container
    const episodeList = getEpisodeListContainer()
    if (!episodeList) {
        return
    }

    // Check if element is already fully visible
    if (isElementVisible(episodeItem, episodeList)) {
        return
    }

    // Calculate scroll position to center the episode item
    const containerRect = episodeList.getBoundingClientRect()
    const itemRect = episodeItem.getBoundingClientRect()
    const scrollTop = episodeList.scrollTop
    const itemOffsetTop = itemRect.top - containerRect.top + scrollTop
    const containerHeight = containerRect.height
    const itemHeight = itemRect.height

    // Smooth scroll to the target position
    const targetScrollTop = itemOffsetTop - containerHeight / 2 + itemHeight / 2
    episodeList.scrollTo({
        top: Math.max(0, targetScrollTop),
        behavior: "smooth",
    })
}

/**
 * Scroll to the currently selected episode
 * This function automatically finds the selected episode and scrolls to it
 */
export function scrollToSelectedEpisode(): void {
    // Find the selected episode item
    const selectedEpisode = getSelectedEpisodeElement()
    if (!selectedEpisode) {
        return
    }

    // Get the episode URL from data attribute
    const episodeUrl = selectedEpisode.getAttribute("data-episode-url")
    if (!episodeUrl) {
        return
    }

    // Scroll to the selected episode
    scrollToEpisode(episodeUrl)
}

/**
 * Restore playback episode from localStorage
 * This function scrolls to the saved episode in the episode list
 * @param episodeUrl - URL of the episode to restore
 */
export function restorePlaybackEpisode(episodeUrl: string): void {
    const episodeItem = getEpisodeItemElement(episodeUrl)

    if (episodeItem) {
        scrollToEpisode(episodeUrl)
    } else {
        // Use MutationObserver to wait for episode list to be rendered
        const observer = new MutationObserver(() => {
            const episodeItem = getEpisodeItemElement(episodeUrl)
            if (episodeItem) {
                observer.disconnect()
                scrollToEpisode(episodeUrl)
            }
        })

        // Start observing
        observer.observe(document.body, {
            childList: true,
            subtree: true,
        })

        // Set a timeout to stop observing after a reasonable time
        setTimeout(() => {
            observer.disconnect()
        }, 5000)
    }
}

/**
 * Setup automatic scrolling when episode selection changes
 * This observes changes to the selected episode and automatically scrolls to it
 */
function setupAutoScrollOnSelection(): void {
    // Use MutationObserver to watch for changes to the selected class
    const observer = new MutationObserver(() => {
        const selectedEpisode = getSelectedEpisodeElement()
        if (selectedEpisode) {
            // Small delay to ensure DOM is fully updated
            setTimeout(() => {
                scrollToSelectedEpisode()
            }, 100)
        }
    })

    // Observe the episode list container for class changes
    const container = getEpisodeListContainer()

    if (container) {
        observer.observe(container, {
            attributes: true,
            attributeFilter: ["class"],
            childList: true,
            subtree: true,
        })
    } else {
        // If container doesn't exist yet, wait for it
        const waitObserver = new MutationObserver(() => {
            const container = getEpisodeListContainer()
            if (container) {
                waitObserver.disconnect()
                observer.observe(container, {
                    attributes: true,
                    attributeFilter: ["class"],
                    childList: true,
                    subtree: true,
                })
            }
        })
        waitObserver.observe(document.body, {
            childList: true,
            subtree: true,
        })
        // Stop waiting after 10 seconds
        setTimeout(() => {
            waitObserver.disconnect()
        }, 10000)
    }
}

const HIDE_DELAY = 3000 // 3 seconds
const EDGE_THRESHOLD = 50 // pixels from edge

let headerHideTimeout: number | null = null
let sidebarHideTimeout: number | null = null

/**
 * Check if player-page has fullscreen-mode class
 * This can be called from Rust to sync state
 */
export function isPlayerPageFullscreen(): boolean {
    const playerPage = document.getElementById("player-page")
    return playerPage ? playerPage.classList.contains("fullscreen-mode") : false
}

/**
 * Update the fullscreen class on player-page element
 * Called from Rust when fullscreen state changes (desktop only)
 * @param isFullscreen - Whether the window is in fullscreen mode
 */
export function setFullscreenMode(isFullscreen: boolean): void {
    const playerPage = document.getElementById("player-page")
    if (!playerPage) { return }

    if (isFullscreen) {
        if (!playerPage.classList.contains("fullscreen-mode")) {
            playerPage.classList.add("fullscreen-mode")
        }
    } else {
        if (playerPage.classList.contains("fullscreen-mode")) {
            playerPage.classList.remove("fullscreen-mode")
        }
    }

    toggleFullscreenStyles()
    setupFullscreenHover()
}

/**
 * Watch for fullscreen mode changes and update body/html styles
 */
function toggleFullscreenStyles(): void {
    const playerPage = document.getElementById("player-page")
    if (!playerPage) {
        return
    }

    const isFullscreen = playerPage.classList.contains("fullscreen-mode")

    if (isFullscreen) {
        document.body.style.margin = "0"
        document.body.style.padding = "0"
        document.body.style.overflow = "hidden"
        document.body.style.width = "100vw"
        document.body.style.height = "100vh"
        document.body.style.position = "fixed"
        document.body.style.top = "0"
        document.body.style.left = "0"
        document.documentElement.style.margin = "0"
        document.documentElement.style.padding = "0"
        document.documentElement.style.overflow = "hidden"
        document.documentElement.style.width = "100vw"
        document.documentElement.style.height = "100vh"
    } else {
        document.body.style.margin = ""
        document.body.style.padding = ""
        document.body.style.overflow = ""
        document.body.style.width = ""
        document.body.style.height = ""
        document.body.style.position = ""
        document.body.style.top = ""
        document.body.style.left = ""
        document.documentElement.style.margin = ""
        document.documentElement.style.padding = ""
        document.documentElement.style.overflow = ""
        document.documentElement.style.width = ""
        document.documentElement.style.height = ""
    }
}

function showHeader(): void {
    const header = document.getElementById("player-header")
    if (header) {
        // Clear any existing hide timeout
        if (headerHideTimeout) {
            clearTimeout(headerHideTimeout)
            headerHideTimeout = null
        }
        header.classList.add("visible")
    }
}

function hideHeader(): void {
    const header = document.getElementById("player-header")
    if (header && header.classList.contains("visible")) {
        // Only start hide timer if not already started
        // This ensures that once mouse leaves, timer starts and won't be reset by subsequent mouse moves
        if (!headerHideTimeout) {
            headerHideTimeout = setTimeout(() => {
                header.classList.remove("visible")
                headerHideTimeout = null
            }, HIDE_DELAY)
        }
    }
}

function showSidebar(): void {
    const sidebar = document.getElementById("episode-sidebar")
    if (sidebar) {
        // Clear any existing hide timeout
        if (sidebarHideTimeout) {
            clearTimeout(sidebarHideTimeout)
            sidebarHideTimeout = null
        }
        sidebar.classList.add("visible")
    }
}

function hideSidebar(): void {
    const sidebar = document.getElementById("episode-sidebar")
    if (sidebar && sidebar.classList.contains("visible")) {
        // Only start hide timer if not already started
        // This ensures that once mouse leaves, timer starts and won't be reset by subsequent mouse moves
        if (!sidebarHideTimeout) {
            sidebarHideTimeout = setTimeout(() => {
                sidebar.classList.remove("visible")
                sidebarHideTimeout = null
            }, HIDE_DELAY)
        }
    }
}

/**
 * Setup fullscreen hover functionality
 */
function setupFullscreenHover(): void {
    const playerPage = document.getElementById("player-page")
    if (!playerPage) {
        setTimeout(setupFullscreenHover, 100)
        return
    }

    // Get header and sidebar dimensions
    const header = document.getElementById("player-header")
    const sidebar = document.getElementById("episode-sidebar")

    // Check if in fullscreen mode
    const isFullscreen = playerPage.classList.contains("fullscreen-mode")
    if (!isFullscreen) {
        // Ensure overlays are hidden when exiting fullscreen
        header?.classList.remove("visible")
        sidebar?.classList.remove("visible")
        return
    }

    // Remove old event listeners if any
    // deno-lint-ignore no-explicit-any
    const oldHandler = (playerPage as any).__fullscreenMouseMoveHandler
    if (oldHandler) {
        playerPage.removeEventListener("mousemove", oldHandler)
    }

    // Mouse move handler
    function handleMouseMove(e: MouseEvent): void {
        const x = e.clientX
        const y = e.clientY

        // Check header visibility
        if (header) {
            const isNearTop = y < EDGE_THRESHOLD

            // When header is visible, check if mouse is in it
            let isInHeader = false
            if (header.classList.contains("visible")) {
                const headerRect = header.getBoundingClientRect()
                // Check if mouse is actually in the visible header area
                // When visible, header should be at top: 0, so check y within header height
                isInHeader = y >= headerRect.top && y <= headerRect.bottom &&
                    x >= headerRect.left && x <= headerRect.right &&
                    headerRect.top >= 0 // Ensure header is actually visible (not translated out)
            }

            if (isNearTop || isInHeader) {
                // Mouse is in header area, show it
                showHeader()
            } else {
                // Mouse is outside header area, start hide timer
                // Don't check for buffer - if mouse is outside, start hiding
                hideHeader()
            }
        }

        // Check sidebar visibility
        if (sidebar) {
            const isNearLeft = x < EDGE_THRESHOLD

            // When sidebar is visible, check if mouse is in it
            let isInSidebar = false
            if (sidebar.classList.contains("visible")) {
                const sidebarRect = sidebar.getBoundingClientRect()
                // Check if mouse is actually in the visible sidebar area
                // When visible, sidebar should be at left: 0, so check x within sidebar width
                isInSidebar = x >= sidebarRect.left && x <= sidebarRect.right &&
                    y >= sidebarRect.top && y <= sidebarRect.bottom &&
                    sidebarRect.left >= 0 // Ensure sidebar is actually visible (not translated out)
            }

            if (isNearLeft || isInSidebar) {
                // Mouse is in sidebar area, show it
                showSidebar()
            } else {
                // Mouse is outside sidebar area, start hide timer
                // Don't check for buffer - if mouse is outside, start hiding
                hideSidebar()
            }
        }
    }

    // Store handler reference for cleanup
    // deno-lint-ignore no-explicit-any
    const existingHandler = (playerPage as any).__fullscreenMouseMoveHandler
    if (existingHandler) {
        playerPage.removeEventListener("mousemove", existingHandler)
    }

    Object.assign(playerPage, { __fullscreenMouseMoveHandler: handleMouseMove })
    playerPage.addEventListener("mousemove", handleMouseMove)

    // Keep visible when hovering over the elements
    if (header) {
        header.addEventListener("mouseenter", showHeader)
        header.addEventListener("mouseleave", () => {
            // Will be handled by mousemove event
        })
    }

    if (sidebar) {
        sidebar.addEventListener("mouseenter", showSidebar)
        sidebar.addEventListener("mouseleave", () => {
            // Will be handled by mousemove event
        })
    }
}

/**
 * Initialize video player controls
 */
export function initVideoPlayerControls(): void {
    const setup = () => {
        toggleFullscreenStyles()
        setupFullscreenHover()
        setupAutoScrollOnSelection()
    }

    // Setup when DOM is ready
    if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", setup)
    } else {
        setup()
    }

    // Also check immediately in case video already exists
    const video = getVideoElement()
    if (video) {
        if (!video.dataset.trackingSetup) {
            video.dataset.trackingSetup = "true"
            // Position will be set by Rust code when video info is loaded
            setupPlaybackTracking("video-player", 0)
        }
    } else {
        const observer = new MutationObserver(() => {
            const video = getVideoElement()
            if (video && !video.dataset.trackingSetup) {
                // Mark as setup to avoid duplicate setup
                video.dataset.trackingSetup = "true"
                // Position will be set by Rust code when video info is loaded
                setupPlaybackTracking("video-player", 0)
            }
        })
        observer.observe(document.body, { childList: true, subtree: true })
    }

    const watchFullscreenModeChanges = () => {
        const playerPage = document.getElementById("player-page")
        if (playerPage) {
            const observer = new MutationObserver((mutations) => {
                mutations.forEach((mutation) => {
                    if (mutation.type === "attributes" && mutation.attributeName === "class") {
                        toggleFullscreenStyles()
                        setupFullscreenHover()
                    }
                })
            })
            observer.observe(playerPage, {
                attributes: true,
                attributeFilter: ["class"],
            })
            return true
        }

        return false
    }

    if (!watchFullscreenModeChanges()) {
        // Retry after a short delay
        setTimeout(() => {
            watchFullscreenModeChanges()
        }, 1000)
    }
}

// Playback position tracking
let playbackProgressInterval: number | null = null
let lastSavedPosition = 0

/**
 * Setup playback position tracking for a video element
 * @param videoId - ID of the video element
 * @param savedPosition - Position to restore (in seconds), must be a number (0 if no saved position)
 */
export function setupPlaybackTracking(
    _videoId: string,
    savedPosition: number,
): void {
    // Clean up existing interval
    if (playbackProgressInterval) {
        clearInterval(playbackProgressInterval)
        playbackProgressInterval = null
    }

    const video = getVideoElement()
    if (video) {
        setupPlaybackTrackingForVideo(video, savedPosition)
        return
    } else {
        // Wait for video element to be added to DOM
        const observer = new MutationObserver(() => {
            const video = getVideoElement()
            if (video) {
                observer.disconnect()
                setupPlaybackTrackingForVideo(video, savedPosition)
            }
        })
        observer.observe(document.body, { childList: true, subtree: true })
    }
}

/**
 * Setup playback tracking for a specific video element
 * @param video - The video element to track
 * @param savedPosition - Position to restore (in seconds), must be a number (0 if no saved position)
 */
function setupPlaybackTrackingForVideo(
    video: HTMLVideoElement,
    savedPosition: number,
): void {
    // Set autoplay attribute
    video.setAttribute("autoplay", "autoplay")

    // Restore position and start playback
    if (savedPosition > 0) {
        // Wait for video metadata to be loaded before restoring position
        const restoreAndPlay = () => {
            if (video.readyState >= 1 && video.duration > 0) {
                if (savedPosition < video.duration) {
                    video.currentTime = savedPosition
                }
                // Start playback when video can play
                if (video.paused && !video.ended) {
                    video.play().catch(console.error)
                }
            }
        }

        // Restore position when metadata is loaded
        if (video.readyState >= 1) {
            restoreAndPlay()
        } else {
            video.addEventListener("loadedmetadata", restoreAndPlay, { once: true })
        }

        // Ensure playback starts when video is ready
        video.addEventListener("canplay", () => {
            if (video.paused && !video.ended) {
                video.play().catch(console.error)
            }
        }, { once: true })
    } else {
        // Auto-play if no saved position
        const startPlayback = () => {
            if (video.paused && !video.ended) {
                video.play().catch(console.error)
            }
        }

        if (video.readyState >= 1) {
            startPlayback()
        } else {
            video.addEventListener("canplay", startPlayback, { once: true })
        }
    }

    // Track playback progress every 2 seconds
    lastSavedPosition = 0

    // Clear any existing interval first
    if (playbackProgressInterval) {
        clearInterval(playbackProgressInterval)
        playbackProgressInterval = null
    }

    playbackProgressInterval = setInterval(() => {
        // Check if video element still exists
        if (!video || !document.contains(video)) {
            if (playbackProgressInterval) {
                clearInterval(playbackProgressInterval)
                playbackProgressInterval = null
            }
            return
        }

        if (video.readyState >= 1 && !video.paused && !video.ended) {
            const currentTime = video.currentTime

            // Only update if position changed significantly (> 1 second)
            if (Math.abs(currentTime - lastSavedPosition) > 1) {
                lastSavedPosition = currentTime
                // Update both localStorage and URL using unified function
                const state = loadAppState()
                if (state?.series && state?.episode) {
                    savePlayerState(
                        state.series,
                        state.episode,
                        currentTime,
                    )
                }
            }
        } else if (video.ended) {
            // Clear playback position using unified function
            const state = loadAppState()
            if (state?.series && state?.episode) {
                savePlayerState(
                    state.series,
                    state.episode,
                    null,
                )
            }
        }
    }, 2000)

    // Handle video end
    video.addEventListener("ended", () => {
        if (playbackProgressInterval) {
            clearInterval(playbackProgressInterval)
            playbackProgressInterval = null
        }
        // Clear playback position using unified function
        const state = loadAppState()
        if (state?.series && state?.episode) {
            savePlayerState(
                state.series,
                state.episode,
                null,
            )
        }

        // Auto-play next episode if enabled
        const autoPlayNext = getSetting("auto_play_next", false)
        if (autoPlayNext) {
            playNextEpisode()
        }
    }, { once: true })
}

/**
 * Play the next episode automatically
 */
function playNextEpisode(): void {
    // Find the currently selected episode
    const selectedEpisode = getSelectedEpisodeElement()
    if (!selectedEpisode) {
        return
    }

    // Find the next episode (sibling element)
    const nextEpisode = selectedEpisode.nextElementSibling as HTMLElement | null
    if (!nextEpisode || !nextEpisode.classList.contains("episode-item")) {
        // No next episode, do nothing
        return
    }

    // Click the next episode to play it
    nextEpisode.click()

    // After clicking, wait a bit for the selection to update, then scroll
    // The auto-scroll observer will handle this, but we can also trigger it directly
    setTimeout(() => {
        scrollToSelectedEpisode()
    }, 100)
}

/**
 * Set auto-play next episode state
 * @param enabled - Whether auto-play next is enabled
 */
export function setAutoPlayNext(enabled: boolean): void {
    setSetting("auto_play_next", enabled)
}
