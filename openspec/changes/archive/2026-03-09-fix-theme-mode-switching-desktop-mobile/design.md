## Context

The app is a multi-platform Dioxus project with three runnable targets: **web** (WASM in browser), **desktop** (Dioxus WebView via tao/wry), and **mobile** (Dioxus WebView). All three targets mount the same root component `web::App` and share the same `packages/web` crate.

Theme switching was implemented with the dark-mode feature (`2026-03-07-add-dark-mode`) using the following pattern:

| Concern | Implementation | Feature gate |
|---|---|---|
| Apply `data-theme` attribute to `<html>` | `document::eval(...)` in `apply_theme_to_document` | `#[cfg(feature = "web")]` |
| Load saved preference at startup | `video_js::getSetting(...)` (dioxus-use-js WASM binding) | `#[cfg(feature = "web")]` |
| Detect system color scheme changes | `document::eval(...)` polling `matchMedia` | `#[cfg(feature = "web")]` |
| Save preference on toggle | `video_js::setSetting(...)` (dioxus-use-js WASM binding) | `#[cfg(feature = "web")]` |

Dioxus desktop and mobile are built with `features = ["desktop"]` / `features = ["mobile"]`, **not** `features = ["web"]`. Consequently every `#[cfg(feature = "web")]` block is dead code on those targets, and the non-web stub of `apply_theme_to_document` is an empty function.

Both `dioxus::desktop` and `dioxus::mobile` use **wry** as their WebView backend, which exposes a fully functional browser-grade JS runtime. `document::eval` works there; `localStorage` is persistent across launches; `window.matchMedia` reflects the OS color scheme.

The only target where JS cannot run is the **server** build (SSR/fullstack hydration), which has no DOM.

## Goals / Non-Goals

**Goals:**

- Theme application (`data-theme` attribute) works on desktop and mobile.
- System color-scheme detection works on desktop and mobile.
- User preference is persisted to and restored from `localStorage` on desktop and mobile.
- No regression on web.

**Non-Goals:**

- Native OS preference storage (e.g. macOS `NSUserDefaults`) — `localStorage` inside the WebView is sufficient and consistent with web.
- Theming for the server-rendered HTML shell — the server build has no persistent state.

## Decisions

### Decision 1: Use `#[cfg(not(feature = "server"))]` instead of `#[cfg(feature = "web")]`

All runtime JS calls (`document::eval`, `localStorage`) work identically in the WebView on desktop/mobile. The only target that cannot run JS is the server. Therefore the correct compile-time predicate is `not(server)`, not `web`.

**Alternative considered**: Add `#[cfg(any(feature = "web", feature = "desktop", feature = "mobile"))]` — rejected because it requires every new platform to be listed explicitly and is harder to maintain.

### Decision 2: Replace `video_js::getSetting` / `setSetting` with direct `document::eval` for persistence

`dioxus-use-js` generates WASM-bindgen bindings under the `web` feature. On desktop/mobile, these functions are either not compiled or call into the server RPC path. Using `document::eval` with a small inline JS snippet (`localStorage.getItem` / `localStorage.setItem`) works identically to the existing TypeScript helpers — it is the same `wco-player-settings` key and JSON schema — so behaviour on web is unchanged while desktop/mobile gain the same capability.

**Alternative considered**: Move `getSetting`/`setSetting` calls behind a trait abstraction with platform-specific implementations — rejected as over-engineering for a 3-line localStorage read/write.

### Decision 3: Keep the single `wco-player-settings` key + JSON shape

The existing `state_manager.ts` persists settings as `{"theme_preference":"dark"}` etc. under the key `wco-player-settings`. The inline JS eval will read/write the same key so web and desktop/mobile share the same storage format and no migration is needed.

## Risks / Trade-offs

- **`document::eval` is fire-and-forget on server builds**: The guard `#[cfg(not(feature = "server"))]` fully prevents eval calls from reaching the server path, so there is no runtime risk.
- **WebView `localStorage` isolation**: Each desktop/mobile app instance has its own WebView data directory, so stored preferences do not leak between users or profiles. This is the expected behaviour.
- **`eval` error handling**: `document::eval` returns a `Result`; the implementation discards errors with `let _ = ...` (same pattern already used in the web-only code). If the WebView fails to execute the snippet the worst case is the preference reverts to `System` on next launch.

## Migration Plan

This is a pure code change with no data migration:

1. Edit `packages/web/src/theme.rs` — replace the two `cfg`-gated versions of `apply_theme_to_document` with one version gated on `#[cfg(not(feature = "server"))]`.
2. Edit `packages/web/src/views/app.rs` — change both `use_effect` blocks from `#[cfg(feature = "web")]` to `#[cfg(not(feature = "server"))]`; replace `video_js::getSetting` with an `eval` + `recv` call reading `localStorage`.
3. Edit `packages/web/src/views/theme_toggle.rs` — change the save `spawn` block from `#[cfg(feature = "web")]` to `#[cfg(not(feature = "server"))]`; replace `video_js::setSetting` with an `eval` call writing to `localStorage`.
4. Build and smoke-test all three targets.
5. Rollback: git revert — no state to migrate.

## Open Questions

- None. The approach is straightforward given the shared WebView runtime across all three targets.
