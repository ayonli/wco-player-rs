//! Theme toggle: icon reflects current theme (sun in dark mode, moon in light), click toggles to the opposite.

use dioxus::prelude::*;
use dioxus_free_icons::{icons::ld_icons::{LdMoon, LdSun}, Icon};

use crate::theme::{ResolvedTheme, ThemePreference};

/// Theme toggle: show moon when light (click → dark), show sun when dark (click → light).
#[component]
pub fn ThemeToggle() -> Element {
    let (mut preference, resolved) =
        use_context::<(Signal<ThemePreference>, Signal<ResolvedTheme>)>();

    let toggle = move |_| {
        let next = match resolved() {
            ResolvedTheme::Light => ThemePreference::Dark,
            ResolvedTheme::Dark => ThemePreference::Light,
        };
        preference.set(next);
        #[cfg(feature = "web")]
        spawn(async move {
            let _ = crate::video_js::setSetting("theme_preference", next.as_str().to_string()).await;
        });
    };

    let (icon, title) = match resolved() {
        ResolvedTheme::Light => (
            rsx! { Icon { icon: LdMoon, width: Some(20), height: Some(20) } },
            "Switch to dark mode",
        ),
        ResolvedTheme::Dark => (
            rsx! { Icon { icon: LdSun, width: Some(20), height: Some(20) } },
            "Switch to light mode",
        ),
    };

    rsx! {
        button {
            class: "theme-toggle",
            title: "{title}",
            onclick: toggle,
            {icon}
        }
    }
}
