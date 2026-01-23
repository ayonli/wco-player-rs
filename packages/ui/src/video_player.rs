//! Video player component

use dioxus::prelude::*;

/// Video player component
#[component]
pub fn VideoPlayer(
    /// Direct video source URL (proxied URL)
    video_src: Option<String>,
    /// Whether video is loading
    #[props(default = false)]
    loading: bool,
    /// Error message (if any)
    error: Option<String>,
    /// Current selected quality
    #[props(default = "hd".to_string())]
    current_quality: String,
) -> Element {
    // Use current_quality as part of the key to force video reload when quality changes
    let video_key = format!("{}-{}", video_src.as_deref().unwrap_or(""), current_quality);
    let video_id = format!("video-player-{}", video_key);

    rsx! {
        div { class: "video-player-container",

            // Video area
            div { class: "video-player-area",

                if loading {
                    div { class: "video-loading",
                        div { class: "spinner large", "" }
                        p { "Loading video..." }
                    }
                } else if let Some(ref err) = error {
                    div { class: "video-error",
                        p { class: "error-icon", "⚠️" }
                        p { "{err}" }
                    }
                } else if let Some(ref src) = video_src {
                    video {
                        key: "{video_key}",
                        id: "{video_id}",
                        class: "video-element",
                        src: "{src}",
                        controls: true,
                        autoplay: true,
                    }
                } else {
                    div { class: "video-placeholder",
                        p { class: "placeholder-icon", "🎬" }
                        p { "Select an episode to play" }
                    }
                }
            }
        }
    }
}
