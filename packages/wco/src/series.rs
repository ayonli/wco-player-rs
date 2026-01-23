use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

use crate::error::{Result, WcoError};
use crate::user_agent::UserAgent;

/// Convert relative thumbnail URL to absolute URL
fn resolve_thumbnail_url(thumbnail_url: &str, base_url: &reqwest::Url) -> String {
    let thumbnail_url = thumbnail_url.trim();
    if thumbnail_url.starts_with("http://") || thumbnail_url.starts_with("https://") {
        // Already absolute URL
        thumbnail_url.to_string()
    } else if thumbnail_url.starts_with("//") {
        // Protocol-relative URL: use the scheme from base_url
        if let Some(scheme) = base_url.scheme().strip_suffix(':') {
            format!("{}:{}", scheme, thumbnail_url)
        } else {
            // Fallback to https if we can't determine scheme
            format!("https:{}", thumbnail_url)
        }
    } else if thumbnail_url.starts_with("/") {
        // Root-relative path: use base_url's origin
        base_url
            .join(thumbnail_url)
            .map(|url| url.to_string())
            .unwrap_or_else(|_| thumbnail_url.to_string())
    } else {
        // Relative path: append to base_url's directory
        base_url
            .join(thumbnail_url)
            .map(|url| url.to_string())
            .unwrap_or_else(|_| thumbnail_url.to_string())
    }
}

/// Extract episodes from HTML document
fn extract_episodes(document: &Html, base_url: &reqwest::Url) -> Vec<Episode> {
    let mut episodes = Vec::new();

    // Try first selector pattern: div.cat-eps
    let items1_selector = Selector::parse("div.cat-eps").unwrap();
    let items1: Vec<_> = document.select(&items1_selector).collect();

    if !items1.is_empty() {
        let anchor_selector = Selector::parse("a").unwrap();

        for item in items1 {
            let anchor = item.select(&anchor_selector).next();
            if anchor.is_none() {
                continue;
            }

            let anchor = anchor.unwrap();
            let episode_title = anchor.text().collect::<String>().trim().to_string();

            if episode_title.is_empty() {
                continue;
            }

            let url = anchor.value().attr("href");
            if url.is_none() {
                continue;
            }

            episodes.push(Episode {
                title: episode_title,
                url: url.unwrap().to_string(),
            });
        }
    } else {
        // Try second selector pattern: div#episodeList > a
        let items2_selector = Selector::parse("div#episodeList > a").unwrap();
        let span_selector = Selector::parse("span").unwrap();
        let items2: Vec<_> = document.select(&items2_selector).collect();

        for anchor in items2 {
            let episode_title = anchor
                .select(&span_selector)
                .next()
                .map(|e| e.text().collect::<String>().trim().to_string());

            if episode_title.is_none() || episode_title.as_ref().unwrap().is_empty() {
                continue;
            }

            let url = anchor.value().attr("href");
            if url.is_none() {
                continue;
            }

            episodes.push(Episode {
                title: episode_title.unwrap(),
                url: url.unwrap().to_string(),
            });
        }
    }

    // Convert relative URLs to absolute and reverse the order (chronological)
    episodes
        .into_iter()
        .map(|mut episode| {
            if let Ok(url) = base_url.join(&episode.url) {
                episode.url = url.to_string();
            }
            episode
        })
        .rev()
        .collect()
}

/// Episode information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Episode {
    pub title: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeriesDetail {
    pub title: String,
    pub url: String,
    pub thumbnail: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub episodes: Vec<Episode>,
}

pub async fn get_series_detail(series_url: &str) -> Result<SeriesDetail> {
    let client = reqwest::Client::new();
    let res = client
        .get(series_url)
        .header("User-Agent", UserAgent::PostMan.as_str())
        .send()
        .await?;

    if !res.status().is_success() {
        return Err(WcoError::Other(format!(
            "Get series detail request failed: {} {}",
            res.status(),
            res.status().canonical_reason().unwrap_or("")
        )));
    }

    let html = res.text().await?;
    let document = Html::parse_document(&html);

    // Extract title from .video-title>h1
    let title_selector = Selector::parse(".video-title>h1").unwrap();
    let title = document
        .select(&title_selector)
        .next()
        .map(|e| e.text().collect::<String>().trim().to_string())
        .ok_or_else(|| WcoError::ParseError("Cannot find title".to_string()))?;

    // Extract thumbnail from #sidebar_cat>img.img5
    let thumbnail_selector = Selector::parse("#sidebar_cat>img.img5").unwrap();
    let base_url =
        reqwest::Url::parse(series_url).map_err(|e| WcoError::ParseError(e.to_string()))?;

    let thumbnail = document
        .select(&thumbnail_selector)
        .next()
        .and_then(|e| e.value().attr("src"))
        .map(|s| resolve_thumbnail_url(s, &base_url));

    // Extract description from #sidebar_cat>p
    let description_selector = Selector::parse("#sidebar_cat>p").unwrap();
    let description = document
        .select(&description_selector)
        .next()
        .map(|e| e.text().collect::<String>().trim().to_string())
        .filter(|s| !s.is_empty());

    // Extract tags from #sidebar_cat>.genre-buton
    let tags_selector = Selector::parse("#sidebar_cat>.genre-buton").unwrap();
    let tags: Vec<String> = document
        .select(&tags_selector)
        .map(|e| e.text().collect::<String>().trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    // Extract episodes
    let episodes = extract_episodes(&document, &base_url);

    Ok(SeriesDetail {
        title,
        url: series_url.to_string(),
        thumbnail,
        description,
        tags,
        episodes,
    })
}
