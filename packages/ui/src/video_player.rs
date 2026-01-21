//! Video player component

use crate::{Select, SelectList, SelectOption, SelectTrigger, SelectValue};
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
) -> Element {
    let mut video_key = use_signal(|| 0u32); // Used to force video element refresh
    let video_id = format!("video-player-{}", video_key());

    // Handle quality change
    let handle_quality_change = move |new_quality: Option<String>| {
        if let Some(quality) = new_quality {
            on_quality_change.call(quality.clone());
            video_key.set(video_key() + 1); // Force video reload
        }
    };

    rsx! {
        div { class: "video-player-container",

            // Header with title and quality selector (hidden in fullscreen)
            if (video_src.is_some() || episode_title.is_some()) && !is_fullscreen {
                div { class: "video-player-header",

                    if let Some(ref title) = episode_title {
                        h2 { class: "video-title", "{title}" }
                    }

                    if video_info.is_some() {
                        div { class: "quality-selector",
                            label { "Quality: " }
                            Select::<String> {
                                value: Some(current_quality.clone()),
                                on_value_change: handle_quality_change,
                                SelectTrigger { SelectValue {} }
                                SelectList {
                                    SelectOption::<String> {
                                        value: "sd".to_string(),
                                        text_value: "SD".to_string(),
                                        index: 0usize,
                                        "SD"
                                    }
                                    if has_hd {
                                        SelectOption::<String> {
                                            value: "hd".to_string(),
                                            text_value: "HD".to_string(),
                                            index: 1usize,
                                            "HD"
                                        }
                                    }
                                    if has_fhd {
                                        SelectOption::<String> {
                                            value: "fhd".to_string(),
                                            text_value: "Full HD".to_string(),
                                            index: 2usize,
                                            "Full HD"
                                        }
                                    }
                                }
                            }
                        }
                    }

                }
            }

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
