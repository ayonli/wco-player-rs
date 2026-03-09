## 1. Fix `apply_theme_to_document` in `theme.rs`

- [x] 1.1 Remove the `#[cfg(feature = "web")]` guard from `apply_theme_to_document` and replace with `#[cfg(not(feature = "server"))]`
- [x] 1.2 Delete the `#[cfg(not(feature = "web"))]` no-op stub so only one implementation exists

## 2. Fix theme preference loading in `views/app.rs`

- [x] 2.1 Change the `use_effect` that loads the saved preference from `#[cfg(feature = "web")]` to `#[cfg(not(feature = "server"))]`
- [x] 2.2 Replace the `video_js::getSetting("theme_preference", ...)` call with a `document::eval` + `eval.recv::<serde_json::Value>()` that reads `localStorage.getItem("wco-player-settings")` and parses the JSON to extract `theme_preference`

## 3. Fix system color-scheme detection in `views/app.rs`

- [x] 3.1 Change the `use_effect` that polls `window.matchMedia` from `#[cfg(feature = "web")]` to `#[cfg(not(feature = "server"))]`

## 4. Fix theme preference saving in `views/theme_toggle.rs`

- [x] 4.1 Change the `#[cfg(feature = "web")]` guard on the preference-save `spawn` block to `#[cfg(not(feature = "server"))]`
- [x] 4.2 Replace the `video_js::setSetting("theme_preference", ...)` call with a `document::eval` call that reads `wco-player-settings` from `localStorage`, sets `theme_preference`, and writes it back

## 5. Verify builds and smoke-test

- [x] 5.1 Run `cargo check --package web --features web` and confirm no compile errors
- [x] 5.2 Run `cargo check --package web --features desktop` and confirm no compile errors
- [x] 5.3 Run `cargo check --package web --features server` and confirm no compile errors
- [ ] 5.4 Launch the desktop app and verify toggling the theme button changes the appearance immediately
- [ ] 5.5 Quit and relaunch the desktop app and verify the theme preference is restored
- [ ] 5.6 Launch the web app and verify theme toggle still works (no regression)

