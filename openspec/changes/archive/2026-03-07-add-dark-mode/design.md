## Context

The app is a Dioxus-based wco player with web and desktop targets. Theme today is inconsistent: `dx-components-theme.css` uses `prefers-color-scheme` for dioxus components, while `player.css` uses fixed light variables (--bg-color, --text-color, etc.). There is no stored user preference or UI to change theme. We need a single, user-controlled theme (Light / Dark / System) applied everywhere and persisted.

## Goals / Non-Goals

**Goals:**

- User can select Light, Dark, or System and see the app update immediately.
- Selected theme is persisted per platform (web: localStorage; desktop: config/prefs) and restored on load.
- All UI (player, search, headers, cards, shared components) uses the same theme via CSS variables or a root class/attribute.
- A visible control (toggle or dropdown) lets users change theme without leaving the page.

**Non-Goals:**

- Per-page or per-route theme; theme is global.
- Custom color pickers or user-defined palettes.
- Theming third-party or embedded content (e.g. video player chrome) beyond what we control.

## Decisions

1. **Apply theme via `data-theme` on document root**
   - Set `document.documentElement.setAttribute("data-theme", "light" | "dark")` (or equivalent in Dioxus) so CSS can use `[data-theme="dark"]` selectors. For System, resolve to "light" or "dark" from `prefers-color-scheme` and update when the media query changes.
   - Alternative: class on body. Chosen: attribute on root to keep one source of truth and align with common patterns (e.g. dx-components-theme already uses :root and media).

2. **Single source of theme variables**
   - Extend the existing variable approach: in player.css (or a small theme layer), define light and dark values under `:root` and `[data-theme="dark"]` (and optionally keep `prefers-color-scheme` for System when no override). Reuse or align with dx-components-theme variables where components depend on them.
   - Alternative: separate theme CSS files loaded conditionally. Chosen: one stylesheet with data-theme selectors to avoid FOUC and extra requests.

3. **Where to hold theme state**
   - App-level reactive state (e.g. Signal) holding the resolved theme ("light" | "dark") and the preference ("light" | "dark" | "system"). Root component reads preference from storage on load (after hydration on web), writes preference to storage on change, and applies resolved theme to the document. Theme toggle reads/writes the same state.
   - Alternative: context-only without touching document. Chosen: document attribute is required so CSS and any non-React/Dioxus styles apply; state drives that attribute.

4. **Persistence**
   - Web: localStorage key (e.g. `theme-preference`) with values "light" | "dark" | "system". Read in use_effect after hydration to avoid SSR/hydration mismatch.
   - Desktop: same key in a config file or platform prefs (e.g. via existing config crate or std::fs). TBD in implementation based on current desktop config approach.
   - No sync across devices; purely local.

5. **Theme toggle placement**
   - Header or nav (e.g. player header, search header) so it’s visible on main flows. Can be an icon button that cycles Light → Dark → System or a small dropdown. Exact placement and UX are implementation details.

## Risks / Trade-offs

- **FOUC on first load (web)**: If we read theme from localStorage after first paint, the page may flash light then switch. Mitigation: inject a small inline script in index.html that reads localStorage and sets `data-theme` before first paint, or load a minimal theme script before body; Dioxus hydration constraints may require the script approach.
- **Desktop storage divergence**: Desktop may use a different storage mechanism than web. Mitigation: abstract “theme preference storage” behind a small trait or module so web and desktop each implement it; same key/semantics.
- **dx-components-theme vs player.css**: Two layers of variables could conflict. Mitigation: ensure player.css (or theme layer) either overrides or uses the same variable names and data-theme contract so both layers stay in sync.

## Migration Plan

- No data migration. Default for users with no stored preference: "system" (current behavior where applicable) or "light" if we want a single default.
- Deploy: ship theme toggle and CSS changes behind the same release; no feature flag required unless we want gradual rollout.
- Rollback: revert deploy; existing localStorage/config keys can be ignored by old builds.

## Open Questions

- Confirm desktop config location (e.g. which crate or path holds user prefs) so persistence implementation is consistent.
- Whether to add an inline script in web index.html for instant theme apply to avoid FOUC (depends on Dioxus build and where index is generated).
