//! Player page - displays episodes and video player

use crate::ServerPort;
use crate::video_js;
use dioxus::prelude::*;
use ui::{
    AppState, EpisodeList, Select, SelectList, SelectOption, SelectTrigger, SelectValue,
    VideoPlayer,
};
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

#[allow(unused_variables)]
fn build_video_url(video_url: &str, server_port: u16) -> String {
    #[cfg(feature = "desktop")]
    {
        format!(
            "http://127.0.0.1:{}/streaming?url={}",
            server_port,
            urlencoding::encode(video_url)
        )
    }
    #[cfg(not(feature = "desktop"))]
    {
        format!("/streaming?url={}", urlencoding::encode(video_url))
    }
}

/// Player page component
#[component]
fn PlayerRoute(
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

    let ServerPort(server_port) = {
        #[cfg(feature = "desktop")]
        {
            use_context::<ServerPort>()
        }
        #[cfg(not(feature = "desktop"))]
        {
            ServerPort(0)
        }
    };

    // Load auto_play_next setting from localStorage on mount
    let mut auto_play_next_init = auto_play_next;
    use_effect(move || {
        spawn(async move {
            let default_value = serde_json::json!(false);
            let setting_result: Result<serde_json::Value, _> =
                video_js::getSetting("auto_play_next", default_value).await;
            if let Ok(value) = setting_result
                && let Some(auto_play) = value.as_bool()
            {
                auto_play_next_init.set(auto_play);
            }
        });
    });

    // Get router for navigation
    let router = router();

    // Get window service for fullscreen control (desktop only)
    #[cfg(feature = "desktop")]
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
                let _ = video_js::setAutoPlayNext(enabled).await;
            });
        }
    });

    // Update fullscreen state
    #[cfg(feature = "desktop")]
    {
        // Desktop: use window service
        let window_service_clone = window_service.clone();
        use_effect(move || {
            let current_fullscreen = window_service_clone.fullscreen().is_some();
            is_fullscreen.set(current_fullscreen);
        });

        let window_service_poll = window_service.clone();
        let is_fullscreen_poll = is_fullscreen;
        use_effect(move || {
            let window_service_inner = window_service_poll.clone();
            let mut is_fullscreen_inner = is_fullscreen_poll;
            spawn(async move {
                let mut last_state = is_fullscreen_inner();
                loop {
                    // Poll fullscreen state
                    let current_state = window_service_inner.fullscreen().is_some();
                    if current_state != last_state {
                        is_fullscreen_inner.set(current_state);
                        last_state = current_state;
                    }
                    // Sleep to avoid busy loop
                    use std::time::Duration;
                    tokio::time::sleep(Duration::from_millis(200)).await;
                }
            });
        });
    }

    #[cfg(not(feature = "desktop"))]
    {
        // Web: use JavaScript
        let is_fullscreen_poll = is_fullscreen;
        use_effect(move || {
            let mut is_fullscreen_inner = is_fullscreen_poll;
            spawn(async move {
                loop {
                    match video_js::isPlayerPageFullscreen().await {
                        Ok(current_state) => {
                            is_fullscreen_inner.set(current_state);
                        }
                        Err(_) => {
                            // Ignore errors
                        }
                    }
                    // Yield to allow other tasks to run
                    use std::future::ready;
                    ready(()).await;
                }
            });
        });
    }

    // State management is handled in JavaScript
    // JavaScript will restore episode and position automatically via setupPlaybackTracking

    // Load episodes when series changes
    // After loading, restore saved episode and position from localStorage
    let series_for_episodes = series.clone();
    #[allow(unused_mut)]
    let mut selected_episode_restore = selected_episode;
    #[allow(unused_mut)]
    let mut playback_position_restore = playback_position;
    #[allow(unused_mut)]
    let mut video_info_restore = video_info;
    #[allow(unused_mut)]
    let mut video_loading_restore = video_loading;
    #[allow(unused_mut)]
    let mut video_error_restore = video_error;
    #[allow(unused_mut)]
    let mut current_quality_restore = current_quality;
    #[allow(unused_mut)]
    let mut episodes_restore = episodes;
    #[allow(unused_mut)]
    let mut episodes_loading_restore = episodes_loading;

    use_effect(move || {
        let series_url: String = series_for_episodes.url.clone();
        spawn(async move {
            episodes_loading_restore.set(true);

            let result = async {
                #[cfg(feature = "desktop")]
                {
                    wco::list_episodes(&series_url).await
                }
                #[cfg(not(feature = "desktop"))]
                {
                    match api::list_episodes(series_url).await {
                        Ok(episodes) => Ok(episodes),
                        Err(e) => Err(wco::WcoError::Other(e.to_string())),
                    }
                }
            }
            .await;

            match result {
                Ok(eps) => {
                    episodes_restore.set(eps.clone());

                    // Try to restore episode from URL first, then from localStorage
                    let mut playback_position_clone = playback_position_restore;
                    let eps_clone = eps.clone();
                    spawn(async move {
                        use crate::utils::parse_time;
                        use video_js::{getUrlHash, getUrlParam};

                        // First, try to get episode_url from URL query parameters
                        let url_episode_result = getUrlParam("episode_url").await;
                        let episode_url_opt = if let Ok(Some(url_episode)) = url_episode_result {
                            // Also check for playback position in URL hash
                            let hash_result = getUrlHash().await;
                            if let Ok(Some(hash)) = hash_result
                                && let Some(time_seconds) = parse_time(&hash)
                            {
                                playback_position_clone.set(Some(time_seconds));
                            }
                            Some(url_episode)
                        } else {
                            // Fallback to localStorage
                            let state_result = video_js::loadAppState::<Option<AppState>>().await;
                            if let Ok(Some(state)) = state_result
                                && let Some(saved_episode) = state.episode
                                && let Some(saved_series) = state.series
                            {
                                // Restore playback position from localStorage if not in URL
                                let saved_position = if playback_position_clone().is_none() {
                                    state.playback_position
                                } else {
                                    playback_position_clone()
                                };

                                if let Some(position) = saved_position {
                                    playback_position_clone.set(Some(position));
                                }

                                // Update URL and localStorage using unified function
                                use video_js::updateSeriesEpisodeAndPosition;
                                let series_obj = serde_json::json!({
                                    "title": saved_series.title,
                                    "url": saved_series.url,
                                });
                                let episode_obj = serde_json::json!({
                                    "title": saved_episode.title,
                                    "url": saved_episode.url,
                                });
                                let _: Result<(), _> = updateSeriesEpisodeAndPosition(
                                    series_obj,
                                    episode_obj,
                                    saved_position,
                                )
                                .await;

                                Some(saved_episode.url)
                            } else {
                                None
                            }
                        };

                        if let Some(episode_url) = episode_url_opt {
                            // Find matching episode in the loaded list
                            if let Some(episode_to_restore) =
                                eps_clone.iter().find(|ep| ep.url == episode_url)
                            {
                                // Set selected episode
                                selected_episode_restore.set(Some(episode_to_restore.clone()));

                                // Scroll to the restored episode in the list
                                let episode_url_for_scroll = episode_to_restore.url.clone();
                                spawn(async move {
                                    let _ =
                                        video_js::restorePlaybackEpisode(&episode_url_for_scroll)
                                            .await;
                                });

                                // Load video info for the restored episode
                                video_loading_restore.set(true);
                                video_error_restore.set(None);

                                // Save the playback position before spawning the async task
                                let saved_position_for_tracking = playback_position_clone();
                                let episode_url_for_info = episode_to_restore.url.clone();
                                spawn(async move {
                                    let result = async {
                                        #[cfg(feature = "desktop")]
                                        {
                                            wco::get_video_info(&episode_url_for_info, None).await
                                        }
                                        #[cfg(not(feature = "desktop"))]
                                        {
                                            match api::get_video_info(episode_url_for_info).await {
                                                Ok(info) => Ok(info),
                                                Err(e) => Err(wco::WcoError::Other(e.to_string())),
                                            }
                                        }
                                    }
                                    .await;

                                    match result {
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

                                            // Setup playback tracking with saved position to trigger autoplay
                                            // setupPlaybackTracking handles waiting for video element internally
                                            let _ = video_js::setupPlaybackTracking(
                                                "video-player",
                                                saved_position_for_tracking,
                                            )
                                            .await;
                                        }
                                        Err(e) => {
                                            video_error_restore.set(Some(e.to_string()));
                                        }
                                    }
                                    video_loading_restore.set(false);
                                });
                            }
                        }
                    });
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

            // Scroll to the selected episode
            let episode_url_for_scroll = episode_url.clone();
            spawn(async move {
                let _ = video_js::scrollToEpisode(&episode_url_for_scroll).await;
            });

            // Save series and episode to localStorage and update URL
            // Clear playback position when switching episodes
            let series_title = series_for_save.title.clone();
            let series_url = series_for_save.url.clone();
            let episode_title = episode_clone.title.clone();
            let episode_url_for_save = episode_clone.url.clone();
            let episode_url_for_info = episode_url.clone();
            spawn(async move {
                use video_js::updateSeriesEpisodeAndPosition;
                // Clear playback position when switching episodes
                let series_obj = serde_json::json!({
                    "title": series_title,
                    "url": series_url,
                });
                let episode_obj = serde_json::json!({
                    "title": episode_title,
                    "url": episode_url_for_save,
                });
                let _: Result<(), _> = updateSeriesEpisodeAndPosition(
                    series_obj,
                    episode_obj,
                    None, // Clear playback position when switching episodes
                )
                .await;
            });

            spawn(async move {
                let result = async {
                    #[cfg(feature = "desktop")]
                    {
                        wco::get_video_info(&episode_url_for_info, None).await
                    }
                    #[cfg(not(feature = "desktop"))]
                    {
                        match api::get_video_info(episode_url_for_info).await {
                            Ok(info) => Ok(info),
                            Err(e) => Err(wco::WcoError::Other(e.to_string())),
                        }
                    }
                }
                .await;

                match result {
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
        build_video_url(&video_url, server_port)
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
                                let _: Result<(), _> = setLastRoute("/search").await;
                            });
                            // Navigate to search route
                            router.push(crate::route::Route::Search {});
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
                                let _: Result<(), _> = video_js::setSetting("auto_play_next", value_json)
                                    .await;
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

// Player route component - needs to get series from context or URL
#[component]
pub fn Player() -> Element {
    let current_series = use_context::<Signal<Option<Series>>>();
    #[allow(unused_mut)]
    let mut series_from_url = use_signal(|| Option::<Series>::None);
    #[allow(unused_mut)]
    let mut loading_from_url = use_signal(|| false);

    // Try to load series from URL query parameters
    use_effect(move || {
        let mut series_from_url_clone = series_from_url;
        let mut loading_from_url_clone = loading_from_url;
        let mut current_series_clone = current_series;

        // Check if we already have a series from context
        if current_series_clone().is_some() {
            return;
        }

        spawn(async move {
            use crate::utils::parse_time;
            use crate::video_js::loadAppState;
            use crate::video_js::{getUrlHash, getUrlParam};

            // Get series_url and episode_url from URL
            let series_url_result = getUrlParam("series_url").await;
            let hash_result = getUrlHash().await;

            if let Ok(Some(series_url)) = series_url_result {
                loading_from_url_clone.set(true);

                // Create a basic Series object from URL
                // We'll get the title from the episode list page later
                let series = Series {
                    title: "Loading...".to_string(),
                    url: series_url,
                    thumbnail: None,
                };

                // Set series in context so PlayerRoute can use it
                current_series_clone.set(Some(series.clone()));
                series_from_url_clone.set(Some(series));

                // Also restore playback position from hash if available
                if let Ok(Some(hash)) = hash_result
                    && let Some(time_seconds) = parse_time(&hash)
                {
                    // Save to localStorage so PlayerRoute can restore it
                    let state_result = loadAppState::<Option<AppState>>().await;
                    if let Ok(Some(mut state)) = state_result {
                        state.playback_position = Some(time_seconds);
                        use crate::video_js::saveAppState;
                        let _: Result<(), _> = saveAppState(state).await;
                    }
                }

                loading_from_url_clone.set(false);
            }
        });
    });

    // Show loading state while loading from URL
    if loading_from_url() {
        return rsx! {
            div {
                class: "error-page",
                style: "display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100vh;",
                div { class: "spinner large", "" }
                p { "Loading..." }
            }
        };
    }

    match current_series() {
        Some(series) => rsx! {
            PlayerRoute { series }
        },
        None => rsx! {
            div {
                class: "error-page",
                style: "display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100vh;",
                h1 { "No series selected" }
                Link { to: crate::route::Route::Search {}, "Go to Search" }
            }
        },
    }
}
