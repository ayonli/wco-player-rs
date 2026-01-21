use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::io::AsyncWriteExt;
use crate::{error::Result, user_agent::UserAgent, WcoError};

/// Video information including URLs and filename
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoInfo {
    pub url: String,
    pub hd_url: Option<String>,
    pub full_hd_url: Option<String>,
    pub filename: String,
}

/// Link information returned from the API
#[derive(Debug, Deserialize)]
struct LinkInfo {
    #[serde(rename = "cdn")]
    cdn: String,
    #[serde(rename = "enc")]
    enc: String,
    #[serde(rename = "hd")]
    hd: Option<String>,
    #[serde(rename = "fhd")]
    fhd: Option<String>,
}

/// Get video information from an episode URL
///
/// # Arguments
/// * `episode_url` - URL of the episode page
/// * `embed_url` - Optional embed URL (defaults to WCO_EMBED_URL env var or "https://embed.wcostream.com/inc/embed")
///
/// # Returns
/// VideoInfo containing download URLs and filename
pub async fn get_video_info(episode_url: &str, embed_url: Option<&str>) -> Result<VideoInfo> {
    let client = reqwest::Client::new();
    
    // Step 1: Fetch the episode page
    let res1 = client
        .get(episode_url)
        .header("User-Agent", UserAgent::PostMan.as_str())
        .send()
        .await?;
    
    if !res1.status().is_success() {
        return Err(WcoError::Other(format!(
            "Unable to fetch web page: {} {}",
            res1.status(),
            res1.status().canonical_reason().unwrap_or("")
        )));
    }
    
    let html = res1.text().await?;
    
    // Step 2: Extract embed URL from the HTML
    let re = Regex::new(r"/embed/index\.php\?file=([^=]+)\.flv")
        .map_err(|e| WcoError::ParseError(e.to_string()))?;
    
    let capture = re.find(&html)
        .ok_or_else(|| WcoError::ParseError("Cannot extract file name of the video".to_string()))?;
    
    let needle = capture.start();
    
    // Find the full URL by searching for quotes around the match
    let start = html[..needle]
        .rfind('"')
        .ok_or_else(|| WcoError::ParseError("Cannot extract video URL".to_string()))?;
    let end = html[needle..]
        .find('"')
        .ok_or_else(|| WcoError::ParseError("Cannot extract video URL".to_string()))?
        + needle;
    
    let url_str = &html[start + 1..end];
    let url = reqwest::Url::parse(url_str)
        .map_err(|e| WcoError::ParseError(format!("Invalid URL: {}", e)))?;
    
    // Verify the file parameter exists
    let _original_file_name = url
        .query_pairs()
        .find(|(key, _)| key == "file")
        .map(|(_, value)| value.to_string())
        .ok_or_else(|| WcoError::ParseError("Cannot extract file name of the video".to_string()))?;
    
    // Step 3: Fetch the video player page
    let env_embed_url = std::env::var("WCO_EMBED_URL").ok();
    let embed_origin = embed_url
        .or_else(|| env_embed_url.as_deref())
        .unwrap_or("https://embed.wcostream.com/inc/embed");
    
    let player_url = format!("{}/video-js.php?{}", embed_origin, url.query().unwrap_or(""));
    
    let res2 = client
        .get(&player_url)
        .header("User-Agent", UserAgent::GoogleChrome.as_str())
        .header("Accept", "application/json, text/javascript, */*; q=0.01")
        .header("Accept-Language", "en-US,en;q=0.9")
        .header("Referer", url.as_str())
        .send()
        .await?;
    
    if !res2.status().is_success() {
        let text = res2.text().await?;
        return Err(WcoError::Other(format!("Unable to get video link info: {}", text)));
    }
    
    let html2 = res2.text().await?;
    
    // Step 4: Extract getvidlink URL
    // Try different regex patterns to match getvidlink
    let patterns = vec![
        r#""/inc/embed/getvidlink\.php([^"]+)""#,
        r#"'/inc/embed/getvidlink\.php([^']+)'"#,
        r#"/inc/embed/getvidlink\.php\?[^"'\s]+"#,
    ];
    
    let mut get_vid_link = None;
    for pattern in &patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(found) = re.find(&html2) {
                get_vid_link = Some(found.as_str().trim_matches('"').trim_matches('\'').to_string());
                break;
            }
        }
    }
    
    let get_vid_link = get_vid_link
        .ok_or_else(|| WcoError::ParseError("Cannot extract link of the video".to_string()))?;
    
    // The get_vid_link already starts with /inc/embed, but embed_origin also ends with /inc/embed
    // So we need to use just the base URL
    let base_url = if embed_origin.contains("/inc/embed") {
        embed_origin.split("/inc/embed").next().unwrap()
    } else {
        embed_origin
    };
    
    let link_info_url = format!("{}{}", base_url, get_vid_link);
    let link_info_url = reqwest::Url::parse(&link_info_url)
        .map_err(|e| WcoError::ParseError(format!("Invalid link info URL: {}", e)))?;
    
    let file_name = link_info_url
        .query_pairs()
        .find(|(key, _)| key == "v")
        .map(|(_, value)| value.to_string())
        .ok_or_else(|| WcoError::ParseError("Cannot extract file name".to_string()))?;
    
    // Step 5: Fetch video link information
    let res3 = client
        .get(link_info_url.as_str())
        .header("User-Agent", UserAgent::GoogleChrome.as_str())
        .header("Accept", "application/json, text/javascript, */*; q=0.01")
        .header("Accept-Language", "en-US,en;q=0.9")
        .header("Referer", &player_url)
        .header("X-Requested-With", "XMLHttpRequest")
        .send()
        .await?;
    
    if !res3.status().is_success() {
        let text = res3.text().await?;
        return Err(WcoError::Other(format!("Unable to get video link info: {}", text)));
    }
    
    let link_info: LinkInfo = res3.json().await?;
    
    Ok(VideoInfo {
        url: format!("{}/getvid?evid={}", link_info.cdn, link_info.enc),
        hd_url: link_info.hd.as_ref().map(|hd| format!("{}/getvid?evid={}", link_info.cdn, hd)),
        full_hd_url: link_info.fhd.as_ref().map(|fhd| format!("{}/getvid?evid={}", link_info.cdn, fhd)),
        filename: format!("{}.mp4", file_name),
    })
}

/// Fetch video stream as a Response
///
/// # Arguments
/// * `url` - Video URL to fetch
///
/// # Returns
/// reqwest::Response containing the video stream
pub async fn fetch_video(url: &str) -> Result<reqwest::Response> {
    let client = reqwest::Client::new();
    let res = client
        .get(url)
        .header("User-Agent", UserAgent::GoogleChrome.as_str())
        .header("Referer", "https://embed.wcostream.com/")
        .send()
        .await?;
    
    if res.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(WcoError::NotFound("Video not found".to_string()));
    } else if !res.status().is_success() {
        return Err(WcoError::Other(format!(
            "Unable to download video: {} {}",
            res.status(),
            res.status().canonical_reason().unwrap_or("")
        )));
    }
    
    Ok(res)
}

/// Download video to a file
///
/// # Arguments
/// * `info` - VideoInfo containing the URL and filename
/// * `output_dir` - Optional output directory (defaults to current_dir/downloads)
///
/// # Returns
/// Path to the downloaded file
pub async fn download_video(info: &VideoInfo, output_dir: Option<&str>) -> Result<String> {
    let res = fetch_video(&info.url).await?;
    
    let output_dir = output_dir.unwrap_or("downloads");
    tokio::fs::create_dir_all(output_dir).await?;
    
    let decoded_filename = urlencoding::decode(&info.filename)
        .map_err(|e| WcoError::ParseError(format!("Failed to decode filename: {}", e)))?;
    
    let output_path = Path::new(output_dir).join(decoded_filename.as_ref());
    let mut file = tokio::fs::File::create(&output_path).await?;
    
    let mut stream = res.bytes_stream();
    use futures_util::StreamExt;
    
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
    }
    
    file.flush().await?;
    
    Ok(output_path.to_string_lossy().to_string())
}
