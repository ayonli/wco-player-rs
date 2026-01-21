/// Basic usage example of the WCO scraper library
///
/// Run with: cargo run --example basic_usage

use wco::{search_series, list_episodes, get_video_info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 WCO Scraper Example\n");
    
    // Step 1: Search for a series
    println!("Searching for 'one piece'...");
    let series = search_series("one piece", None).await?;
    println!("✅ Found {} series\n", series.len());
    
    // Display first 3 results
    for (i, s) in series.iter().take(3).enumerate() {
        println!("{}. {}", i + 1, s.title);
        println!("   URL: {}", s.url);
        if let Some(thumb) = &s.thumbnail {
            println!("   Thumbnail: {}", thumb);
        }
        println!();
    }
    
    // Step 2: List episodes for the first series
    if let Some(first_series) = series.first() {
        println!("\n📺 Listing episodes for: {}", first_series.title);
        println!("─────────────────────────────────────────────\n");
        
        let episodes = list_episodes(&first_series.url).await?;
        println!("✅ Found {} episodes\n", episodes.len());
        
        // Display first 5 episodes
        for (i, episode) in episodes.iter().take(5).enumerate() {
            println!("{}. {}", i + 1, episode.title);
            println!("   URL: {}", episode.url);
        }
        
        // Step 3: Get video info for the first episode
        if let Some(first_episode) = episodes.first() {
            println!("\n\n🎬 Getting video info for: {}", first_episode.title);
            println!("─────────────────────────────────────────────\n");
            
            let video_info = get_video_info(&first_episode.url, None).await?;
            
            println!("✅ Video Information:");
            println!("   Filename: {}", video_info.filename);
            println!("   SD URL: {}", video_info.url);
            
            if let Some(hd_url) = &video_info.hd_url {
                println!("   HD URL: {}", hd_url);
            }
            
            if let Some(fhd_url) = &video_info.full_hd_url {
                println!("   Full HD URL: {}", fhd_url);
            }
            
            println!("\n💡 To download, use: download_video(&video_info, Some(\"downloads\")).await?");
        }
    }
    
    Ok(())
}
