use wco::{search_series, list_episodes};

#[tokio::test]
async fn test_list_episodes() {
    // First search for a series
    let series = search_series("Futurama", None).await;
    assert!(series.is_ok(), "Search should succeed");
    
    let series = series.unwrap();
    assert!(!series.is_empty(), "Search results should not be empty");
    
    // Then list episodes
    let episodes = list_episodes(&series[0].url).await;
    assert!(episodes.is_ok(), "List episodes should succeed");
    
    let episodes = episodes.unwrap();
    assert!(!episodes.is_empty(), "Episodes list should not be empty");
    
    // Verify structure
    let first = &episodes[0];
    assert!(!first.title.is_empty(), "Episode title should not be empty");
    assert!(!first.url.is_empty(), "Episode URL should not be empty");
}
