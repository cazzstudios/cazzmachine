use crate::db::models::CrawlItem;

pub struct FetchedItem {
    pub source: String,
    pub category: String,
    pub title: String,
    pub url: String,
    pub thumbnail_url: Option<String>,
    pub thumbnail_data: Option<String>,
    pub description: Option<String>,
}

impl FetchedItem {
    pub fn into_crawl_item(self) -> CrawlItem {
        let now = chrono::Local::now();
        let id = generate_stable_id(&self.url);
        CrawlItem {
            id,
            source: self.source,
            category: self.category,
            title: self.title,
            url: self.url,
            thumbnail_url: self.thumbnail_url,
            thumbnail_data: self.thumbnail_data,
            description: self.description,
            fetched_at: now.format("%Y-%m-%d %H:%M:%S").to_string(),
            is_seen: false,
            is_saved: false,
            is_consumed: false,
            session_date: now.format("%Y-%m-%d").to_string(),
        }
    }
}

fn generate_stable_id(url: &str) -> String {
    // Use first 32 chars of SHA256 hash of URL as stable ID
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(url.as_bytes());
    let result = hasher.finalize();
    hex::encode(&result[..16])
}

#[async_trait::async_trait]
#[allow(dead_code)]
pub trait ContentProvider: Send + Sync {
    fn name(&self) -> &str;
    fn category(&self) -> &str;
    async fn fetch(&self, client: &reqwest::Client) -> Vec<FetchedItem>;
}
