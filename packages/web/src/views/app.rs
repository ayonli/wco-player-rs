use dioxus::prelude::*;

const PLAYER_CSS: Asset = asset!("/assets/player.css");
const DX_COMPONENTS_THEME: Asset = asset!("/assets/dx-components-theme.css");

#[component]
pub fn App() -> Element {
    rsx! {
        // Load styles (theme first for CSS variables, then player.css to override body styles)
        document::Link { rel: "stylesheet", href: DX_COMPONENTS_THEME }
        document::Link { rel: "stylesheet", href: PLAYER_CSS }

        Router::<crate::route::Route> {}
    }
}
