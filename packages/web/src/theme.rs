//! Theme preference state and document application (light / dark / system).

use dioxus::prelude::*;

/// User-selectable theme preference.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemePreference {
    Light,
    Dark,
    System,
}

impl ThemePreference {
    pub fn as_str(self) -> &'static str {
        match self {
            ThemePreference::Light => "light",
            ThemePreference::Dark => "dark",
            ThemePreference::System => "system",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "dark" => ThemePreference::Dark,
            "light" => ThemePreference::Light,
            _ => ThemePreference::System,
        }
    }
}

/// Resolved theme applied to the DOM (light or dark).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResolvedTheme {
    Light,
    Dark,
}

impl ResolvedTheme {
    pub fn as_str(self) -> &'static str {
        match self {
            ResolvedTheme::Light => "light",
            ResolvedTheme::Dark => "dark",
        }
    }
}

/// Apply resolved theme to document root (data-theme attribute).
/// Works on all WebView-capable targets (web, desktop, mobile).
/// Excluded only on the server build which has no DOM.
#[cfg(not(feature = "server"))]
pub fn apply_theme_to_document(resolved: ResolvedTheme) {
    let s = resolved.as_str();
    let _ = document::eval(&format!(
        "document.documentElement.setAttribute('data-theme', \"{}\");",
        s
    ));
}

#[cfg(feature = "server")]
pub fn apply_theme_to_document(_resolved: ResolvedTheme) {}
