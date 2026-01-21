//! Player page - displays episodes and video player

use crate::{api_client, ServerConfig};
use dioxus::prelude::*;
use ui::{EpisodeList, Select, SelectList, SelectOption, SelectTrigger, SelectValue, VideoPlayer};
use wco::{Episode, Series, VideoInfo};

/// Get the video URL for a specific quality
fn get_quality_url(info: &VideoInfo, quality: &str) -> String {
    match quality {
        "fhd" | "fullhd" | "1080" | "Full HD" => info
            .full_hd_url
            .clone()
            .or_else(|| info.hd_url.clone())
            .unwrap_or_else(|| info.url.clone()),
        "hd" | "720" | "HD" => info.hd_url.clone().unwrap_or_else(|| info.url.clone()),
        _ => info.url.clone(),
    }
}

/// Build a proxy URL for a video
fn build_video_url(video_url: &str, port: u16) -> String {
    format!(
        "http://127.0.0.1:{}/video?url={}",
        port,
        urlencoding::encode(video_url)
    )
}

/// Player page component
#[component]
pub fn Player(
    /// The selected series to play
    series: Series,
    /// Callback to go back to search
    on_back: EventHandler<()>,
) -> Element {
    let mut episodes = use_signal(Vec::<Episode>::new);
    let mut episodes_loading = use_signal(|| true);
    let mut selected_episode = use_signal(|| Option::<Episode>::None);
    let mut video_info = use_signal(|| Option::<VideoInfo>::None);
    let mut video_loading = use_signal(|| false);
    let mut video_error = use_signal(|| Option::<String>::None);
    let mut current_quality = use_signal(|| "sd".to_string()); // Default to SD

    // Get server config from context
    let server_config = use_context::<ServerConfig>();

    // Get window service for fullscreen control
    let window_service = dioxus::desktop::use_window();

    // Track fullscreen state
    #[allow(unused_mut)]
    let mut is_fullscreen = use_signal(|| false);

    // Initialize video player controls globally (only once)
    use_effect(move || {
        spawn(async move {
            eprintln!("🎬 Initializing video player controls globally...");
            match crate::video_js::init_video_player_controls().await {
                Ok(_) => {
                    eprintln!("✅ Video player controls initialized (Rust side)");
                }
                Err(e) => {
                    eprintln!("❌ Failed to initialize video player controls: {}", e);
                }
            }
        });
    });

    // Update fullscreen state when window fullscreen changes (for macOS native button)
    // Clone window_service for use in closure
    let window_service_clone = window_service.clone();
    // Check initial fullscreen state
    use_effect(move || {
        let current_fullscreen = window_service_clone.fullscreen().is_some();
        is_fullscreen.set(current_fullscreen);
        eprintln!("🎬 Initial fullscreen state: {}", current_fullscreen);
    });

    // Set up periodic check for fullscreen state changes
    // This detects when user clicks macOS native fullscreen button
    let window_service_poll = window_service.clone();
    let is_fullscreen_poll = is_fullscreen;
    use_effect(move || {
        let window_service_inner = window_service_poll.clone();
        let mut is_fullscreen_inner = is_fullscreen_poll;
        spawn(async move {
            let mut last_state = is_fullscreen_inner();
            loop {
                tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                let current_state = window_service_inner.fullscreen().is_some();
                if current_state != last_state {
                    eprintln!(
                        "🔄 Fullscreen state changed: {} -> {}",
                        last_state, current_state
                    );
                    is_fullscreen_inner.set(current_state);
                    last_state = current_state;
                }
            }
        });
    });

    // Load episodes when series changes
    use_effect(move || {
        let series_url: String = series.url.clone();
        spawn(async move {
            episodes_loading.set(true);

            match api_client::list_episodes(&series_url, server_config.port).await {
                Ok(eps) => {
                    episodes.set(eps);
                }
                Err(_e) => {
                    episodes.set(vec![]);
                }
            }

            episodes_loading.set(false);
        });
    });

    // Handle episode selection
    let handle_episode_select = move |episode: Episode| {
        let episode_url = episode.url.clone();
        selected_episode.set(Some(episode));
        video_info.set(None);
        video_error.set(None);
        video_loading.set(true);

        spawn(async move {
            match api_client::get_video_info(&episode_url, server_config.port).await {
                Ok(info) => {
                    // Auto-select best available quality BEFORE setting video_info
                    let best_quality = if info.full_hd_url.is_some() {
                        "fhd"
                    } else if info.hd_url.is_some() {
                        "hd"
                    } else {
                        "sd"
                    };

                    current_quality.set(best_quality.to_string());
                    video_info.set(Some(info));
                }
                Err(e) => {
                    video_error.set(Some(e.to_string()));
                }
            }
            video_loading.set(false);
        });
    };

    // Handle quality change
    let handle_quality_change = {
        let mut current_quality_clone = current_quality;
        move |new_quality: Option<String>| {
            if let Some(quality) = new_quality {
                current_quality_clone.set(quality);
            }
        }
    };

    // Build proxy URL for the video
    let video_src = video_info.read().as_ref().map(|info| {
        let video_url = get_quality_url(info, &current_quality());
        build_video_url(&video_url, server_config.port)
    });

    let selected_url = selected_episode.read().as_ref().map(|e| e.url.clone());
    let episode_title = selected_episode.read().as_ref().map(|e| e.title.clone());

    // Check available qualities
    let has_hd = video_info
        .read()
        .as_ref()
        .map(|i| i.hd_url.is_some())
        .unwrap_or(false);
    let has_fhd = video_info
        .read()
        .as_ref()
        .map(|i| i.full_hd_url.is_some())
        .unwrap_or(false);

    // Determine default quality - use SD if current_quality is empty
    let default_quality = if current_quality().is_empty() {
        "sd".to_string()
    } else {
        current_quality()
    };

    rsx! {
        div {
            class: if is_fullscreen() { "player-page fullscreen-mode" } else { "player-page" },
            id: "player-page",

            // Header with breadcrumb, quality selector, and back button (hidden in fullscreen, shown on hover)
            div {
                class: "player-header fullscreen-overlay",
                id: "player-header",
                div { class: "breadcrumb",
                    span { "Home" }
                    span { " / " }
                    span { class: "current", "{series.title}" }
                    if let Some(ref ep) = selected_episode() {
                        span { " / " }
                        span { class: "current", "{ep.title}" }
                    }
                }
                div { class: "header-controls",
                    if video_info().is_some() {
                        div { class: "quality-selector",
                            label { "Quality: " }
                            Select::<String> {
                                value: Some(default_quality.clone()),
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
                    button {
                        class: "back-button",
                        onclick: move |_| on_back.call(()),
                        "← Back to Search"
                    }
                }
            }

            // Main content: sidebar + player
            div { class: "player-content",
                // Episode list sidebar (hidden in fullscreen, shown on hover)
                div {
                    class: "episode-sidebar-wrapper fullscreen-overlay",
                    id: "episode-sidebar",
                    EpisodeList {
                        episodes: episodes(),
                        selected_url,
                        on_select: handle_episode_select,
                        loading: episodes_loading(),
                    }
                }

                // Video player with quality selector
                VideoPlayer {
                    video_info: video_info(),
                    video_src,
                    episode_title,
                    loading: video_loading(),
                    error: video_error(),
                    has_hd,
                    has_fhd,
                    current_quality: current_quality(),
                    on_quality_change: move |q: String| {
                        current_quality.set(q);
                    },
                    is_fullscreen: is_fullscreen(),
                }
            }
        }
    }
}
