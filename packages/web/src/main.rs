#[cfg(feature = "server")]
use axum::routing::get;

#[cfg(feature = "server")]
mod streaming;
#[cfg(feature = "server")]
use streaming::streaming_video;

fn main() {
    #[cfg(feature = "server")]
    dioxus::serve(|| async move {
        let router = dioxus::server::router(web::App);
        let new_router: axum::Router = router.route("/streaming", get(streaming_video));
        Ok(new_router)
    });

    #[cfg(not(feature = "server"))]
    dioxus::launch(web::App);
}
