## ADDED Requirements

### Requirement: User can set theme preference

The system SHALL allow the user to choose a theme preference of **Light**, **Dark**, or **System**. The system SHALL apply the chosen theme to the entire application UI (player, search, headers, cards, and shared components).

#### Scenario: User selects Light

- **WHEN** the user sets the theme preference to Light
- **THEN** the application SHALL display in light theme (light backgrounds, dark text) across all screens

#### Scenario: User selects Dark

- **WHEN** the user sets the theme preference to Dark
- **THEN** the application SHALL display in dark theme (dark backgrounds, light text) across all screens

#### Scenario: User selects System

- **WHEN** the user sets the theme preference to System
- **THEN** the application SHALL follow the operating system or browser color scheme (prefers-color-scheme) and SHALL update automatically when the system preference changes

---

### Requirement: Theme preference is persisted

The system SHALL persist the user's theme preference so that it is restored on the next visit or application launch.

#### Scenario: Preference restored on web load

- **WHEN** the user has previously set a theme preference and later loads the web application
- **THEN** the application SHALL apply the stored theme preference (e.g. from localStorage) before or as part of initial render

#### Scenario: Preference restored on desktop launch

- **WHEN** the user has previously set a theme preference and later launches the desktop application
- **THEN** the application SHALL apply the stored theme preference (e.g. from config or platform prefs) at startup

---

### Requirement: User can change theme from the UI

The system SHALL provide a visible control (e.g. toggle or dropdown in header or navigation) that allows the user to change the theme preference without leaving the current page.

#### Scenario: Changing theme from control

- **WHEN** the user activates the theme control and selects a different option (Light, Dark, or System)
- **THEN** the application SHALL update the visible theme immediately and SHALL persist the new preference

#### Scenario: Theme control is discoverable

- **WHEN** the user is on a main screen (e.g. search or player)
- **THEN** the theme control SHALL be present in a consistent, visible location (e.g. header or nav)
