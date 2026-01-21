use wco::search_series;

#[tokio::test]
async fn test_search() {
    let list = search_series("Futurama", None).await;
    assert!(list.is_ok(), "Search should succeed");
    
    let list = list.unwrap();
    assert!(!list.is_empty(), "Search results should not be empty");
    
    // Verify structure
    let first = &list[0];
    assert!(!first.title.is_empty(), "Title should not be empty");
    assert!(!first.url.is_empty(), "URL should not be empty");
}

#[tokio::test]
async fn test_search_multiple_keywords() {
    let keywords = vec!["Family Guy", "The Simpsons", "Rick and Morty"];
    
    for keyword in keywords {
        let result = search_series(keyword, None).await;
        assert!(result.is_ok(), "Search for '{}' should succeed", keyword);
        
        let series = result.unwrap();
        assert!(!series.is_empty(), "Search results for '{}' should not be empty", keyword);
    }
}
