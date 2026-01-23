use wco::{get_series_detail, search_series};

#[tokio::test]
async fn test_get_series_detail() {
    // First search for a series
    let series = search_series("Futurama", None).await;
    assert!(series.is_ok(), "Search should succeed");

    let series = series.unwrap();
    assert!(!series.is_empty(), "Search results should not be empty");

    // Get series detail
    let detail = get_series_detail(&series[0].url).await;
    assert!(detail.is_ok(), "Get series detail should succeed");

    let detail = detail.unwrap();

    // Verify structure
    assert!(!detail.title.is_empty(), "Title should not be empty");
    assert!(!detail.url.is_empty(), "URL should not be empty");
    assert!(
        detail.url.starts_with("http"),
        "URL should be valid HTTP(S)"
    );

    // Verify episodes
    assert!(!detail.episodes.is_empty(), "Episodes should not be empty");
    let first_episode = &detail.episodes[0];
    assert!(
        !first_episode.title.is_empty(),
        "Episode title should not be empty"
    );
    assert!(
        !first_episode.url.is_empty(),
        "Episode URL should not be empty"
    );
}

#[tokio::test]
async fn test_series_detail_structure() {
    let series = search_series("Family Guy", None).await.unwrap();
    assert!(!series.is_empty());

    let detail = get_series_detail(&series[0].url).await.unwrap();

    // Verify all required fields
    assert!(!detail.title.is_empty(), "Title should not be empty");
    assert!(!detail.url.is_empty(), "URL should not be empty");
    assert!(!detail.episodes.is_empty(), "Episodes should not be empty");

    // Verify optional fields (may or may not be present)
    if let Some(thumbnail) = &detail.thumbnail {
        assert!(
            !thumbnail.is_empty(),
            "Thumbnail should not be empty if present"
        );
        assert!(
            thumbnail.starts_with("http") || thumbnail.starts_with("/"),
            "Thumbnail should be a valid URL or path"
        );
    }

    if let Some(description) = &detail.description {
        assert!(
            !description.is_empty(),
            "Description should not be empty if present"
        );
    }

    // Tags can be empty, but if present should not be empty strings
    for tag in &detail.tags {
        assert!(!tag.is_empty(), "Tag should not be empty");
    }

    println!("✅ Series detail structure validated for: {}", detail.title);
}

#[tokio::test]
async fn test_series_detail_multiple_series() {
    let keywords = vec!["The Simpsons", "Rick and Morty"];

    for keyword in keywords {
        let series = search_series(keyword, None).await.unwrap();
        assert!(
            !series.is_empty(),
            "Search results for '{}' should not be empty",
            keyword
        );

        let detail = get_series_detail(&series[0].url).await;
        assert!(
            detail.is_ok(),
            "Get series detail for '{}' should succeed",
            keyword
        );

        let detail = detail.unwrap();
        assert!(
            !detail.title.is_empty(),
            "Title for '{}' should not be empty",
            keyword
        );
        assert!(
            !detail.episodes.is_empty(),
            "Episodes for '{}' should not be empty",
            keyword
        );

        println!(
            "✅ {} - Got {} episodes",
            detail.title,
            detail.episodes.len()
        );
    }
}
