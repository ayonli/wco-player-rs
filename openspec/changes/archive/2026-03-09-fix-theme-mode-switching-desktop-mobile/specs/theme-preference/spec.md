## MODIFIED Requirements

### Requirement: Theme preference is persisted

The system SHALL persist the user's theme preference so that it is restored on the next visit or application launch. On all platforms (web, desktop, and mobile) the system SHALL use `localStorage` inside the WebView to store the preference, and SHALL restore it at startup via `document::eval`.

#### Scenario: Preference restored on web load

- **WHEN** the user has previously set a theme preference and later loads the web application
- **THEN** the application SHALL apply the stored theme preference (from `localStorage`) before or as part of initial render

#### Scenario: Preference restored on desktop launch

- **WHEN** the user has previously set a theme preference and later launches the desktop application
- **THEN** the application SHALL apply the stored theme preference (from the WebView `localStorage`) at startup

#### Scenario: Preference restored on mobile launch

- **WHEN** the user has previously set a theme preference and later launches the mobile application
- **THEN** the application SHALL apply the stored theme preference (from the WebView `localStorage`) at startup

---

### Requirement: User can set theme preference

The system SHALL allow the user to choose a theme preference of **Light**, **Dark**, or **System**. The system SHALL apply the chosen theme to the entire application UI on all platforms (web, desktop, and mobile) by setting the `data-theme` attribute on the document root element via `document::eval`.

#### Scenario: User selects Light

- **WHEN** the user sets the theme preference to Light
- **THEN** the application SHALL display in light theme (light backgrounds, dark text) across all screens on all platforms

#### Scenario: User selects Dark

- **WHEN** the user sets the theme preference to Dark
- **THEN** the application SHALL display in dark theme (dark backgrounds, light text) across all screens on all platforms

#### Scenario: User selects System

- **WHEN** the user sets the theme preference to System
- **THEN** the application SHALL follow the operating system or browser color scheme (`prefers-color-scheme`) on all platforms and SHALL update automatically when the system preference changes

## ADDED Requirements

### Requirement: Theme application works on desktop and mobile

The system SHALL apply the `data-theme` attribute to the document root on desktop and mobile targets (not only web), using `document::eval`. The `apply_theme_to_document` function SHALL NOT be a no-op on any WebView-capable target; it SHALL be excluded only on the server build (`#[cfg(not(feature = "server"))]`).

#### Scenario: Theme applied on desktop at launch

- **WHEN** the desktop application launches and a theme preference is resolved
- **THEN** the document root SHALL have the correct `data-theme` attribute set and all CSS variables SHALL reflect the active theme

#### Scenario: Theme applied on mobile at launch

- **WHEN** the mobile application launches and a theme preference is resolved
- **THEN** the document root SHALL have the correct `data-theme` attribute set and all CSS variables SHALL reflect the active theme

#### Scenario: Theme updates on desktop after user toggle

- **WHEN** the user toggles the theme on the desktop application
- **THEN** the document root `data-theme` attribute SHALL update immediately and the new preference SHALL be persisted to `localStorage`

#### Scenario: Theme updates on mobile after user toggle

- **WHEN** the user toggles the theme on the mobile application
- **THEN** the document root `data-theme` attribute SHALL update immediately and the new preference SHALL be persisted to `localStorage`

### Requirement: System color scheme detection works on desktop and mobile

The system SHALL detect the OS color scheme via `window.matchMedia('(prefers-color-scheme: dark)')` on desktop and mobile (not only web) and SHALL apply it when the preference is set to System.

#### Scenario: System dark mode detected on desktop

- **WHEN** the desktop application is running with preference set to System and the OS is in dark mode
- **THEN** the application SHALL display in dark theme

#### Scenario: System light mode detected on mobile

- **WHEN** the mobile application is running with preference set to System and the OS is in light mode
- **THEN** the application SHALL display in light theme
