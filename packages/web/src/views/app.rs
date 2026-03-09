use dioxus::prelude::*;

use crate::theme::{apply_theme_to_document, ResolvedTheme, ThemePreference};

const PLAYER_CSS: Asset = asset!("/assets/player.css");
const DX_COMPONENTS_THEME: Asset = asset!("/assets/dx-components-theme.css");

fn compute_resolved(preference: ThemePreference, system_scheme: Option<ResolvedTheme>) -> ResolvedTheme {
    match preference {
        ThemePreference::Light => ResolvedTheme::Light,
        ThemePreference::Dark => ResolvedTheme::Dark,
        ThemePreference::System => system_scheme.unwrap_or(ResolvedTheme::Light),
    }
}

#[component]
pub fn App() -> Element {
    let mut preference = use_signal(|| ThemePreference::System);
    let system_scheme = use_signal(|| None::<ResolvedTheme>);
    let mut resolved = use_signal(|| ResolvedTheme::Light);

    // Load theme preference from storage (all WebView targets, after hydration)
    #[cfg(not(feature = "server"))]
    use_effect(move || {
        spawn(async move {
            let mut eval = document::eval(
                r#"
                (function() {
                    try {
                        var raw = localStorage.getItem('wco-player-settings');
                        if (raw) {
                            var s = JSON.parse(raw);
                            dioxus.send(s.theme_preference || 'system');
                        } else {
                            dioxus.send('system');
                        }
                    } catch(e) {
                        dioxus.send('system');
                    }
                })();
                "#,
            );
            if let Ok(s) = eval.recv::<String>().await {
                preference.set(ThemePreference::from_str(&s));
            }
        });
    });

    // Poll system color scheme (all WebView targets) so resolved theme stays in sync when preference is System
    #[cfg(not(feature = "server"))]
    use_effect(move || {
        let mut system_scheme = system_scheme;
        spawn(async move {
            let mut eval = document::eval(
                r#"
                (function() {
                    function send() {
                        var dark = window.matchMedia('(prefers-color-scheme: dark)').matches;
                        dioxus.send(dark ? 'dark' : 'light');
                    }
                    send();
                    setInterval(send, 1000);
                })();
                "#,
            );
            while let Ok(s) = eval.recv::<String>().await {
                match s.as_str() {
                    "dark" => system_scheme.set(Some(ResolvedTheme::Dark)),
                    "light" => system_scheme.set(Some(ResolvedTheme::Light)),
                    _ => {}
                }
            }
        });
    });

    // Compute resolved theme and apply to document
    use_effect(move || {
        let r = compute_resolved(preference(), system_scheme());
        resolved.set(r);
        apply_theme_to_document(r);
    });

    use_context_provider(|| (preference, resolved));

    rsx! {
        // Load styles (theme first for CSS variables, then player.css to override body styles)
        document::Link { rel: "stylesheet", href: DX_COMPONENTS_THEME }
        document::Link { rel: "stylesheet", href: PLAYER_CSS }

        Router::<crate::route::Route> {}
    }
}
