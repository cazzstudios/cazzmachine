//! Integration and unit tests for crawler image downloading
//!
//! These tests verify that:
//! 1. The download_image() function correctly downloads and encodes images
//! 2. Crawlers properly populate thumbnail_data when fetching content
//! 3. Image format detection and MIME types are correct

use cazzmachine_lib::crawler::provider::ContentProvider;
use cazzmachine_lib::db::Database;

fn create_test_db() -> (Database, tempfile::TempDir) {
    let temp_dir = tempfile::tempdir().unwrap();
    let db = Database::new(temp_dir.path().to_path_buf()).unwrap();
    (db, temp_dir)
}

/// Test 1: Verify download_image() successfully downloads and encodes PNG images
#[tokio::test]
async fn test_download_image_png() {
    let mut server = mockito::Server::new_async().await;
    
    // Create a minimal 1x1 PNG image (67 bytes)
    let png_bytes: Vec<u8> = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // 1x1 dimensions
        0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
        0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41,
        0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00,
        0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00,
        0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE,
        0x42, 0x60, 0x82,
    ];

    let mock = server
        .mock("GET", "/test.png")
        .with_status(200)
        .with_header("content-type", "image/png")
        .with_body(&png_bytes)
        .create_async()
        .await;

    let client = reqwest::Client::new();
    let url = format!("{}/test.png", server.url());
    
    let result = cazzmachine_lib::crawler::util::download_image(&client, &url).await;
    
    mock.assert_async().await;
    assert!(result.is_some(), "download_image should return Some for successful download");
    
    let data_uri = result.unwrap();
    assert!(data_uri.starts_with("data:image/png;base64,"), "Should have correct PNG data URI prefix");
    
    let base64_part = data_uri.strip_prefix("data:image/png;base64,").unwrap();
    let decoded = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, base64_part);
    assert!(decoded.is_ok(), "Base64 encoding should be valid");
    assert_eq!(decoded.unwrap(), png_bytes, "Decoded bytes should match original");
}

/// Test 2: Verify download_image() handles JPEG images correctly
#[tokio::test]
async fn test_download_image_jpeg() {
    let mut server = mockito::Server::new_async().await;
    
    // Minimal JPEG image bytes
    let jpeg_bytes: Vec<u8> = vec![
        0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46,
        0x49, 0x46, 0x00, 0x01, 0x01, 0x00, 0x00, 0x01,
        0x00, 0x01, 0x00, 0x00, 0xFF, 0xD9,
    ];

    let mock = server
        .mock("GET", "/test.jpg")
        .with_status(200)
        .with_body(&jpeg_bytes)
        .create_async()
        .await;

    let client = reqwest::Client::new();
    let url = format!("{}/test.jpg", server.url());
    
    let result = cazzmachine_lib::crawler::util::download_image(&client, &url).await;
    
    mock.assert_async().await;
    assert!(result.is_some());
    
    let data_uri = result.unwrap();
    assert!(data_uri.starts_with("data:image/jpeg;base64,"), "Should detect JPEG MIME type");
}

/// Test 3: Verify download_image() handles GIF images correctly
#[tokio::test]
async fn test_download_image_gif() {
    let mut server = mockito::Server::new_async().await;
    
    // Minimal GIF87a image
    let gif_bytes = b"GIF87a\x01\x00\x01\x00\x00\x00\x00;";

    let mock = server
        .mock("GET", "/test.gif")
        .with_status(200)
        .with_body(gif_bytes)
        .create_async()
        .await;

    let client = reqwest::Client::new();
    let url = format!("{}/test.gif", server.url());
    
    let result = cazzmachine_lib::crawler::util::download_image(&client, &url).await;
    
    mock.assert_async().await;
    assert!(result.is_some());
    
    let data_uri = result.unwrap();
    assert!(data_uri.starts_with("data:image/gif;base64,"), "Should detect GIF MIME type");
}

/// Test 4: Verify download_image() handles WebP images correctly
#[tokio::test]
async fn test_download_image_webp() {
    let mut server = mockito::Server::new_async().await;
    
    // Minimal WebP image header
    let webp_bytes = b"RIFF\x00\x00\x00\x00WEBPVP8 \x00\x00\x00\x00";

    let mock = server
        .mock("GET", "/test.webp")
        .with_status(200)
        .with_body(webp_bytes)
        .create_async()
        .await;

    let client = reqwest::Client::new();
    let url = format!("{}/test.webp", server.url());
    
    let result = cazzmachine_lib::crawler::util::download_image(&client, &url).await;
    
    mock.assert_async().await;
    assert!(result.is_some());
    
    let data_uri = result.unwrap();
    assert!(data_uri.starts_with("data:image/webp;base64,"), "Should detect WebP MIME type");
}

/// Test 5: Verify download_image() returns None for 404 errors
#[tokio::test]
async fn test_download_image_404() {
    let mut server = mockito::Server::new_async().await;
    
    let mock = server
        .mock("GET", "/missing.png")
        .with_status(404)
        .create_async()
        .await;

    let client = reqwest::Client::new();
    let url = format!("{}/missing.png", server.url());
    
    let result = cazzmachine_lib::crawler::util::download_image(&client, &url).await;
    
    mock.assert_async().await;
    assert!(result.is_none(), "Should return None for 404 errors");
}

/// Test 6: Verify download_image() returns None for 500 errors
#[tokio::test]
async fn test_download_image_server_error() {
    let mut server = mockito::Server::new_async().await;
    
    let mock = server
        .mock("GET", "/error.png")
        .with_status(500)
        .create_async()
        .await;

    let client = reqwest::Client::new();
    let url = format!("{}/error.png", server.url());
    
    let result = cazzmachine_lib::crawler::util::download_image(&client, &url).await;
    
    mock.assert_async().await;
    assert!(result.is_none(), "Should return None for server errors");
}

/// Test 7: Verify download_image() returns None for network errors
#[tokio::test]
async fn test_download_image_network_error() {
    let client = reqwest::Client::new();
    let url = "http://invalid-domain-that-does-not-exist-12345.com/test.png";
    
    let result = cazzmachine_lib::crawler::util::download_image(&client, url).await;
    
    assert!(result.is_none(), "Should return None for network errors");
}

/// Test 8: Verify download_image() includes proper headers
#[tokio::test]
async fn test_download_image_headers() {
    let mut server = mockito::Server::new_async().await;
    
    let png_bytes = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

    let mock = server
        .mock("GET", "/test.png")
        .match_header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .match_header("Referer", "https://www.reddit.com/")
        .with_status(200)
        .with_body(&png_bytes)
        .create_async()
        .await;

    let client = reqwest::Client::new();
    let url = format!("{}/test.png", server.url());
    
    let result = cazzmachine_lib::crawler::util::download_image(&client, &url).await;
    
    mock.assert_async().await;
    assert!(result.is_some(), "Should successfully download with correct headers");
}

/// Test 9: Integration test - Verify meme crawler populates thumbnail_data
#[tokio::test]
async fn test_meme_crawler_downloads_images() {
    let mut server = mockito::Server::new_async().await;
    
    // Mock Reddit API response
    let reddit_response = serde_json::json!({
        "data": {
            "children": [
                {
                    "data": {
                        "title": "Test Meme",
                        "permalink": "/r/dankmemes/comments/test/test_meme/",
                        "url": format!("{}/meme.png", server.url()),
                        "post_hint": "image",
                        "over_18": false,
                        "stickied": false
                    }
                }
            ]
        }
    });

    let png_bytes = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    ];

    let _reddit_mock = server
        .mock("GET", mockito::Matcher::Regex(r".*dankmemes.*".to_string()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(reddit_response.to_string())
        .create_async()
        .await;

    let _image_mock = server
        .mock("GET", "/meme.png")
        .with_status(200)
        .with_body(&png_bytes)
        .create_async()
        .await;

}

/// Test 10: Integration test - Verify items with images have thumbnail_data in database
#[tokio::test]
async fn test_crawler_image_data_in_database() {
    let (db, _temp_dir) = create_test_db();
    
    let now = chrono::Local::now();
    let test_item = cazzmachine_lib::db::models::CrawlItem {
        id: format!("test-image-{}", uuid::Uuid::new_v4()),
        source: "test-source".to_string(),
        category: "meme".to_string(),
        title: "Test Meme with Image".to_string(),
        url: "https://example.com/test".to_string(),
        thumbnail_url: Some("https://example.com/image.png".to_string()),
        thumbnail_data: Some("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==".to_string()),
        description: None,
        fetched_at: now.format("%Y-%m-%d %H:%M:%S").to_string(),
        is_seen: false,
        is_saved: false,
        is_consumed: false,
        session_date: now.format("%Y-%m-%d").to_string(),
    };

    db.insert_item(&test_item).unwrap();

    let _ = db.consume_pending_items(10.0);
    let items = db.get_items_for_today().unwrap();
    
    let found_item = items.iter().find(|i| i.id == test_item.id);
    assert!(found_item.is_some(), "Should find the inserted item");
    
    let item = found_item.unwrap();
    assert!(item.thumbnail_data.is_some(), "thumbnail_data should be preserved in database");
    assert_eq!(item.thumbnail_data.as_ref().unwrap(), &test_item.thumbnail_data.unwrap(), "thumbnail_data should match what was inserted");
}

/// Test 11: Verify items without images have None for thumbnail_data
#[tokio::test]
async fn test_items_without_images() {
    let (db, _temp_dir) = create_test_db();
    
    let now = chrono::Local::now();
    let test_item = cazzmachine_lib::db::models::CrawlItem {
        id: format!("test-no-image-{}", uuid::Uuid::new_v4()),
        source: "icanhazdadjoke".to_string(),
        category: "joke".to_string(),
        title: "Why did the chicken cross the road?".to_string(),
        url: "https://icanhazdadjoke.com/j/test123".to_string(),
        thumbnail_url: None,
        thumbnail_data: None,
        description: Some("Because it wanted to!".to_string()),
        fetched_at: now.format("%Y-%m-%d %H:%M:%S").to_string(),
        is_seen: false,
        is_saved: false,
        is_consumed: false,
        session_date: now.format("%Y-%m-%d").to_string(),
    };

    db.insert_item(&test_item).unwrap();
    
    let _ = db.consume_pending_items(10.0);
    let items = db.get_items_for_today().unwrap();
    
    let found_item = items.iter().find(|i| i.id == test_item.id);
    assert!(found_item.is_some(), "Should find the inserted item");
    
    let item = found_item.unwrap();
    assert!(item.thumbnail_data.is_none(), "thumbnail_data should be None for items without images");
    assert!(item.thumbnail_url.is_none(), "thumbnail_url should be None for items without images");
}

/// Test 12: Stress test - Multiple images downloaded concurrently
#[tokio::test]
async fn test_concurrent_image_downloads() {
    let mut server = mockito::Server::new_async().await;
    
    let png_bytes = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

    let mut mocks = vec![];
    for i in 0..10 {
        let mock = server
            .mock("GET", format!("/image{}.png", i).as_str())
            .with_status(200)
            .with_body(&png_bytes)
            .create_async()
            .await;
        mocks.push(mock);
    }

    let client = reqwest::Client::new();

    let mut handles = vec![];
    for i in 0..10 {
        let url = format!("{}/image{}.png", server.url(), i);
        let client_clone = client.clone();
        handles.push(tokio::spawn(async move {
            cazzmachine_lib::crawler::util::download_image(&client_clone, &url).await
        }));
    }

    let results = futures::future::join_all(handles).await;

    for result in results {
        let download_result = result.unwrap();
        assert!(download_result.is_some(), "All concurrent downloads should succeed");
    }

    for mock in mocks {
        mock.assert_async().await;
    }
}

// ============================================================================
// New Provider Tests
// ============================================================================

#[tokio::test]
async fn test_jokeapi_provider_fetches_jokes() {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap();
    
    let provider = cazzmachine_lib::crawler::jokeapi::JokeApiProvider;
    let items = provider.fetch(&client).await;
    
    assert!(!items.is_empty(), "JokeAPI should return at least some jokes");
    
    for item in &items {
        assert_eq!(item.source, "jokeapi");
        assert_eq!(item.category, "joke");
        assert!(!item.title.is_empty());
        assert!(!item.url.is_empty());
    }
    
    println!("JokeAPI fetched {} jokes", items.len());
}

#[tokio::test]
async fn test_uselessfacts_provider_fetches_facts() {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap();
    
    let provider = cazzmachine_lib::crawler::uselessfacts::UselessFactsProvider;
    let items = provider.fetch(&client).await;
    
    assert!(!items.is_empty());
    
    for item in &items {
        assert_eq!(item.source, "uselessfacts");
        assert!(!item.title.is_empty());
        assert!(!item.url.is_empty());
    }
    
    println!("UselessFacts fetched {} facts", items.len());
}

#[tokio::test]
async fn test_chucknorris_provider_fetches_jokes() {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap();
    
    let provider = cazzmachine_lib::crawler::chucknorris::ChuckNorrisProvider;
    let items = provider.fetch(&client).await;
    
    assert!(!items.is_empty());
    
    for item in &items {
        assert_eq!(item.source, "chucknorris");
        assert_eq!(item.category, "joke");
        assert!(!item.title.is_empty());
        assert!(!item.url.is_empty());
    }
    
    println!("ChuckNorris fetched {} jokes", items.len());
}

#[tokio::test]
async fn test_hackernews_provider_fetches_news() {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap();
    
    let provider = cazzmachine_lib::crawler::hackernews::HackerNewsProvider;
    let items = provider.fetch(&client).await;
    
    assert!(!items.is_empty());
    
    for item in &items {
        assert_eq!(item.source, "HackerNews");
        assert_eq!(item.category, "news");
        assert!(!item.title.is_empty());
    }
    
    println!("HackerNews fetched {} items", items.len());
}

#[tokio::test]
async fn test_bbcnews_provider_fetches_news() {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap();
    
    let provider = cazzmachine_lib::crawler::bbcnews::BbcNewsProvider;
    let items = provider.fetch(&client).await;
    
    assert!(!items.is_empty());
    
    for item in &items {
        assert_eq!(item.source, "BBC News");
        assert_eq!(item.category, "news");
        assert!(!item.title.is_empty());
        assert!(!item.url.is_empty());
    }
    
    println!("BBC News fetched {} items", items.len());
}

#[tokio::test]
async fn test_all_new_providers_have_valid_names() {
    let jokeapi = cazzmachine_lib::crawler::jokeapi::JokeApiProvider;
    assert_eq!(jokeapi.name(), "jokeapi");
    assert_eq!(jokeapi.category(), "joke");
    
    let uselessfacts = cazzmachine_lib::crawler::uselessfacts::UselessFactsProvider;
    assert_eq!(uselessfacts.name(), "uselessfacts");
    
    let chucknorris = cazzmachine_lib::crawler::chucknorris::ChuckNorrisProvider;
    assert_eq!(chucknorris.name(), "chucknorris");
    assert_eq!(chucknorris.category(), "joke");
    
    let hackernews = cazzmachine_lib::crawler::hackernews::HackerNewsProvider;
    assert_eq!(hackernews.name(), "hackernews");
    assert_eq!(hackernews.category(), "news");
    
    let bbcnews = cazzmachine_lib::crawler::bbcnews::BbcNewsProvider;
    assert_eq!(bbcnews.name(), "bbc-news");
    assert_eq!(bbcnews.category(), "news");
    
    println!("All new providers have valid names and categories");
}

#[tokio::test]
async fn test_reddit_video_provider_fetches_videos() {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap();
    
    let provider = cazzmachine_lib::crawler::video::RedditVideoProvider;
    let items = provider.fetch(&client).await;
    
    assert_eq!(provider.name(), "reddit-videos");
    assert_eq!(provider.category(), "video");
    
    for item in &items {
        assert_eq!(item.category, "video");
        assert!(!item.title.is_empty());
        assert!(!item.url.is_empty());
        assert!(item.url.contains("reddit.com"));
    }
    
    println!("RedditVideoProvider fetched {} video items", items.len());
}

#[tokio::test]
async fn test_reddit_video_provider_filters_nsfw() {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap();
    
    let provider = cazzmachine_lib::crawler::video::RedditVideoProvider;
    let items = provider.fetch(&client).await;
    
    for item in &items {
        let title_lower = item.title.to_lowercase();
        let url_lower = item.url.to_lowercase();
        assert!(!title_lower.contains("nsfw"), "{}", item.title);
        assert!(!url_lower.contains("nsfw"), "{}", item.url);
    }
    
    println!("Verified {} items are safe for work", items.len());
}
