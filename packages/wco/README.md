# WCO Scraper

A Rust library for scraping anime series and episodes from WCO streaming sites.

This is a port of the original TypeScript library, using `scraper` instead of `cheerio` for HTML parsing.

## Features

- 🔍 Search for anime series
- 📝 List episodes for a series
- 🎬 Get video download links (SD, HD, Full HD)
- ⬇️ Download videos

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
wco = { path = "../wco" }
tokio = { version = "1", features = ["full"] }
```

### Example

```rust
use wco::{search_series, list_episodes, get_video_info, download_video};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Search for a series
    let series = search_series("naruto", None).await?;
    println!("Found {} series", series.len());
    
    if let Some(first_series) = series.first() {
        println!("Series: {}", first_series.title);
        
        // List episodes
        let episodes = list_episodes(&first_series.url).await?;
        println!("Found {} episodes", episodes.len());
        
        if let Some(first_episode) = episodes.first() {
            println!("Episode: {}", first_episode.title);
            
            // Get video info
            let video_info = get_video_info(&first_episode.url, None).await?;
            println!("Video URL: {}", video_info.url);
            
            // Download video (optional)
            // let path = download_video(&video_info, Some("downloads")).await?;
            // println!("Downloaded to: {}", path);
        }
    }
    
    Ok(())
}
```

## Environment Variables

- `WCO_URL` - Base URL for WCO site (default: "https://www.wcoflix.tv")
- `WCO_EMBED_URL` - Embed URL (default: "https://embed.wcostream.com/inc/embed")

## Testing

Run all tests (requires internet connection):
```bash
cargo test -p wco
```

Run a specific test:
```bash
cargo test -p wco test_search
```

Run tests with output:
```bash
cargo test -p wco -- --nocapture
```

**Note**: Tests make real network requests to the WCO site and may take a few seconds to complete. Video download URLs are time-limited, so tests verify URL structure rather than actual downloads.

## API

### `search_series(keyword: &str, base_url: Option<&str>) -> Result<Vec<Series>>`

Search for anime series by keyword.

### `list_episodes(series_url: &str) -> Result<Vec<Episode>>`

List all episodes for a given series URL.

### `get_video_info(episode_url: &str, embed_url: Option<&str>) -> Result<VideoInfo>`

Get video download URLs and metadata for an episode.

### `fetch_video(url: &str) -> Result<reqwest::Response>`

Fetch a video stream as a Response object.

### `download_video(info: &VideoInfo, output_dir: Option<&str>) -> Result<String>`

Download a video to a file. Returns the path to the downloaded file.

## License

This is a port of the original TypeScript library. See the main project for license information.
