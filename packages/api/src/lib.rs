use dioxus::prelude::*;

#[post("/api/search")]
pub async fn search_series(keyword: String) -> Result<Vec<wco::Series>, ServerFnError> {
    match wco::search_series(&keyword, None).await {
        Ok(results) => Ok(results),
        Err(e) => Err(ServerFnError::ServerError {
            message: e.to_string(),
            code: 500,
            details: None,
        }),
    }
}

#[post("/api/episodes")]
pub async fn list_episodes(series_url: String) -> Result<Vec<wco::Episode>, ServerFnError> {
    match wco::list_episodes(&series_url).await {
        Ok(episodes) => Ok(episodes),
        Err(e) => Err(ServerFnError::ServerError {
            message: e.to_string(),
            code: 500,
            details: None,
        }),
    }
}

#[post("/api/video_info")]
pub async fn get_video_info(episode_url: String) -> Result<wco::VideoInfo, ServerFnError> {
    match wco::get_video_info(&episode_url, None).await {
        Ok(video_info) => Ok(video_info),
        Err(e) => Err(ServerFnError::ServerError {
            message: e.to_string(),
            code: 500,
            details: None,
        }),
    }
}
