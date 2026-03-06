## 1. Theme state and document application

- [x] 1.1 Add app-level theme state (preference: light | dark | system; resolved: light | dark) and resolve "system" from prefers-color-scheme
- [x] 1.2 Apply resolved theme to document root (e.g. set data-theme on document.documentElement) and react to preference/resolution changes
- [x] 1.3 Subscribe to prefers-color-scheme changes when preference is "system" and update resolved theme

## 2. CSS theme variables

- [x] 2.1 Add dark theme variable definitions under [data-theme="dark"] in player.css (or theme layer) for --bg-color, --text-color, --border-color, etc.
- [x] 2.2 Ensure dx-components-theme and player.css work together (same data-theme contract or variable names) so all UI respects theme

## 3. Persistence

- [x] 3.1 Web: read theme preference from localStorage (e.g. key "theme-preference") after hydration in use_effect and write on preference change
- [x] 3.2 Default to "system" (or "light") when no stored preference exists

## 4. Theme toggle UI

- [x] 4.1 Create a theme toggle component (e.g. icon button or dropdown) that cycles or selects Light / Dark / System and updates app theme state
- [x] 4.2 Place the theme control in a visible, consistent location (e.g. player header and search header) so it is available on main screens

## 5. Optional: avoid FOUC on web

- [ ] 5.1 If needed, add a small inline script in web index.html that reads localStorage and sets data-theme on document root before first paint to prevent flash of wrong theme
