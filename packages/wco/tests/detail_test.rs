use wco::{get_video_info, list_episodes, search_series};

#[tokio::test]
async fn test_family_guy() {
    let series = search_series("Family Guy", None).await.unwrap();
    assert!(!series.is_empty());

    let episodes = list_episodes(&series[0].url).await.unwrap();
    assert!(!episodes.is_empty());

    let info = get_video_info(&episodes[0].url, None).await.unwrap();

    assert!(!info.url.is_empty());
    assert!(!info.filename.is_empty());
    assert!(info.url.starts_with("http"));
    println!("✅ Family Guy - Got video info: {}", info.filename);
}

#[tokio::test]
async fn test_futurama() {
    let series = search_series("Futurama", None).await.unwrap();
    assert!(!series.is_empty());

    let episodes = list_episodes(&series[0].url).await.unwrap();
    assert!(!episodes.is_empty());

    // Test with last episode
    let last_episode = episodes.last().unwrap();
    let info = get_video_info(&last_episode.url, None).await.unwrap();

    assert!(!info.url.is_empty());
    assert!(!info.filename.is_empty());

    // Verify HD URLs if available
    if let Some(hd_url) = &info.hd_url {
        assert!(hd_url.starts_with("http"));
    }
    if let Some(fhd_url) = &info.full_hd_url {
        assert!(fhd_url.starts_with("http"));
    }

    println!("✅ Futurama - Got video info: {}", info.filename);
}

#[tokio::test]
async fn test_the_simpsons() {
    let series = search_series("The Simpsons", None).await.unwrap();
    assert!(!series.is_empty());

    let episodes = list_episodes(&series[0].url).await.unwrap();
    assert!(!episodes.is_empty());

    let info = get_video_info(&episodes[0].url, None).await.unwrap();

    assert!(!info.url.is_empty());
    assert!(!info.filename.is_empty());
    assert!(info.filename.ends_with(".mp4"));

    println!("✅ The Simpsons - Got video info: {}", info.filename);
}

#[tokio::test]
async fn test_rick_and_morty() {
    let series = search_series("Rick and Morty", None).await.unwrap();
    assert!(!series.is_empty());

    let episodes = list_episodes(&series[0].url).await.unwrap();
    assert!(!episodes.is_empty());

    let info = get_video_info(&episodes[0].url, None).await.unwrap();

    assert!(!info.url.is_empty());
    assert!(!info.filename.is_empty());

    println!("✅ Rick and Morty - Got video info: {}", info.filename);
}

#[tokio::test]
async fn test_the_penguins_of_madagascar() {
    let series = search_series("The Penguins of Madagascar", None)
        .await
        .unwrap();
    assert!(!series.is_empty());

    let episodes = list_episodes(&series[0].url).await.unwrap();
    assert!(!episodes.is_empty());

    let info = get_video_info(&episodes[0].url, None).await.unwrap();

    assert!(!info.url.is_empty());
    assert!(!info.filename.is_empty());

    println!(
        "✅ The Penguins of Madagascar - Got video info: {}",
        info.filename
    );
}

#[tokio::test]
async fn test_video_info_structure() {
    let series = search_series("Futurama", None).await.unwrap();
    let episodes = list_episodes(&series[0].url).await.unwrap();
    let info = get_video_info(&episodes[0].url, None).await.unwrap();

    // Verify all fields
    assert!(!info.url.is_empty(), "SD URL should not be empty");
    assert!(info.url.starts_with("http"), "URL should be valid HTTP(S)");
    assert!(!info.filename.is_empty(), "Filename should not be empty");
    assert!(
        info.filename.ends_with(".mp4"),
        "Filename should end with .mp4"
    );

    // HD and Full HD are optional
    if let Some(hd_url) = &info.hd_url {
        assert!(hd_url.starts_with("http"), "HD URL should be valid HTTP(S)");
    }

    if let Some(fhd_url) = &info.full_hd_url {
        assert!(
            fhd_url.starts_with("http"),
            "Full HD URL should be valid HTTP(S)"
        );
    }

    println!("✅ Video info structure validated");
}
