// Video player fullscreen and overlay control logic
import { loadAppState, updatePlaybackPosition } from "./state_manager"

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

/**
 * Scroll to a specific episode in the episode list
 * Only scrolls if the episode is not fully visible in the viewport
 * @param episodeUrl - URL of the episode to scroll to
 */
export function scrollToEpisode(episodeUrl: string): void {
    // Find the episode item by data attribute
    const episodeItem = document.querySelector(
        `.episode-item[data-episode-url="${CSS.escape(episodeUrl)}"]`,
    ) as HTMLElement | null

    if (!episodeItem) {
        return
    }

    // Get the episode list container
    const episodeListContent = document.querySelector(
        ".episode-list-content",
    ) as HTMLElement | null

    if (!episodeListContent) {
        return
    }

    // Check if element is already fully visible
    if (isElementVisible(episodeItem, episodeListContent)) {
        return
    }

    // Calculate scroll position to center the episode item
    const containerRect = episodeListContent.getBoundingClientRect()
    const itemRect = episodeItem.getBoundingClientRect()
    const scrollTop = episodeListContent.scrollTop
    const itemOffsetTop = itemRect.top - containerRect.top + scrollTop
    const containerHeight = containerRect.height
    const itemHeight = itemRect.height

    // Scroll to center the item in the container
    const targetScrollTop = itemOffsetTop - containerHeight / 2 + itemHeight / 2

    // Smooth scroll to the target position
    episodeListContent.scrollTo({
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
    const selectedEpisode = document.querySelector(
        ".episode-item.selected",
    ) as HTMLElement | null

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
 * @param episodeUrl - URL of the episode to restore (optional, will load from localStorage if not provided)
 */
export function restorePlaybackEpisode(episodeUrl?: string): void {
    // If episodeUrl is not provided, try to load from localStorage
    let urlToScroll = episodeUrl
    if (!urlToScroll) {
        const state = loadAppState()
        if (state && state.episode) {
            urlToScroll = state.episode.url
        } else {
            return
        }
    }

    if (!urlToScroll) {
        return
    }

    // Wait for DOM to be ready, then scroll to the episode
    // Use MutationObserver to wait for episode list to be rendered
    const observer = new MutationObserver(() => {
        const episodeItem = document.querySelector(
            `.episode-item[data-episode-url="${CSS.escape(urlToScroll!)}"]`,
        ) as HTMLElement | null

        if (episodeItem) {
            // Episode item found, scroll to it
            observer.disconnect()
            scrollToEpisode(urlToScroll!)
        }
    })

    // Start observing
    observer.observe(document.body, {
        childList: true,
        subtree: true,
    })

    // Also try immediately in case the episode list is already rendered
    const episodeItem = document.querySelector(
        `.episode-item[data-episode-url="${CSS.escape(urlToScroll)}"]`,
    ) as HTMLElement | null

    if (episodeItem) {
        observer.disconnect()
        scrollToEpisode(urlToScroll)
    } else {
        // If not found immediately, set a timeout to stop observing after a reasonable time
        setTimeout(() => {
            observer.disconnect()
        }, 5000) // Stop observing after 5 seconds
    }
}

/**
 * Setup automatic scrolling when episode selection changes
 * This observes changes to the selected episode and automatically scrolls to it
 */
export function setupAutoScrollOnSelection(): void {
    // Use MutationObserver to watch for changes to the selected class
    const observer = new MutationObserver(() => {
        // Check if there's a selected episode
        const selectedEpisode = document.querySelector(
            ".episode-item.selected",
        ) as HTMLElement | null

        if (selectedEpisode) {
            // Small delay to ensure DOM is fully updated
            setTimeout(() => {
                scrollToSelectedEpisode()
            }, 50)
        }
    })

    // Observe the episode list container for class changes
    const episodeListContent = document.querySelector(
        ".episode-list-content",
    ) as HTMLElement | null

    if (episodeListContent) {
        observer.observe(episodeListContent, {
            attributes: true,
            attributeFilter: ["class"],
            childList: true,
            subtree: true,
        })
    } else {
        // If container doesn't exist yet, wait for it
        const waitObserver = new MutationObserver(() => {
            const container = document.querySelector(
                ".episode-list-content",
            ) as HTMLElement | null
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
 * Check if window is in fullscreen mode
 * In Dioxus Desktop (Wry), when window enters fullscreen, it takes up the entire screen
 */
export function checkWindowFullscreen(): boolean {
    const playerPage = document.getElementById("player-page")
    if (!playerPage) { return false }

    // Check if window dimensions match screen dimensions (with small tolerance)
    // This detects native macOS fullscreen button
    const windowWidth = globalThis.innerWidth
    const windowHeight = globalThis.innerHeight
    const screenWidth = screen.width
    const screenHeight = screen.height

    // Consider fullscreen if window takes up at least 95% of screen
    const isFullscreen = (windowWidth >= screenWidth * 0.95 &&
        windowHeight >= screenHeight * 0.95) ||
        // Also check if window is maximized (close to screen size)
        (Math.abs(windowWidth - screenWidth) < 10 &&
            Math.abs(windowHeight - screenHeight) < 10)

    // Update player-page class based on fullscreen state
    if (isFullscreen) {
        if (!playerPage.classList.contains("fullscreen-mode")) {
            playerPage.classList.add("fullscreen-mode")
        }
    } else {
        if (playerPage.classList.contains("fullscreen-mode")) {
            playerPage.classList.remove("fullscreen-mode")
        }
    }

    return isFullscreen
}

/**
 * Watch for fullscreen mode changes and update body/html styles
 */
export function updateFullscreenStyles(): void {
    const playerPage = document.getElementById("player-page")
    if (!playerPage) {
        return
    }

    // Check window fullscreen state first (for native button)
    checkWindowFullscreen()

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
export function setupFullscreenHover(): void {
    const playerPage = document.getElementById("player-page")
    if (!playerPage) {
        setTimeout(setupFullscreenHover, 100)
        return
    }

    // Update body/html styles for fullscreen
    updateFullscreenStyles()

    // Check if in fullscreen mode
    const isFullscreen = playerPage.classList.contains("fullscreen-mode")

    if (!isFullscreen) {
        // Ensure overlays are hidden when exiting fullscreen
        const header = document.getElementById("player-header")
        const sidebar = document.getElementById("episode-sidebar")
        if (header) { header.classList.remove("visible") }
        if (sidebar) { sidebar.classList.remove("visible") }
        return // Not in fullscreen, no need to setup hover
    }

    // Get header and sidebar dimensions
    const header = document.getElementById("player-header")
    const sidebar = document.getElementById("episode-sidebar")

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
    // Setup when DOM is ready
    if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", setupFullscreenHover)
    } else {
        setupFullscreenHover()
    }

    // Setup automatic scrolling when episode selection changes
    setupAutoScrollOnSelection()

    // Setup global observer to automatically track video elements when they're added to DOM
    const videoObserver = new MutationObserver(() => {
        const video = document.querySelector("video.video-element") as HTMLVideoElement | null
        if (video && !video.dataset.trackingSetup) {
            // Mark as setup to avoid duplicate setup
            video.dataset.trackingSetup = "true"
            setupPlaybackTracking("video-player", null)
        }
    })
    videoObserver.observe(document.body, { childList: true, subtree: true })

    // Also check immediately in case video already exists
    const existingVideo = document.querySelector("video.video-element") as HTMLVideoElement | null
    if (existingVideo && !existingVideo.dataset.trackingSetup) {
        existingVideo.dataset.trackingSetup = "true"
        setupPlaybackTracking("video-player", null)
    }

    // Watch for fullscreen mode changes
    const observer = new MutationObserver((mutations) => {
        mutations.forEach((mutation) => {
            if (mutation.type === "attributes" && mutation.attributeName === "class") {
                updateFullscreenStyles()
                setupFullscreenHover()
            }
        })
    })

    const playerPage = document.getElementById("player-page")
    if (playerPage) {
        // Initial update
        updateFullscreenStyles()

        observer.observe(playerPage, {
            attributes: true,
            attributeFilter: ["class"],
        })

        // Listen for window resize to detect native fullscreen button (macOS)
        globalThis.addEventListener("resize", () => {
            const wasFullscreen = playerPage.classList.contains("fullscreen-mode")
            const isNowFullscreen = checkWindowFullscreen()

            if (wasFullscreen !== isNowFullscreen) {
                updateFullscreenStyles()
                setupFullscreenHover()
            }
        })
    } else {
        // Retry if player-page not found yet
        setTimeout(() => {
            const playerPage = document.getElementById("player-page")
            if (playerPage) {
                updateFullscreenStyles()
                observer.observe(playerPage, {
                    attributes: true,
                    attributeFilter: ["class"],
                })

                // Listen for window resize to detect native fullscreen button (macOS)
                globalThis.addEventListener("resize", () => {
                    const wasFullscreen = playerPage.classList.contains("fullscreen-mode")
                    const isNowFullscreen = checkWindowFullscreen()

                    if (wasFullscreen !== isNowFullscreen) {
                        updateFullscreenStyles()
                        setupFullscreenHover()
                    }
                })
            }
        }, 100)
    }

    // Also check periodically in case resize event doesn't fire
    setInterval(() => {
        const playerPage = document.getElementById("player-page")
        if (playerPage) {
            const wasFullscreen = playerPage.classList.contains("fullscreen-mode")
            const isNowFullscreen = checkWindowFullscreen()

            if (wasFullscreen !== isNowFullscreen) {
                updateFullscreenStyles()
                setupFullscreenHover()
            }
        }
    }, 500)
}

// Playback position tracking
let playbackProgressInterval: number | null = null
let lastSavedPosition = 0

// Global state for playback tracking
let currentPlaybackPosition: number | null = null
let playbackEnded = false

/**
 * Setup playback position tracking for a video element
 * @param videoId - ID of the video element
 * @param savedPosition - Position to restore (in seconds)
 */
export function setupPlaybackTracking(
    _videoId: string,
    savedPosition: number | null,
): void {
    // Clean up existing interval
    if (playbackProgressInterval) {
        clearInterval(playbackProgressInterval)
        playbackProgressInterval = null
    }

    // Find video element by class (more reliable)
    const video = document.querySelector("video.video-element") as HTMLVideoElement | null
    if (!video) {
        // Wait for video element to be added to DOM
        const observer = new MutationObserver(() => {
            const videoElement = document.querySelector("video.video-element") as
                | HTMLVideoElement
                | null
            if (videoElement) {
                observer.disconnect()
                setupPlaybackTrackingForVideo(videoElement, savedPosition)
            }
        })
        observer.observe(document.body, { childList: true, subtree: true })
        // Also try again after a short delay as fallback
        setTimeout(() => {
            const videoElement = document.querySelector("video.video-element") as
                | HTMLVideoElement
                | null
            if (videoElement) {
                observer.disconnect()
                setupPlaybackTrackingForVideo(videoElement, savedPosition)
            }
        }, 100)
        return
    }

    setupPlaybackTrackingForVideo(video, savedPosition)
}

/**
 * Setup playback tracking for a specific video element
 */
function setupPlaybackTrackingForVideo(
    video: HTMLVideoElement,
    savedPosition: number | null,
): void {
    // Reset ended flag
    playbackEnded = false

    // Set autoplay attribute via JavaScript (since Dioxus may not support it directly in RSX)
    video.setAttribute("autoplay", "autoplay")

    // Get saved position from localStorage if not provided
    // But only if it matches the current episode
    let positionToRestore = savedPosition
    if (positionToRestore === null) {
        const state = loadAppState()
        if (state && state.playback_position && state.episode) {
            // Check if the saved episode matches the current video
            // We can't directly check the video URL, but we can check if episode URL matches
            // For now, we'll restore if episode exists in state
            // The episode URL should match what's in the DOM or be set by updateSeriesAndEpisode
            positionToRestore = state.playback_position
        } else if (state && state.playback_position && !state.episode) {
            // If there's a position but no episode, clear it (stale data)
            updatePlaybackPosition(null)
        }
    }

    // Restore saved position and auto-play
    if (positionToRestore !== null && positionToRestore > 0) {
        let positionRestored = false

        // Function to restore position - must wait for video to have enough data loaded
        const restorePosition = () => {
            // Wait for video to have enough data (readyState >= 2 means HAVE_CURRENT_DATA)
            // Also check that we haven't already restored
            if (!positionRestored && video.readyState >= 2 && video.duration > 0) {
                if (positionToRestore < video.duration) {
                    video.currentTime = positionToRestore
                    positionRestored = true
                    // Update global state so getCurrentPlaybackPosition returns correct value
                    currentPlaybackPosition = positionToRestore
                    return true
                }
            }
            return false
        }

        // Try to restore position immediately if video is already ready
        if (!restorePosition()) {
            // Wait for loadeddata event (fires when enough data is loaded to start playback)
            video.addEventListener("loadeddata", () => {
                if (!restorePosition()) {
                    // If still not ready, wait a bit more and try again
                    setTimeout(() => restorePosition(), 100)
                    setTimeout(() => restorePosition(), 300)
                }
            }, { once: true })

            // Also try on canplay event (fires when enough data is loaded to play)
            video.addEventListener("canplay", () => {
                if (!restorePosition()) {
                    setTimeout(() => restorePosition(), 200)
                }
            }, { once: true })

            // And on canplaythrough for maximum compatibility
            video.addEventListener("canplaythrough", () => {
                restorePosition()
            }, { once: true })
        }

        // Auto-play after position is restored
        const tryAutoPlay = () => {
            if (video.readyState >= 2 && video.paused && !video.ended) {
                video.play().catch(() => {})
                return true
            }
            return false
        }

        // Try multiple times to ensure playback starts
        const playAfterRestore = () => {
            if (!tryAutoPlay()) {
                // Retry after a short delay
                setTimeout(() => tryAutoPlay(), 100)
                setTimeout(() => tryAutoPlay(), 500)
            }
        }

        // Wait for canplay event before trying to play
        video.addEventListener("canplay", playAfterRestore, { once: true })
        // Also try on loadeddata as fallback
        video.addEventListener("loadeddata", () => {
            setTimeout(() => playAfterRestore(), 100)
        }, { once: true })
        // And on canplaythrough for maximum compatibility
        video.addEventListener("canplaythrough", playAfterRestore, { once: true })
    } else {
        // Auto-play if no saved position
        const tryAutoPlay = () => {
            if (video.readyState >= 2 && video.paused && !video.ended) {
                video.play().catch(() => {})
                return true
            }
            return false
        }

        // Try multiple times to ensure playback starts
        const playNewVideo = () => {
            if (!tryAutoPlay()) {
                // Retry after delays
                setTimeout(() => tryAutoPlay(), 100)
                setTimeout(() => tryAutoPlay(), 500)
            }
        }

        // Try immediately if video is already ready
        if (!tryAutoPlay()) {
            // Wait for canplay event
            video.addEventListener("canplay", playNewVideo, { once: true })
            // Also try on loadeddata as fallback
            video.addEventListener("loadeddata", playNewVideo, { once: true })
            // And on canplaythrough for maximum compatibility
            video.addEventListener("canplaythrough", playNewVideo, { once: true })
        } else {
            // If already playing, also retry after delay to ensure it continues
            setTimeout(() => tryAutoPlay(), 200)
        }
    }

    // Track playback progress every 2 seconds and save to localStorage
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

        const readyState = video.readyState
        const isPaused = video.paused
        const isEnded = video.ended
        const currentTime = video.currentTime

        if (readyState >= 2 && !isPaused && !isEnded) {
            // Update global state
            currentPlaybackPosition = currentTime

            // Only update if position changed significantly (> 1 second)
            if (Math.abs(currentTime - lastSavedPosition) > 1) {
                lastSavedPosition = currentTime
                // Save to localStorage
                updatePlaybackPosition(currentTime)
            }
        } else if (isEnded) {
            currentPlaybackPosition = null
            playbackEnded = true
            // Clear position when video ends
            updatePlaybackPosition(null)
        }
    }, 2000)

    // Clear position when video ends
    video.addEventListener("ended", () => {
        if (playbackProgressInterval) {
            clearInterval(playbackProgressInterval)
            playbackProgressInterval = null
        }
        currentPlaybackPosition = null
        playbackEnded = true
        // Clear position in localStorage
        updatePlaybackPosition(null)
        // Check if auto-play next is enabled
        const autoPlayNext = video.dataset.autoPlayNext === "true"
        if (autoPlayNext) {
            playNextEpisode()
        }
    }, { once: true })

    // Note: We don't clear interval on loadstart anymore
    // because loadstart fires when video src changes, which would stop tracking
    // The interval will be cleared when setupPlaybackTracking is called again for a new video
}

/**
 * Play the next episode automatically
 */
export function playNextEpisode(): void {
    // Find the currently selected episode
    const selectedEpisode = document.querySelector(".episode-item.selected") as HTMLElement | null
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
    const video = document.querySelector("video.video-element") as HTMLVideoElement | null
    if (video) {
        video.dataset.autoPlayNext = enabled ? "true" : "false"
    }
}

/**
 * Get current playback position
 */
export function getCurrentPlaybackPosition(): number | null {
    const video = document.querySelector("video.video-element") as HTMLVideoElement | null
    if (!video) {
        return null
    }

    // If video ended, return null to clear position
    if (video.ended || playbackEnded) {
        currentPlaybackPosition = null
        playbackEnded = false // Reset for next video
        return null
    }

    // Get position from video element if it has valid data
    // This ensures we get the actual currentTime, especially after restoring position
    if (video.readyState >= 2 && video.currentTime > 0) {
        const currentTime = video.currentTime
        // Update global state
        currentPlaybackPosition = currentTime
        return currentTime
    }

    // Fallback to global state if video not ready yet
    return currentPlaybackPosition
}
