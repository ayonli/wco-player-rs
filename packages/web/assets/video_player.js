// Video player fullscreen and overlay control logic
const HIDE_DELAY = 3000; // 3 seconds
const EDGE_THRESHOLD = 50; // pixels from edge
let headerHideTimeout = null;
let sidebarHideTimeout = null;
/**
 * Check if player-page has fullscreen-mode class
 * This can be called from Rust to sync state
 */
export function isPlayerPageFullscreen() {
    const playerPage = document.getElementById("player-page");
    return playerPage ? playerPage.classList.contains("fullscreen-mode") : false;
}
/**
 * Check if window is in fullscreen mode
 * In Dioxus Desktop (Wry), when window enters fullscreen, it takes up the entire screen
 */
export function checkWindowFullscreen() {
    const playerPage = document.getElementById("player-page");
    if (!playerPage) {
        return false;
    }
    // Check if window dimensions match screen dimensions (with small tolerance)
    // This detects native macOS fullscreen button
    const windowWidth = globalThis.innerWidth;
    const windowHeight = globalThis.innerHeight;
    const screenWidth = screen.width;
    const screenHeight = screen.height;
    // Consider fullscreen if window takes up at least 95% of screen
    const isFullscreen = (windowWidth >= screenWidth * 0.95 &&
        windowHeight >= screenHeight * 0.95) ||
        // Also check if window is maximized (close to screen size)
        (Math.abs(windowWidth - screenWidth) < 10 &&
            Math.abs(windowHeight - screenHeight) < 10);
    // Update player-page class based on fullscreen state
    if (isFullscreen) {
        if (!playerPage.classList.contains("fullscreen-mode")) {
            console.log("🔄 Window entered fullscreen (native button)");
            playerPage.classList.add("fullscreen-mode");
        }
    }
    else {
        if (playerPage.classList.contains("fullscreen-mode")) {
            console.log("🔄 Window exited fullscreen (native button)");
            playerPage.classList.remove("fullscreen-mode");
        }
    }
    return isFullscreen;
}
/**
 * Watch for fullscreen mode changes and update body/html styles
 */
export function updateFullscreenStyles() {
    const playerPage = document.getElementById("player-page");
    if (!playerPage) {
        console.log("⚠️ player-page not found");
        return;
    }
    // Check window fullscreen state first (for native button)
    checkWindowFullscreen();
    const isFullscreen = playerPage.classList.contains("fullscreen-mode");
    console.log("🎬 updateFullscreenStyles: isFullscreen =", isFullscreen);
    if (isFullscreen) {
        console.log("✅ Applying fullscreen styles to body/html");
        document.body.style.margin = "0";
        document.body.style.padding = "0";
        document.body.style.overflow = "hidden";
        document.body.style.width = "100vw";
        document.body.style.height = "100vh";
        document.body.style.position = "fixed";
        document.body.style.top = "0";
        document.body.style.left = "0";
        document.documentElement.style.margin = "0";
        document.documentElement.style.padding = "0";
        document.documentElement.style.overflow = "hidden";
        document.documentElement.style.width = "100vw";
        document.documentElement.style.height = "100vh";
    }
    else {
        console.log("🔄 Removing fullscreen styles from body/html");
        document.body.style.margin = "";
        document.body.style.padding = "";
        document.body.style.overflow = "";
        document.body.style.width = "";
        document.body.style.height = "";
        document.body.style.position = "";
        document.body.style.top = "";
        document.body.style.left = "";
        document.documentElement.style.margin = "";
        document.documentElement.style.padding = "";
        document.documentElement.style.overflow = "";
        document.documentElement.style.width = "";
        document.documentElement.style.height = "";
    }
}
function showHeader() {
    const header = document.getElementById("player-header");
    if (header) {
        // Clear any existing hide timeout
        if (headerHideTimeout) {
            clearTimeout(headerHideTimeout);
            headerHideTimeout = null;
        }
        header.classList.add("visible");
    }
}
function hideHeader() {
    const header = document.getElementById("player-header");
    if (header && header.classList.contains("visible")) {
        // Only start hide timer if not already started
        // This ensures that once mouse leaves, timer starts and won't be reset by subsequent mouse moves
        if (!headerHideTimeout) {
            headerHideTimeout = setTimeout(() => {
                header.classList.remove("visible");
                headerHideTimeout = null;
            }, HIDE_DELAY);
        }
    }
}
function showSidebar() {
    const sidebar = document.getElementById("episode-sidebar");
    if (sidebar) {
        // Clear any existing hide timeout
        if (sidebarHideTimeout) {
            clearTimeout(sidebarHideTimeout);
            sidebarHideTimeout = null;
        }
        sidebar.classList.add("visible");
    }
}
function hideSidebar() {
    const sidebar = document.getElementById("episode-sidebar");
    if (sidebar && sidebar.classList.contains("visible")) {
        // Only start hide timer if not already started
        // This ensures that once mouse leaves, timer starts and won't be reset by subsequent mouse moves
        if (!sidebarHideTimeout) {
            sidebarHideTimeout = setTimeout(() => {
                sidebar.classList.remove("visible");
                sidebarHideTimeout = null;
            }, HIDE_DELAY);
        }
    }
}
/**
 * Setup fullscreen hover functionality
 */
export function setupFullscreenHover() {
    const playerPage = document.getElementById("player-page");
    if (!playerPage) {
        console.log("⚠️ player-page not found, retrying...");
        setTimeout(setupFullscreenHover, 100);
        return;
    }
    // Update body/html styles for fullscreen
    updateFullscreenStyles();
    // Check if in fullscreen mode
    const isFullscreen = playerPage.classList.contains("fullscreen-mode");
    console.log("🎬 setupFullscreenHover: isFullscreen=", isFullscreen);
    if (!isFullscreen) {
        // Ensure overlays are hidden when exiting fullscreen
        const header = document.getElementById("player-header");
        const sidebar = document.getElementById("episode-sidebar");
        if (header) {
            header.classList.remove("visible");
        }
        if (sidebar) {
            sidebar.classList.remove("visible");
        }
        return; // Not in fullscreen, no need to setup hover
    }
    // Get header and sidebar dimensions
    const header = document.getElementById("player-header");
    const sidebar = document.getElementById("episode-sidebar");
    // Remove old event listeners if any
    // deno-lint-ignore no-explicit-any
    const oldHandler = playerPage.__fullscreenMouseMoveHandler;
    if (oldHandler) {
        playerPage.removeEventListener("mousemove", oldHandler);
    }
    // Mouse move handler
    function handleMouseMove(e) {
        const x = e.clientX;
        const y = e.clientY;
        // Check header visibility
        if (header) {
            const isNearTop = y < EDGE_THRESHOLD;
            // When header is visible, check if mouse is in it
            let isInHeader = false;
            if (header.classList.contains("visible")) {
                const headerRect = header.getBoundingClientRect();
                // Check if mouse is actually in the visible header area
                // When visible, header should be at top: 0, so check y within header height
                isInHeader = y >= headerRect.top && y <= headerRect.bottom &&
                    x >= headerRect.left && x <= headerRect.right &&
                    headerRect.top >= 0; // Ensure header is actually visible (not translated out)
            }
            if (isNearTop || isInHeader) {
                // Mouse is in header area, show it
                showHeader();
            }
            else {
                // Mouse is outside header area, start hide timer
                // Don't check for buffer - if mouse is outside, start hiding
                hideHeader();
            }
        }
        // Check sidebar visibility
        if (sidebar) {
            const isNearLeft = x < EDGE_THRESHOLD;
            // When sidebar is visible, check if mouse is in it
            let isInSidebar = false;
            if (sidebar.classList.contains("visible")) {
                const sidebarRect = sidebar.getBoundingClientRect();
                // Check if mouse is actually in the visible sidebar area
                // When visible, sidebar should be at left: 0, so check x within sidebar width
                isInSidebar = x >= sidebarRect.left && x <= sidebarRect.right &&
                    y >= sidebarRect.top && y <= sidebarRect.bottom &&
                    sidebarRect.left >= 0; // Ensure sidebar is actually visible (not translated out)
            }
            if (isNearLeft || isInSidebar) {
                // Mouse is in sidebar area, show it
                showSidebar();
            }
            else {
                // Mouse is outside sidebar area, start hide timer
                // Don't check for buffer - if mouse is outside, start hiding
                hideSidebar();
            }
        }
    }
    // Store handler reference for cleanup
    // deno-lint-ignore no-explicit-any
    const existingHandler = playerPage.__fullscreenMouseMoveHandler;
    if (existingHandler) {
        console.log("🔄 Removing old mousemove handler");
        playerPage.removeEventListener("mousemove", existingHandler);
    }
    Object.assign(playerPage, { __fullscreenMouseMoveHandler: handleMouseMove });
    playerPage.addEventListener("mousemove", handleMouseMove);
    console.log("✅ Added mousemove handler for fullscreen hover");
    // Keep visible when hovering over the elements
    if (header) {
        header.addEventListener("mouseenter", showHeader);
        header.addEventListener("mouseleave", () => {
            // Will be handled by mousemove event
        });
    }
    if (sidebar) {
        sidebar.addEventListener("mouseenter", showSidebar);
        sidebar.addEventListener("mouseleave", () => {
            // Will be handled by mousemove event
        });
    }
}
/**
 * Initialize video player controls
 */
export function initVideoPlayerControls() {
    console.log("🚀 initVideoPlayerControls called");
    // Setup when DOM is ready
    if (document.readyState === "loading") {
        console.log("⏳ Document still loading, waiting for DOMContentLoaded");
        document.addEventListener("DOMContentLoaded", setupFullscreenHover);
    }
    else {
        console.log("✅ Document ready, calling setupFullscreenHover");
        setupFullscreenHover();
    }
    // Watch for fullscreen mode changes
    const observer = new MutationObserver((mutations) => {
        mutations.forEach((mutation) => {
            if (mutation.type === "attributes" && mutation.attributeName === "class") {
                console.log("🔄 Class changed, updating styles...");
                updateFullscreenStyles();
                setupFullscreenHover();
            }
        });
    });
    const playerPage = document.getElementById("player-page");
    if (playerPage) {
        // Initial update
        updateFullscreenStyles();
        observer.observe(playerPage, {
            attributes: true,
            attributeFilter: ["class"],
        });
        // Listen for window resize to detect native fullscreen button (macOS)
        globalThis.addEventListener("resize", () => {
            console.log("🔄 Window resized, checking fullscreen state...");
            const wasFullscreen = playerPage.classList.contains("fullscreen-mode");
            const isNowFullscreen = checkWindowFullscreen();
            if (wasFullscreen !== isNowFullscreen) {
                console.log("🔄 Fullscreen state changed via native button:", isNowFullscreen);
                updateFullscreenStyles();
                setupFullscreenHover();
            }
        });
    }
    else {
        // Retry if player-page not found yet
        setTimeout(() => {
            const playerPage = document.getElementById("player-page");
            if (playerPage) {
                updateFullscreenStyles();
                observer.observe(playerPage, {
                    attributes: true,
                    attributeFilter: ["class"],
                });
                // Listen for window resize to detect native fullscreen button (macOS)
                globalThis.addEventListener("resize", () => {
                    const wasFullscreen = playerPage.classList.contains("fullscreen-mode");
                    const isNowFullscreen = checkWindowFullscreen();
                    if (wasFullscreen !== isNowFullscreen) {
                        updateFullscreenStyles();
                        setupFullscreenHover();
                    }
                });
            }
        }, 100);
    }
    // Also check periodically in case resize event doesn't fire
    setInterval(() => {
        const playerPage = document.getElementById("player-page");
        if (playerPage) {
            const wasFullscreen = playerPage.classList.contains("fullscreen-mode");
            const isNowFullscreen = checkWindowFullscreen();
            if (wasFullscreen !== isNowFullscreen) {
                console.log("🔄 Fullscreen state changed (periodic check):", isNowFullscreen);
                updateFullscreenStyles();
                setupFullscreenHover();
            }
        }
    }, 500);
}
