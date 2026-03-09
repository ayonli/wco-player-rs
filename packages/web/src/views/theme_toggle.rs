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
        #[cfg(not(feature = "server"))]
        {
            let theme_str = next.as_str();
            let js = format!(
                r#"
                (function() {{
                    try {{
                        var raw = localStorage.getItem('wco-player-settings');
                        var s = raw ? JSON.parse(raw) : {{}};
                        s.theme_preference = '{}';
                        localStorage.setItem('wco-player-settings', JSON.stringify(s));
                    }} catch(e) {{}}
                }})();
                "#,
                theme_str
            );
            let _ = document::eval(&js);
        }
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
