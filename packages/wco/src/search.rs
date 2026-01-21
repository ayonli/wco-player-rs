use crate::{error::Result, user_agent::UserAgent, WcoError};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

/// Series information from search results
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Series {
    pub title: String,
    pub url: String,
    pub thumbnail: Option<String>,
}

/// Search for series by keyword
///
/// # Arguments
/// * `keyword` - Search keyword
/// * `base_url` - Optional base URL (defaults to WCO_URL env var or "https://www.wcoflix.tv")
///
/// # Returns
/// Vector of Series matching the search keyword
pub async fn search_series(keyword: &str, base_url: Option<&str>) -> Result<Vec<Series>> {
    let env_url = std::env::var("WCO_URL").ok();
    let base_url = base_url
        .or(env_url.as_deref())
        .unwrap_or("https://www.wcoflix.tv");

    let client = reqwest::Client::new();
    let form = reqwest::multipart::Form::new()
        .text("catara", keyword.to_string())
        .text("konuara", "series");

    let url = format!("{}/search", base_url);
    let res = client
        .post(&url)
        .multipart(form)
        .header("User-Agent", UserAgent::PostMan.as_str())
        .send()
        .await?;

    if !res.status().is_success() {
        return Err(WcoError::Other(format!(
            "Search request failed: {} {}",
            res.status(),
            res.status().canonical_reason().unwrap_or("")
        )));
    }

    let html = res.text().await?;
    let document = Html::parse_document(&html);

    // Parse the search results
    let items_selector = Selector::parse("ul.items > li").unwrap();
    let title_selector = Selector::parse("div.recent-release-episodes").unwrap();
    let link_selector = Selector::parse("div.recent-release-episodes > a").unwrap();
    let img_selector = Selector::parse("img").unwrap();

    let items: Vec<_> = document.select(&items_selector).collect();

    if items.is_empty() {
        return Err(WcoError::NotFound("No search results found".to_string()));
    }

    let mut results = Vec::new();

    for item in items {
        let title = item
            .select(&title_selector)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string());

        if title.is_none() || title.as_ref().unwrap().is_empty() {
            continue;
        }

        let url = item
            .select(&link_selector)
            .next()
            .and_then(|e| e.value().attr("href"));

        if url.is_none() {
            continue;
        }

        let thumbnail = item
            .select(&img_selector)
            .next()
            .and_then(|e| e.value().attr("src"))
            .map(|s| s.to_string());

        let full_url = if url.unwrap().starts_with("http") {
            url.unwrap().to_string()
        } else {
            format!("{}{}", base_url, url.unwrap())
        };

        results.push(Series {
            title: title.unwrap(),
            url: full_url,
            thumbnail,
        });
    }

    Ok(results)
}
