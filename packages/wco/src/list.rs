use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use crate::{error::Result, user_agent::UserAgent, WcoError};

/// Episode information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Episode {
    pub title: String,
    pub url: String,
}

/// List all episodes for a series
///
/// # Arguments
/// * `series_url` - URL of the series page
///
/// # Returns
/// Vector of Episodes in the series (in chronological order)
pub async fn list_episodes(series_url: &str) -> Result<Vec<Episode>> {
    let client = reqwest::Client::new();
    let res = client
        .get(series_url)
        .header("User-Agent", UserAgent::PostMan.as_str())
        .send()
        .await?;
    
    if !res.status().is_success() {
        return Err(WcoError::Other(format!(
            "List episodes request failed: {} {}",
            res.status(),
            res.status().canonical_reason().unwrap_or("")
        )));
    }
    
    let html = res.text().await?;
    let document = Html::parse_document(&html);
    
    let mut results = Vec::new();
    
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
            let title = anchor.text().collect::<String>().trim().to_string();
            
            if title.is_empty() {
                continue;
            }
            
            let url = anchor.value().attr("href");
            if url.is_none() {
                continue;
            }
            
            results.push(Episode {
                title,
                url: url.unwrap().to_string(),
            });
        }
    } else {
        // Try second selector pattern: div#episodeList > a
        let items2_selector = Selector::parse("div#episodeList > a").unwrap();
        let span_selector = Selector::parse("span").unwrap();
        let items2: Vec<_> = document.select(&items2_selector).collect();
        
        for anchor in items2 {
            let title = anchor
                .select(&span_selector)
                .next()
                .map(|e| e.text().collect::<String>().trim().to_string());
            
            if title.is_none() || title.as_ref().unwrap().is_empty() {
                continue;
            }
            
            let url = anchor.value().attr("href");
            if url.is_none() {
                continue;
            }
            
            results.push(Episode {
                title: title.unwrap(),
                url: url.unwrap().to_string(),
            });
        }
    }
    
    if results.is_empty() {
        return Err(WcoError::NotFound("No episodes found".to_string()));
    }
    
    // Convert relative URLs to absolute and reverse the order (chronological)
    let base_url = reqwest::Url::parse(series_url)
        .map_err(|e| WcoError::ParseError(e.to_string()))?;
    
    let results: Vec<Episode> = results
        .into_iter()
        .map(|mut episode| {
            if let Ok(url) = base_url.join(&episode.url) {
                episode.url = url.to_string();
            }
            episode
        })
        .rev()
        .collect();
    
    Ok(results)
}
