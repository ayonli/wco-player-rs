use crate::{error::Result, user_agent::UserAgent, WcoError};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::io::AsyncWriteExt;

/// Video information including URLs and filename
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

    let capture = re
        .find(&html)
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
        .or(env_embed_url.as_deref())
        .unwrap_or("https://embed.wcostream.com/inc/embed");

    let player_url = format!(
        "{}/video-js.php?{}",
        embed_origin,
        url.query().unwrap_or("")
    );

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
        return Err(WcoError::Other(format!(
            "Unable to get video link info: {}",
            text
        )));
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
                get_vid_link = Some(
                    found
                        .as_str()
                        .trim_matches('"')
                        .trim_matches('\'')
                        .to_string(),
                );
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
        return Err(WcoError::Other(format!(
            "Unable to get video link info: {}",
            text
        )));
    }

    let link_info: LinkInfo = res3.json().await?;

    // Filter out empty strings - only include quality if it has a non-empty value
    let hd_url = link_info
        .hd
        .as_ref()
        .filter(|s| !s.is_empty())
        .map(|hd| format!("{}/getvid?evid={}", link_info.cdn, hd));

    let full_hd_url = link_info
        .fhd
        .as_ref()
        .filter(|s| !s.is_empty())
        .map(|fhd| format!("{}/getvid?evid={}", link_info.cdn, fhd));

    Ok(VideoInfo {
        url: format!("{}/getvid?evid={}", link_info.cdn, link_info.enc),
        hd_url,
        full_hd_url,
        filename: format!("{}.mp4", file_name),
    })
}

/// Fetch video stream as a Response
///
/// This function matches the TypeScript version's API, accepting:
/// - URL to fetch
/// - Optional additional headers to merge with defaults
///
/// # Arguments
/// * `url` - Video URL to fetch
/// * `extra_headers` - Optional additional headers to include in the request
///
/// # Returns
/// reqwest::Response containing the video stream with full status and headers preserved
///
/// # Examples
/// ```no_run
/// use wco::fetch_video;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Simple fetch
/// let response = fetch_video("https://example.com/video.mp4", None).await?;
///
/// // With additional headers (e.g., Range)
/// let mut headers = reqwest::header::HeaderMap::new();
/// headers.insert("Range", "bytes=0-1023".parse()?);
/// let response = fetch_video("https://example.com/video.mp4", Some(&headers)).await?;
/// # Ok(())
/// # }
/// ```
pub async fn fetch_video(
    url: &str,
    extra_headers: Option<&reqwest::header::HeaderMap>,
) -> Result<reqwest::Response> {
    let client = reqwest::Client::new();
    let user_agent = UserAgent::GoogleChrome.as_str();
    let referer = "https://embed.wcostream.com/";

    // Step 1: Fetch JSON endpoint to get real stream URL
    let json_url = format!("{}&json", url);
    let json_response = client
        .get(&json_url)
        .header("User-Agent", user_agent)
        .header("Referer", referer)
        .send()
        .await
        .map_err(|e| WcoError::Other(format!("Failed to fetch JSON endpoint: {}", e)))?;

    if !json_response.status().is_success() {
        return Err(WcoError::Other(format!(
            "JSON endpoint returned {}: {}",
            json_response.status(),
            json_response.status().canonical_reason().unwrap_or("")
        )));
    }

    // Get JSON text (may be wrapped in quotes)
    let json_text = json_response
        .text()
        .await
        .map_err(|e| WcoError::Other(format!("Failed to read JSON response: {}", e)))?;

    // The &json endpoint returns a JSON string (the URL itself), not a JSON object
    // Parse it as a JSON string to handle escape sequences like \/
    let real_stream_url: String = serde_json::from_str(&json_text)
        .map_err(|e| WcoError::ParseError(format!("Failed to parse JSON string: {}", e)))?;

    // Build request with required headers
    let mut req = client
        .get(&real_stream_url)
        .header("Accept-Encoding", "identity;q=1, *;q=0")
        .header("Accept-Language", "en-US,en;q=0.9")
        .header("Cache-Control", "no-cache")
        .header("Pragma", "no-cache")
        .header("User-Agent", user_agent)
        .header("Referer", referer);

    // Merge additional headers if provided (like Range for seeking)
    if let Some(headers) = extra_headers {
        for (key, value) in headers.iter() {
            req = req.header(key, value);
        }
    }

    let res = match req.send().await {
        Ok(r) => r,
        Err(e) => {
            return Err(e.into());
        }
    };

    // Check response status
    if !res.status().is_success() && res.status() != reqwest::StatusCode::PARTIAL_CONTENT {
        if res.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(WcoError::NotFound(
                res.status()
                    .canonical_reason()
                    .unwrap_or("Not Found")
                    .to_string(),
            ));
        } else {
            return Err(WcoError::Other(format!(
                "Unable to download video: {} {}",
                res.status(),
                res.status().canonical_reason().unwrap_or("")
            )));
        }
    }

    // Return the full Response so callers can preserve status and headers
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
    let res = fetch_video(&info.url, None).await?;

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
