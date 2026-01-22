//! Video player component

use dioxus::prelude::*;
use wco::VideoInfo;

/// Video player component
#[component]
pub fn VideoPlayer(
    /// Video info (if loaded)
    video_info: Option<VideoInfo>,
    /// Direct video source URL (proxied URL)
    video_src: Option<String>,
    /// Current episode title
    episode_title: Option<String>,
    /// Whether video is loading
    #[props(default = false)]
    loading: bool,
    /// Error message (if any)
    error: Option<String>,
    /// Whether HD quality is available
    #[props(default = false)]
    has_hd: bool,
    /// Whether Full HD quality is available
    #[props(default = false)]
    has_fhd: bool,
    /// Current selected quality
    #[props(default = "hd".to_string())]
    current_quality: String,
    /// Callback when quality changes
    on_quality_change: EventHandler<String>,
    /// Whether the app is in fullscreen mode
    #[props(default = false)]
    is_fullscreen: bool,
    /// Initial playback position to restore (in seconds)
    #[props(default = None)]
    initial_playback_position: Option<f64>,
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
