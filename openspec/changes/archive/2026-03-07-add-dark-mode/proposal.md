## Why

Users expect a dark mode option for reduced eye strain and preference in low-light environments. The app currently follows system preference for dioxus components but the main player UI (player.css) is light-only; there is no user-controlled theme choice or persistence.

## What Changes

- Add a user-selectable theme: **Light**, **Dark**, or **System** (follow OS).
- Persist the chosen theme (via localStorage) so it survives reloads.
- Apply the selected theme consistently across the whole app: player layout, search, headers, cards, and any shared UI.
- Expose a theme toggle (e.g. in header or settings) so users can switch without leaving the page.

## Capabilities

### New Capabilities

- `theme-preference`: User can choose Light, Dark, or System; choice is persisted and applied app-wide. Covers storage, application of theme (CSS variables / class on root), and UI to change the preference.

### Modified Capabilities

- (none)

## Impact

- **packages/web**: Root/app component to read persisted theme and set `data-theme` or class on document/root; theme toggle component; player.css (or new theme layer) with dark variable values; possible use of existing dx-components-theme patterns.
- **packages/ui**: Any shared components that rely on theme (e.g. select) should respect the active theme; may need to align with theme variables.
- **Dependencies**: No new external dependencies required; use existing storage and DOM/CSS mechanisms.
