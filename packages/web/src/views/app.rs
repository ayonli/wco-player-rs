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

    // Load theme preference from storage (web, after hydration)
    #[cfg(feature = "web")]
    use_effect(move || {
        spawn(async move {
            let default_value = serde_json::json!("system");
            let result = crate::video_js::getSetting("theme_preference", default_value).await;
            if let Ok(serde_json::Value::String(s)) = result {
                preference.set(ThemePreference::from_str(&s));
            }
        });
    });

    // Poll system color scheme (web) so resolved theme stays in sync when preference is System
    #[cfg(feature = "web")]
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
