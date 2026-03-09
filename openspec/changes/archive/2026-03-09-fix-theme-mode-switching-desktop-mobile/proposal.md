## Why

The light/dark theme toggle works on web but is a silent no-op on the desktop and mobile apps. All three platforms are built on Dioxus WebView and share the same `web::App` root component, so the underlying web APIs (`document::eval`, `localStorage`, `window.matchMedia`) are available on every target — the only thing preventing them from working is a set of `#[cfg(feature = "web")]` guards that exclude the relevant code when building for desktop or mobile.

## What Changes

- Remove `#[cfg(feature = "web")]` / `#[cfg(not(feature = "web"))]` guards from `apply_theme_to_document` and replace with `#[cfg(not(feature = "server"))]` so theme application runs on all WebView-capable targets.
- Remove `#[cfg(feature = "web")]` guard from the system color-scheme polling `use_effect` in `App` so `window.matchMedia` is observed on desktop and mobile too.
- Replace `#[cfg(feature = "web")]` guards on theme preference *loading* and *saving* (which call `video_js::getSetting` / `setSetting` — WASM-only `dioxus-use-js` bindings) with `#[cfg(not(feature = "server"))]`, and rewrite those code paths using direct `document::eval` + `localStorage` calls so they work in the WebView on all three targets.

## Capabilities

### New Capabilities

- (none)

### Modified Capabilities

- `theme-preference`: Extends the existing persistence and system-scheme detection requirements to explicitly cover desktop and mobile targets, not just the web browser.

## Impact

- **`packages/web/src/theme.rs`**: Remove the duplicate `apply_theme_to_document` stub; keep a single implementation guarded by `#[cfg(not(feature = "server"))]`.
- **`packages/web/src/views/app.rs`**: Change two `use_effect` blocks (localStorage load, matchMedia poll) from `#[cfg(feature = "web")]` to `#[cfg(not(feature = "server"))]`; replace `video_js::getSetting` call with a direct `document::eval` localStorage read.
- **`packages/web/src/views/theme_toggle.rs`**: Change the preference-save `spawn` block from `#[cfg(feature = "web")]` to `#[cfg(not(feature = "server"))]`; replace `video_js::setSetting` call with a direct `document::eval` localStorage write.
- No new dependencies, no API changes, no breaking changes.
