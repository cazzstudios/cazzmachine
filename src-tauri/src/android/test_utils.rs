#[cfg(test)]
pub mod mocks {
    use mockito::{Server, Mock};
    use reqwest::Client;
    use std::sync::Arc;
    use tokio::sync::watch;
    
    pub struct MockServer {
        pub server: Server,
        pub url: String,
    }
    
    impl MockServer {
        pub async fn new() -> Self {
            let server = Server::new_async().await;
            let url = server.url();
            Self { server, url }
        }
        
        pub fn create_reddit_memes_mock(&mut self) -> Mock {
            let response = serde_json::json!({
                "data": {
                    "children": [
                        {
                            "data": {
                                "id": "test123",
                                "title": "Test Meme",
                                "url": "https://example.com/meme.jpg",
                                "permalink": "/r/memes/comments/test123",
                                "subreddit": "memes"
                            }
                        }
                    ]
                }
            });
            
            self.server
                .mock("GET", "/r/memes/new.json")
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body(response.to_string())
                .create()
        }
        
        pub fn create_dadjoke_mock(&mut self) -> Mock {
            let response = serde_json::json!({
                "id": "test-joke-123",
                "joke": "Why do programmers hate nature? It has too many bugs.",
                "status": 200
            });
            
            self.server
                .mock("GET", "/")
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body(response.to_string())
                .create()
        }
        
        pub fn create_uselessfacts_mock(&mut self) -> Mock {
            let response = serde_json::json!({
                "id": "test-fact",
                "text": "The quick brown fox jumps over the lazy dog.",
                "source": "test",
                "source_url": "test.com",
                "language": "en"
            });
            
            self.server
                .mock("GET", "/api/v1/facts/random")
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body(response.to_string())
                .create()
        }
        
        pub fn create_chucknorris_mock(&mut self) -> Mock {
            let response = serde_json::json!({
                "value": {
                    "id": 1,
                    "joke": "Chuck Norris doesn't debug, he just stares at the code until it confesses."
                }
            });
            
            self.server
                .mock("GET", "/jokes/random")
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body(response.to_string())
                .create()
        }
        
        pub fn create_hackernews_mock(&mut self) -> Mock {
            let response = serde_json::json!([12345, 12346, 12347]);
            
            self.server
                .mock("GET", "/topstories.json")
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body(response.to_string())
                .create()
        }
        
        pub fn create_bbcnews_mock(&mut self) -> Mock {
            let response = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <item>
      <title>Test News Story</title>
      <link>https://bbc.com/news/test</link>
      <description>A test news story</description>
    </item>
  </channel>
</rss>"#;
            
            self.server
                .mock("GET", "/news/rss.xml")
                .with_status(200)
                .with_header("content-type", "application/rss+xml")
                .with_body(response)
                .create()
        }
        
        pub fn create_empty_mock(&mut self, path: &str) -> Mock {
            self.server
                .mock("GET", path)
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body("[]")
                .create()
        }
        
        pub fn create_error_mock(&mut self, path: &str, status: u16) -> Mock {
            self.server
                .mock("GET", path)
                .with_status(status)
                .with_body("Internal Server Error")
                .create()
        }
    }
    
    pub fn create_mock_client(url: &str) -> Client {
        Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("Failed to create mock client")
    }
    
    pub fn create_shutdown_channel() -> (watch::Sender<bool>, watch::Receiver<bool>) {
        watch::channel(false)
    }
}

#[cfg(test)]
pub mod test_helpers {
    use cazzmachine_lib::db::Database;
    use std::sync::Arc;
    use tempfile::TempDir;
    
    pub struct TestDatabase {
        pub db: Database,
        pub _temp_dir: TempDir,
    }
    
    impl TestDatabase {
        pub fn new() -> Self {
            let temp_dir = tempfile::tempdir().unwrap();
            let db = Database::new(temp_dir.path().to_path_buf())
                .expect("Failed to create test database");
            Self { db, _temp_dir: temp_dir }
        }
        
        pub fn arc(self: &Arc<Database>) -> Arc<Database> {
            self.db.clone()
        }
    }
    
    impl Default for TestDatabase {
        fn default() -> Self {
            Self::new()
        }
    }
}

#[cfg(test)]
pub mod health_check {
    use mockito::Server;
    
    pub async fn check_server_health(url: &str) -> bool {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(2))
            .build();
            
        match client {
            Ok(c) => {
                let result = c.get(url).send().await;
                result.is_ok()
            }
            Err(_) => false
        }
    }
    
    pub async fn wait_for_server(url: &str, timeout_secs: u64) -> bool {
        let start = std::time::Instant::now();
        while start.elapsed().as_secs() < timeout_secs {
            if check_server_health(url).await {
                return true;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        false
    }
}
