use dioxus::prelude::*;
use wco::Series;
use web::route::Route;

#[cfg(feature = "server")]
use axum::routing::get;

#[cfg(feature = "server")]
mod streaming;
#[cfg(feature = "server")]
use streaming::streaming_video;

const PLAYER_CSS: Asset = asset!("/assets/player.css");
const DX_COMPONENTS_THEME: Asset = asset!("/assets/dx-components-theme.css");

fn main() {
    #[cfg(feature = "server")]
    dioxus::serve(|| async move {
        let router = dioxus::server::router(App);
        let new_router: axum::Router = router.route("/streaming", get(streaming_video));
        Ok(new_router)
    });

    #[cfg(not(feature = "server"))]
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    // Current series for player page (stored in context)
    let current_series = use_signal(|| Option::<Series>::None);

    // Provide current series context
    use_context_provider(|| current_series);

    rsx! {
        // Load styles (theme first for CSS variables, then player.css to override body styles)
        document::Link { rel: "stylesheet", href: DX_COMPONENTS_THEME }
        document::Link { rel: "stylesheet", href: PLAYER_CSS }

        Router::<Route> {}
    }
}
