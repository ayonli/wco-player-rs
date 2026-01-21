# Testing Guide

This document describes the testing strategy for the WCO scraper library.

## Test Structure

The tests are organized into three categories:

### 1. Search Tests (`tests/search_test.rs`)
- `test_search` - Basic search functionality
- `test_search_multiple_keywords` - Search with various keywords

### 2. List Tests (`tests/list_test.rs`)
- `test_list_episodes` - List episodes for a series

### 3. Detail Tests (`tests/detail_test.rs`)
- `test_family_guy` - Full workflow test with Family Guy
- `test_futurama` - Test with HD/Full HD URLs
- `test_the_simpsons` - Test video info structure
- `test_rick_and_morty` - Test with Rick and Morty
- `test_the_penguins_of_madagascar` - Test with another series
- `test_video_info_structure` - Verify VideoInfo fields

## Running Tests

### Run All Tests
⚠️ **Warning**: These tests make real HTTP requests to WCO servers:
```bash
cargo test -p wco
```

### Run Specific Test
```bash
cargo test -p wco test_search
cargo test -p wco test_family_guy
```

### Run with Debug Output
```bash
cargo test -p wco -- --nocapture
```

### Run Tests Sequentially (slower but avoids rate limiting)
```bash
cargo test -p wco -- --test-threads=1
```

## Test Strategy

### Integration Tests
All tests are integration tests located in the `tests/` directory. They:
- Make real network requests to WCO servers
- Depend on external website availability  
- May take several seconds to complete
- Validate the entire workflow from search to video info extraction

### Why No Unit Tests?
Most library functions require network access and interact with external services, making integration tests more appropriate than isolated unit tests.

## Important Notes

### Video Download URLs
Video download URLs are time-limited and may expire quickly. Tests verify that:
1. URLs are successfully extracted
2. URLs have the correct structure (start with http, etc.)
3. The video info contains all expected fields

Tests do NOT actually download videos or verify video URLs work, as these URLs may expire within seconds or minutes.

### Debug Output
When running with `--nocapture`, you'll see debug output showing:
- Extracted URLs and parameters
- API responses
- Progress through the scraping workflow

This is helpful for debugging when tests fail.

## Adding New Tests

When adding new tests:

1. **Network tests**: All tests require network access
2. **Use descriptive names**: `test_feature_name`
3. **Add assertions**: Verify expected behavior
4. **Handle errors**: Use `.unwrap()` with helpful error messages
5. **Add debug output**: Use `println!()` to show progress

Example:
```rust
#[tokio::test]
async fn test_new_feature() {
    let result = your_function().await.unwrap();
    assert!(!result.is_empty(), "Result should not be empty");
    println!("✅ Got {} items", result.len());
}
```

## Continuous Integration

For CI/CD pipelines:

1. Ensure stable internet connection
2. Consider running tests sequentially to avoid rate limiting:
   ```bash
   cargo test -p wco -- --test-threads=1
   ```

3. Use environment variables for configuration:
   ```bash
   WCO_URL=https://custom.url cargo test -p wco
   ```

4. Consider caching test results or running tests on a schedule rather than every commit

## Troubleshooting

### Tests Fail with Network Errors
- Check internet connection
- Verify WCO site is accessible
- Check if website structure has changed
- Review debug output with `--nocapture`

### Tests Timeout
- Run tests sequentially: `cargo test -p wco -- --test-threads=1`
- Increase timeout if using custom test runners

### Rate Limiting
- Run fewer tests at once
- Use `--test-threads=1` to run sequentially
- Add delays between test runs if needed

### Parsing Errors
- Website HTML structure may have changed
- Check debug output to see what was extracted
- May need to update regex patterns or selectors in the library code
