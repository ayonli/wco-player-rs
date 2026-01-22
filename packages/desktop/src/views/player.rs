//! Player page - displays episodes and video player

use crate::{api_client, video_js, ServerConfig};
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
) -> Element {
    #[allow(unused_mut)]
    let mut episodes = use_signal(Vec::<Episode>::new);
    #[allow(unused_mut)]
    let mut episodes_loading = use_signal(|| true);
    #[allow(unused_mut)]
    let mut selected_episode = use_signal(|| Option::<Episode>::None);
    #[allow(unused_mut)]
    let mut video_info = use_signal(|| Option::<VideoInfo>::None);
    #[allow(unused_mut)]
    let mut video_loading = use_signal(|| false);
    #[allow(unused_mut)]
    let mut video_error = use_signal(|| Option::<String>::None);
    #[allow(unused_mut)]
    let mut current_quality = use_signal(|| "sd".to_string()); // Default to SD
    #[allow(unused_mut)]
    let mut playback_position = use_signal(|| Option::<f64>::None);
    #[allow(unused_mut)]
    let mut auto_play_next = use_signal(|| false);

    // Load auto_play_next setting from localStorage on mount
    let mut auto_play_next_init = auto_play_next;
    use_effect(move || {
        spawn(async move {
            let default_value = serde_json::json!(false);
            let setting_result: Result<serde_json::Value, _> =
                video_js::getSetting("auto_play_next", default_value).await;
            if let Ok(value) = setting_result {
                if let Some(auto_play) = value.as_bool() {
                    auto_play_next_init.set(auto_play);
                }
            }
        });
    });

    // Get server config from context
    let server_config = use_context::<ServerConfig>();

    // Get router for navigation
    let router = router();

    // Get window service for fullscreen control
    let window_service = dioxus::desktop::use_window();

    // Track fullscreen state
    #[allow(unused_mut)]
    let mut is_fullscreen = use_signal(|| false);

    // Initialize video player controls globally (only once)
    use_effect(move || {
        spawn(async move {
            let _ = video_js::initVideoPlayerControls().await;
        });
    });

    // Sync auto-play-next state to JavaScript when it changes or when video info changes
    let auto_play_next_sync = auto_play_next;
    let video_info_sync = video_info;
    use_effect(move || {
        let enabled = auto_play_next_sync();
        let has_video = video_info_sync().is_some();
        if has_video {
            spawn(async move {
                // Wait a bit for video element to be created
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                let _ = video_js::setAutoPlayNext(enabled).await;
            });
        }
    });

    // Update fullscreen state when window fullscreen changes (for macOS native button)
    // Clone window_service for use in closure
    let window_service_clone = window_service.clone();
    // Check initial fullscreen state
    use_effect(move || {
        let current_fullscreen = window_service_clone.fullscreen().is_some();
        is_fullscreen.set(current_fullscreen);
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
                    is_fullscreen_inner.set(current_state);
                    last_state = current_state;
                }
            }
        });
    });

    // State management is handled in JavaScript
    // JavaScript will restore episode and position automatically via setupPlaybackTracking

    // Load episodes when series changes
    // After loading, restore saved episode and position from localStorage
    let series_for_episodes = series.clone();
    let mut selected_episode_restore = selected_episode;
    let mut playback_position_restore = playback_position;
    let mut video_info_restore = video_info;
    let mut video_loading_restore = video_loading;
    let mut video_error_restore = video_error;
    let mut current_quality_restore = current_quality;
    let mut episodes_restore = episodes;
    let mut episodes_loading_restore = episodes_loading;

    use_effect(move || {
        let series_url: String = series_for_episodes.url.clone();
        let server_config_clone = server_config;
        spawn(async move {
            episodes_loading_restore.set(true);

            match api_client::list_episodes(&series_url, server_config_clone.port).await {
                Ok(eps) => {
                    episodes_restore.set(eps.clone());

                    // After episodes are loaded, try to restore saved episode
                    let state_result = video_js::loadAppState::<Option<crate::AppState>>().await;
                    if let Ok(Some(state)) = state_result {
                        if let Some(saved_episode) = state.episode {
                            // Find matching episode in the loaded list
                            if let Some(episode_to_restore) =
                                eps.iter().find(|ep| ep.url == saved_episode.url)
                            {
                                // Set selected episode
                                selected_episode_restore.set(Some(episode_to_restore.clone()));

                                // Restore playback position if available
                                if let Some(saved_position) = state.playback_position {
                                    playback_position_restore.set(Some(saved_position));
                                }

                                // Scroll to the restored episode in the list
                                let episode_url_for_scroll = episode_to_restore.url.clone();
                                spawn(async move {
                                    // Wait a bit for DOM to update with the selected episode
                                    tokio::time::sleep(tokio::time::Duration::from_millis(100))
                                        .await;
                                    let _ =
                                        video_js::restorePlaybackEpisode(&episode_url_for_scroll)
                                            .await;
                                });

                                // Load video info for the restored episode
                                video_loading_restore.set(true);
                                video_error_restore.set(None);

                                let episode_url_for_info = episode_to_restore.url.clone();
                                let server_config_for_info = server_config_clone;
                                spawn(async move {
                                    match api_client::get_video_info(
                                        &episode_url_for_info,
                                        server_config_for_info.port,
                                    )
                                    .await
                                    {
                                        Ok(info) => {
                                            // Auto-select best available quality
                                            let best_quality = if info.full_hd_url.is_some() {
                                                "fhd"
                                            } else if info.hd_url.is_some() {
                                                "hd"
                                            } else {
                                                "sd"
                                            };
                                            current_quality_restore.set(best_quality.to_string());
                                            video_info_restore.set(Some(info));
                                        }
                                        Err(e) => {
                                            video_error_restore.set(Some(e.to_string()));
                                        }
                                    }
                                    video_loading_restore.set(false);
                                });
                            }
                        }
                    }
                }
                Err(_) => {
                    episodes_restore.set(vec![]);
                }
            }

            episodes_loading_restore.set(false);
        });
    });

    // Handle episode selection
    let series_for_save = series.clone();
    let handle_episode_select = {
        let mut selected_episode_clone = selected_episode;
        let mut playback_position_clone = playback_position;
        let mut video_info_clone = video_info;
        let mut video_loading_clone = video_loading;
        let mut video_error_clone = video_error;
        let mut current_quality_clone = current_quality;
        move |episode: Episode| {
            let episode_url = episode.url.clone();
            let episode_clone = episode.clone();
            selected_episode_clone.set(Some(episode_clone.clone()));
            // Clear playback position when selecting a new episode (unless restoring)
            playback_position_clone.set(None);
            video_info_clone.set(None);
            video_error_clone.set(None);
            video_loading_clone.set(true);

            // Scroll to the selected episode after a short delay to ensure DOM is updated
            let episode_url_for_scroll = episode_url.clone();
            spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                let _ = video_js::scrollToEpisode(&episode_url_for_scroll).await;
            });

            // Save series and episode to localStorage
            let series_title = series_for_save.title.clone();
            let series_url = series_for_save.url.clone();
            let episode_title = episode_clone.title.clone();
            let episode_url_for_save = episode_clone.url.clone();
            let episode_url_for_info = episode_url.clone();
            spawn(async move {
                use video_js::setSeriesAndEpisode;
                let series_obj = serde_json::json!({
                    "title": series_title,
                    "url": series_url,
                });
                let episode_obj = serde_json::json!({
                    "title": episode_title,
                    "url": episode_url_for_save,
                });
                let _: Result<(), _> = setSeriesAndEpisode(series_obj, episode_obj).await;
            });

            spawn(async move {
                match api_client::get_video_info(&episode_url_for_info, server_config.port).await {
                    Ok(info) => {
                        // Auto-select best available quality BEFORE setting video_info
                        let best_quality = if info.full_hd_url.is_some() {
                            "fhd"
                        } else if info.hd_url.is_some() {
                            "hd"
                        } else {
                            "sd"
                        };

                        current_quality_clone.set(best_quality.to_string());
                        video_info_clone.set(Some(info));
                    }
                    Err(e) => {
                        video_error_clone.set(Some(e.to_string()));
                    }
                }
                video_loading_clone.set(false);
            });
        }
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
        let quality = current_quality();
        let video_url = get_quality_url(info, &quality);
        build_video_url(&video_url, server_config.port)
    });

    let selected_url = selected_episode.read().as_ref().map(|e| e.url.clone());
    let episode_title = selected_episode.read().as_ref().map(|e| e.title.clone());

    // Playback tracking is automatically set up by initVideoPlayerControls()
    // which uses MutationObserver to detect when video elements are added to DOM
    // No need to manually call setupPlaybackTracking here

    // State saving is handled entirely in JavaScript
    // No periodic save task needed in Rust

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

    let is_fullscreen_value = is_fullscreen();
    rsx! {
        div {
            class: if is_fullscreen_value { "player-page fullscreen-mode" } else { "player-page" },
            id: "player-page",

            // Header with breadcrumb, quality selector, and back button (hidden in fullscreen, shown on hover)
            div {
                class: "player-header fullscreen-overlay",
                id: "player-header",
                div { class: "header-left",
                    button {
                        class: "back-button",
                        title: "Back to Search",
                        onclick: move |_| {
                            // Update route in localStorage before navigating
                            spawn(async move {
                                use crate::video_js::setLastRoute;
                                let _: Result<(), _> = setLastRoute("/search".to_string()).await;
                            });
                            router.push(crate::Route::Search {});
                        },
                        "←"
                    }
                    div { class: "breadcrumb",
                        span { "Home" }
                        span { " / " }
                        span { class: "current", "{series.title}" }
                        if let Some(ref ep) = selected_episode() {
                            span { " / " }
                            span { class: "current", "{ep.title}" }
                        }
                    }
                }
                div { class: "header-controls",
                    if video_info().is_some() {
                        div { class: "quality-selector",
                            label { "Quality: " }
                            Select::<String> {
                                value: {
                                    let q = current_quality();
                                    Some(if q.is_empty() { "sd".to_string() } else { q })
                                },
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
                        class: if auto_play_next() { "auto-play-next-btn active" } else { "auto-play-next-btn" },
                        title: "自动播放下一集",
                        onclick: move |_| {
                            let new_value = !auto_play_next();
                            auto_play_next.set(new_value);
                            // Save to localStorage
                            let value_json = serde_json::json!(new_value);
                            spawn(async move {
                                let _: Result<(), _> = video_js::setSetting("auto_play_next", value_json).await;
                            });
                        },
                        "⏭"
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
                    initial_playback_position: playback_position(),
                }
            }
        }
    }
}
