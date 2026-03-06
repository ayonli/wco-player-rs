//! Player page - displays episodes and video player

use std::fmt;

use crate::ServerPort;
use crate::video_js;
use dioxus::prelude::*;
use dioxus_free_icons::{
    Icon,
    icons::ld_icons::{LdChevronsRight, LdSearch},
};
use serde::Deserialize;
use serde::Serialize;
use crate::views::ThemeToggle;
use ui::{
    AppState, EpisodeList, Select, SelectList, SelectOption, SelectTrigger, SelectValue,
    SeriesDescription, VideoPlayer,
};
use wco::{Episode, SeriesDetail, VideoInfo};

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
    #[cfg(any(feature = "desktop", feature = "mobile"))]
    {
        format!(
            "http://127.0.0.1:{}/streaming?url={}",
            server_port,
            urlencoding::encode(video_url)
        )
    }
    #[cfg(not(any(feature = "desktop", feature = "mobile")))]
    {
        format!("/streaming?url={}", urlencoding::encode(video_url))
    }
}

/// Load series detail (including episodes)
async fn load_series_detail(series_url: &str) -> Result<SeriesDetail, String> {
    #[cfg(any(feature = "desktop", feature = "mobile"))]
    {
        wco::get_series_detail(series_url)
            .await
            .map_err(|e| e.to_string())
    }
    #[cfg(not(any(feature = "desktop", feature = "mobile")))]
    {
        api::get_series_detail(series_url.to_string())
            .await
            .map_err(|e| e.to_string())
    }
}

/// Load video info for an episode
async fn load_video_info(episode_url: &str) -> Result<VideoInfo, String> {
    #[cfg(any(feature = "desktop", feature = "mobile"))]
    {
        wco::get_video_info(episode_url, None)
            .await
            .map_err(|e| e.to_string())
    }
    #[cfg(not(any(feature = "desktop", feature = "mobile")))]
    {
        api::get_video_info(episode_url.to_string())
            .await
            .map_err(|e| e.to_string())
    }
}

/// Get best available quality for video info
fn get_best_quality(info: &VideoInfo) -> String {
    if info.full_hd_url.is_some() {
        "fhd".to_string()
    } else if info.hd_url.is_some() {
        "hd".to_string()
    } else {
        "sd".to_string()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PlayerQuery {
    pub series_url: String,
    pub episode_url: Option<String>,
}

impl From<&str> for PlayerQuery {
    fn from(s: &str) -> Self {
        serde_qs::from_str(s).unwrap()
    }
}

// Dioxus uses Display to generate the URL during navigation
impl fmt::Display for PlayerQuery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match serde_qs::to_string(self) {
            Ok(s) => write!(f, "{}", s),
            Err(_) => Err(fmt::Error),
        }
    }
}

#[component]
pub fn Player(query: PlayerQuery) -> Element {
    let playback_position = use_signal(|| Option::<f64>::None);

    // Get playback position from URL hash (priority) or localStorage
    use_effect(move || {
        let current = playback_position();

        if current.is_none() {
            let mut playback_position_clone = playback_position;
            spawn(async move {
                use crate::utils::parse_time;
                use video_js::{getUrlHash, loadAppState};

                // First try URL hash
                if let Ok(Some(hash)) = getUrlHash().await
                    && let Some(time_seconds) = parse_time(&hash)
                {
                    playback_position_clone.set(Some(time_seconds));
                    return;
                }

                // Then try localStorage
                if let Ok(Some(state)) = loadAppState::<Option<AppState>>().await
                    && let Some(position) = state.playback_position
                {
                    playback_position_clone.set(Some(position));
                    return;
                }

                playback_position_clone.set(Some(0.0));
            });
        }
    });

    // series_url is always provided via query parameter from route
    if query.series_url.is_empty() {
        return rsx! {
            div {
                class: "error-page",
                style: "display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100vh;",
                h1 { "No series selected" }
                Link { to: crate::route::Route::Search {}, "Go to Search" }
            }
        };
    }

    // Wait for playback position to be loaded before rendering PlayerPage
    let position = playback_position();
    if position.is_none() {
        return rsx! {
            div {
                class: "loading-page",
                style: "display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100vh;",
                div { class: "spinner large", "" }
                p { "Loading..." }
            }
        };
    }

    rsx! {
        PlayerPage {
            series_url: query.series_url.clone(),
            episode_url: query.episode_url.clone(),
            playback_position: position.unwrap_or(0.0),
        }
    }
}

/// Player page component
#[component]
fn PlayerPage(
    /// The URL of the series to play
    series_url: String,
    /// Episode URL from query parameter (if provided, skip restore logic)
    episode_url: Option<String>,
    /// Playback position (from URL hash or localStorage)
    playback_position: f64,
) -> Element {
    let series_detail = use_signal(|| Option::<SeriesDetail>::None);
    let series_loading = use_signal(|| true);
    let selected_episode = use_signal(|| Option::<Episode>::None);
    let video_info = use_signal(|| Option::<VideoInfo>::None);
    let video_loading = use_signal(|| false);
    let video_error = use_signal(|| Option::<String>::None);
    #[allow(unused_mut)]
    let mut current_quality = use_signal(|| "sd".to_string());
    #[allow(unused_mut)]
    let mut auto_play_next = use_signal(|| false);
    #[allow(unused_mut)]
    let mut description_expanded = use_signal(|| true);

    let ServerPort(server_port) = {
        #[cfg(any(feature = "desktop", feature = "mobile"))]
        {
            use_context::<ServerPort>()
        }
        #[cfg(not(any(feature = "desktop", feature = "mobile")))]
        {
            ServerPort(0)
        }
    };

    // Initialize video player controls
    use_effect(move || {
        spawn(async move {
            let _ = video_js::initVideoPlayerControls().await;
        });
    });

    // Load auto_play_next setting
    use_effect(move || {
        let mut auto_play_next_clone = auto_play_next;
        spawn(async move {
            let default_value = serde_json::json!(false);
            let setting_result: Result<serde_json::Value, _> =
                video_js::getSetting("auto_play_next", default_value).await;
            if let Ok(value) = setting_result
                && let Some(enabled) = value.as_bool()
            {
                auto_play_next_clone.set(enabled);
            }
        });
    });

    use_fullscreen_change_tracking();

    // Initialize series and episode loading
    use_series_initialization(SeriesInitializationParams {
        series_url: series_url.clone(),
        episode_url: episode_url.clone(),
        playback_position,
        series_detail,
        series_loading,
        selected_episode,
        video_info,
        video_loading,
        video_error,
        current_quality,
    });

    // Handle episode selection
    let handle_episode_select = {
        let series_detail_for_save = series_detail;
        let series_url_for_save = series_url.clone();
        let mut selected_episode_clone = selected_episode;
        let mut video_info_clone = video_info;
        let mut video_loading_clone = video_loading;
        let mut video_error_clone = video_error;
        let mut current_quality_clone = current_quality;

        move |episode: Episode| {
            // Prevent concurrent loading: ignore if already loading
            if video_loading_clone() {
                return;
            }

            // Prevent loading the same episode that's already selected
            if let Some(ref current_episode) = selected_episode_clone() {
                if current_episode.url == episode.url {
                    return;
                }
            }

            selected_episode_clone.set(Some(episode.clone()));
            video_info_clone.set(None);
            video_error_clone.set(None);
            video_loading_clone.set(true);

            // Scroll to episode
            let episode_url = episode.url.clone();
            spawn(async move {
                let _ = video_js::scrollToEpisode(&episode_url).await;
            });

            // Save to localStorage
            let series_title_value = series_detail_for_save()
                .as_ref()
                .map(|d| d.title.clone())
                .unwrap_or_else(|| "Loading...".to_string());
            let series_url = series_url_for_save.clone();
            let episode_title = episode.title.clone();
            let episode_url_for_save = episode.url.clone();
            spawn(async move {
                use video_js::savePlayerState;
                let series_obj = serde_json::json!({
                    "title": series_title_value,
                    "url": series_url,
                });
                let episode_obj = serde_json::json!({
                    "title": episode_title,
                    "url": episode_url_for_save,
                });
                let _: Result<(), _> =
                    savePlayerState(series_obj, episode_obj, Option::<f64>::None).await;
            });

            // Load video info
            let episode_url_for_info = episode.url.clone();
            let selected_episode_for_validation = selected_episode_clone;
            spawn(async move {
                match load_video_info(&episode_url_for_info).await {
                    Ok(info) => {
                        // Verify that the loaded episode is still the selected one
                        if let Some(ref current_episode) = selected_episode_for_validation() {
                            if current_episode.url == episode_url_for_info {
                                // Only update if still the selected episode
                                current_quality_clone.set(get_best_quality(&info));
                                video_info_clone.set(Some(info));
                            }
                            // If episode changed, ignore this result
                        }
                    }
                    Err(e) => {
                        // Verify that the error is for the currently selected episode
                        if let Some(ref current_episode) = selected_episode_for_validation() {
                            if current_episode.url == episode_url_for_info {
                                video_error_clone.set(Some(e));
                            }
                        }
                    }
                }
                video_loading_clone.set(false);
            });
        }
    };

    // Handle quality change
    let mut handle_quality_change = {
        let mut current_quality_clone = current_quality;
        move |new_quality: Option<String>| {
            if let Some(quality) = new_quality {
                current_quality_clone.set(quality);
            }
        }
    };

    // Handle description toggle
    let handle_description_toggle = {
        let mut description_expanded_clone = description_expanded;
        move |_| {
            description_expanded_clone.set(!description_expanded_clone());
        }
    };

    // Handle auto-play next toggle
    let handle_auto_play_next_toggle = {
        let mut auto_play_next_clone = auto_play_next;
        move |_| {
            let new_value = !auto_play_next_clone();
            auto_play_next_clone.set(new_value);
            spawn(async move {
                // Sync to JavaScript immediately
                let _: Result<(), _> = video_js::setAutoPlayNext(new_value).await;
            });
        }
    };

    // Build video URL
    let video_src = video_info.read().as_ref().map(|info| {
        let quality = current_quality();
        let video_url = get_quality_url(info, &quality);
        build_video_url(&video_url, server_port)
    });

    let selected_url = selected_episode.read().as_ref().map(|e| e.url.clone());
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

    let series_title = series_detail()
        .as_ref()
        .map(|d| d.title.clone())
        .unwrap_or_else(|| "Loading...".to_string());
    let episodes = series_detail()
        .as_ref()
        .map(|d| d.episodes.clone())
        .unwrap_or_default();

    rsx! {
        div { class: "player-page", id: "player-page",
            PlayerHeader {
                series_title,
                has_video_info: video_info().is_some(),
                current_quality,
                on_quality_change: move |quality| {
                    handle_quality_change(quality);
                },
                auto_play_next,
                on_auto_play_next_toggle: handle_auto_play_next_toggle,
                has_hd,
                has_fhd,
            }

            div { class: "player-content",
                PlayerSidebar {
                    series_detail,
                    description_expanded,
                    on_description_toggle: handle_description_toggle,
                    episodes,
                    selected_url,
                    on_episode_select: handle_episode_select,
                    loading: series_loading(),
                }

                VideoPlayer {
                    video_src,
                    loading: video_loading(),
                    error: video_error(),
                    current_quality: current_quality(),
                }
            }
        }
    }
}

/// Parameters for series initialization hook
struct SeriesInitializationParams {
    series_url: String,
    episode_url: Option<String>,
    playback_position: f64,
    series_detail: Signal<Option<SeriesDetail>>,
    series_loading: Signal<bool>,
    selected_episode: Signal<Option<Episode>>,
    video_info: Signal<Option<VideoInfo>>,
    video_loading: Signal<bool>,
    video_error: Signal<Option<String>>,
    current_quality: Signal<String>,
}

/// Hook to initialize series loading and auto-play episode if provided
fn use_series_initialization(params: SeriesInitializationParams) {
    let SeriesInitializationParams {
        series_url,
        episode_url,
        playback_position,
        series_detail,
        series_loading,
        selected_episode,
        video_info,
        video_loading,
        video_error,
        current_quality,
    } = params;

    let series_url_for_loading = series_url.clone();
    let episode_url_for_loading = episode_url.clone();
    use_effect(move || {
        let series_url = series_url_for_loading.clone();
        let episode_url = episode_url_for_loading.clone();
        let mut series_detail_clone = series_detail;
        let mut series_loading_clone = series_loading;
        let selected_episode_clone = selected_episode;
        let video_info_clone = video_info;
        let video_loading_clone = video_loading;
        let video_error_clone = video_error;
        let current_quality_clone = current_quality;

        spawn(async move {
            series_loading_clone.set(true);

            let series = match load_series_detail(&series_url).await {
                Ok(series) => series,
                Err(_) => {
                    series_detail_clone.set(None);
                    series_loading_clone.set(false);
                    return;
                }
            };

            let episodes = series.episodes.clone();
            series_detail_clone.set(Some(series.clone()));
            series_loading_clone.set(false);

            // If episode_url is provided from query, select and play it
            if let Some(ref episode_url) = episode_url {
                handle_query_episode(QueryEpisodeParams {
                    episode_url,
                    playback_position,
                    series: &series,
                    episodes: &episodes,
                    selected_episode: selected_episode_clone,
                    video_info: video_info_clone,
                    video_loading: video_loading_clone,
                    video_error: video_error_clone,
                    current_quality: current_quality_clone,
                });
            }
        });
    });
}

/// Parameters for handling query episode selection
struct QueryEpisodeParams<'a> {
    episode_url: &'a str,
    playback_position: f64,
    series: &'a SeriesDetail,
    episodes: &'a [Episode],
    selected_episode: Signal<Option<Episode>>,
    video_info: Signal<Option<VideoInfo>>,
    video_loading: Signal<bool>,
    video_error: Signal<Option<String>>,
    current_quality: Signal<String>,
}

/// Handle episode selection from query parameter
fn handle_query_episode(params: QueryEpisodeParams<'_>) {
    let QueryEpisodeParams {
        episode_url,
        playback_position,
        series,
        episodes,
        selected_episode,
        video_info,
        video_loading,
        video_error,
        current_quality,
    } = params;

    // Find episode from query parameter
    let episode = match episodes.iter().find(|ep| ep.url == *episode_url) {
        Some(ep) => ep,
        None => return,
    };

    let mut selected_episode = selected_episode;
    selected_episode.set(Some(episode.clone()));

    // Save to localStorage (same as manual selection)
    save_episode_state(series, episode, playback_position);

    // Scroll to episode
    scroll_to_episode(&episode.url);

    // Load video info
    load_episode_video_info(LoadVideoInfoParams {
        episode_url: episode.url.clone(),
        playback_position,
        video_info,
        video_loading,
        video_error,
        current_quality,
    });
}

/// Save episode state to localStorage
fn save_episode_state(series: &SeriesDetail, episode: &Episode, playback_position: f64) {
    let series_title = series.title.clone();
    let series_url = series.url.to_string();
    let episode_title = episode.title.clone();
    let episode_url = episode.url.clone();

    spawn(async move {
        use video_js::savePlayerState;
        let series_obj = serde_json::json!({
            "title": series_title,
            "url": series_url,
        });
        let episode_obj = serde_json::json!({
            "title": episode_title,
            "url": episode_url,
        });
        let _: Result<(), _> = savePlayerState(series_obj, episode_obj, playback_position).await;
    });
}

/// Scroll to the selected episode in the episode list
fn scroll_to_episode(episode_url: &str) {
    let episode_url = episode_url.to_string();
    spawn(async move {
        let _ = video_js::restorePlaybackEpisode(&episode_url).await;
    });
}

/// Parameters for loading episode video info
struct LoadVideoInfoParams {
    episode_url: String,
    playback_position: f64,
    video_info: Signal<Option<VideoInfo>>,
    video_loading: Signal<bool>,
    video_error: Signal<Option<String>>,
    current_quality: Signal<String>,
}

/// Load video info for the selected episode
fn load_episode_video_info(params: LoadVideoInfoParams) {
    let LoadVideoInfoParams {
        episode_url,
        playback_position,
        mut video_info,
        mut video_loading,
        mut video_error,
        mut current_quality,
    } = params;

    video_loading.set(true);
    video_error.set(None);

    spawn(async move {
        match load_video_info(&episode_url).await {
            Ok(info) => {
                current_quality.set(get_best_quality(&info));
                video_info.set(Some(info));

                // Setup playback tracking with position
                let _ = video_js::setupPlaybackTracking("video-player", playback_position).await;
            }
            Err(e) => {
                video_error.set(Some(e));
            }
        }
        video_loading.set(false);
    });
}

/// Player header component
#[component]
fn PlayerHeader(
    /// Series title to display
    series_title: String,
    /// Whether video info is available (to show quality selector)
    has_video_info: bool,
    /// Current quality setting
    current_quality: Signal<String>,
    /// Callback when quality changes
    on_quality_change: EventHandler<Option<String>>,
    /// Whether auto-play next is enabled
    auto_play_next: Signal<bool>,
    /// Callback when auto-play next is toggled
    on_auto_play_next_toggle: EventHandler<()>,
    /// Whether HD quality is available
    has_hd: bool,
    /// Whether Full HD quality is available
    has_fhd: bool,
) -> Element {
    let router = router();

    rsx! {
        div { class: "player-header fullscreen-overlay", id: "player-header",
            div { class: "header-left",
                button {
                    class: "back-button",
                    title: "Back to Search",
                    onclick: move |_| {
                        spawn(async move {
                            let _: Result<(), _> = video_js::setLastRoute("/search").await;
                        });
                        router.push(crate::route::Route::Search {});
                    },
                    Icon {
                        icon: LdSearch,
                        width: Some(20),
                        height: Some(20),
                    }
                }
                div {
                    class: "series-title",
                    style: "overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                    "{series_title}"
                }
            }
            div { class: "header-controls",
                if has_video_info {
                    div { class: "quality-selector",
                        label { "Quality: " }
                        Select::<String> {
                            value: {
                                let q = current_quality();
                                Some(if q.is_empty() { "sd".to_string() } else { q })
                            },
                            on_value_change: move |quality| on_quality_change.call(quality),
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
                        on_auto_play_next_toggle.call(());
                    },
                    Icon {
                        icon: LdChevronsRight,
                        width: Some(20),
                        height: Some(20),
                    }
                }
                ThemeToggle {}
            }
        }
    }
}

/// Player sidebar component (contains description and episode list)
#[component]
fn PlayerSidebar(
    /// Series detail data
    series_detail: Signal<Option<SeriesDetail>>,
    /// Whether description is expanded
    description_expanded: Signal<bool>,
    /// Callback when description expand/collapse is toggled
    on_description_toggle: EventHandler<()>,
    /// List of episodes
    episodes: Vec<Episode>,
    /// Currently selected episode URL
    selected_url: Option<String>,
    /// Callback when an episode is selected
    on_episode_select: EventHandler<Episode>,
    /// Whether episodes are loading
    loading: bool,
) -> Element {
    rsx! {
        div {
            class: "episode-sidebar-wrapper fullscreen-overlay",
            id: "episode-sidebar",
            if let Some(detail) = series_detail().as_ref() {
                SeriesDescription {
                    detail: detail.clone(),
                    description_expanded,
                    on_toggle: move |_| on_description_toggle.call(()),
                }
            }
            EpisodeList {
                episodes,
                selected_url,
                on_select: move |e| on_episode_select.call(e),
                loading,
            }
        }
    }
}

fn use_fullscreen_change_tracking() {
    #[allow(unused_mut, unused)]
    let mut is_fullscreen = use_signal(|| false);

    #[cfg(feature = "desktop")]
    {
        use dioxus::desktop::tao::event::{Event as WryEvent, WindowEvent};
        use dioxus::desktop::use_wry_event_handler;
        let window = dioxus::desktop::use_window();

        // Desktop application may be in fullscreen mode on startup
        let window_clone = window.clone();
        use_effect(move || {
            if let Some(_) = window_clone.fullscreen() {
                is_fullscreen.set(true);
                spawn(async move {
                    let _ = video_js::setFullscreenMode(true).await;
                });
            } else {
                is_fullscreen.set(false);
                spawn(async move {
                    let _ = video_js::setFullscreenMode(false).await;
                });
            }
        });

        use_wry_event_handler(move |event, _el| {
            if let WryEvent::WindowEvent { event, .. } = event
                && let WindowEvent::Resized(_) = event
            {
                if let Some(_) = window.fullscreen() {
                    is_fullscreen.set(true);
                    spawn(async move {
                        let _ = video_js::setFullscreenMode(true).await;
                    });
                } else {
                    is_fullscreen.set(false);
                    spawn(async move {
                        let _ = video_js::setFullscreenMode(false).await;
                    });
                }
            }
        });
    }

    #[cfg(feature = "web")]
    {
        #[allow(unused_mut)]
        let mut is_fullscreen_clone = is_fullscreen;
        use_effect(move || {
            let mut is_fullscreen_clone = is_fullscreen_clone;
            spawn(async move {
                let mut eval = document::eval(
                    r#"
                    (async () => {
                        while (true) {
                            await new Promise(resolve => setTimeout(resolve, 200));
                            dioxus.send(window.matchMedia('(display-mode: fullscreen)').matches);
                        }
                    })();
                "#,
                );

                loop {
                    if let Ok(current_state) = eval.recv().await
                        && current_state != is_fullscreen_clone()
                    {
                        is_fullscreen_clone.set(current_state);
                        spawn(async move {
                            let _ = video_js::setFullscreenMode(current_state).await;
                        });
                    }
                }
            });
        });
    }
}
