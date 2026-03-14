use super::provider::{ContentProvider, FetchedItem};
use serde::Deserialize;

pub struct HackerNewsProvider;

#[derive(Deserialize)]
#[allow(dead_code)]
struct HnItem {
    id: i64,
    title: String,
    url: Option<String>,
}

#[async_trait::async_trait]
impl ContentProvider for HackerNewsProvider {
    fn name(&self) -> &str {
        "hackernews"
    }

    fn category(&self) -> &str {
        "news"
    }

    async fn fetch(&self, client: &reqwest::Client) -> Vec<FetchedItem> {
        let ids_url = "https://hacker-news.firebaseio.com/v0/topstories.json";
        
        let ids: Vec<i64> = match client
            .get(ids_url)
            .header("User-Agent", "cazzmachine/0.2.0")
            .send()
            .await
        {
            Ok(r) => match r.json().await {
                Ok(ids) => ids,
                Err(_) => return vec![],
            },
            Err(_) => return vec![],
        };

        let mut items = Vec::new();
        
        for id in ids.iter().take(8) {
            let item_url = format!("https://hacker-news.firebaseio.com/v0/item/{}.json", id);
            
            let item: HnItem = match client
                .get(&item_url)
                .header("User-Agent", "cazzmachine/0.2.0")
                .send()
                .await
            {
                Ok(r) => match r.json().await {
                    Ok(i) => i,
                    Err(_) => continue,
                },
                Err(_) => continue,
            };

            let domain = item.url
                .as_ref()
                .and_then(|u| u.split("://").nth(1))
                .and_then(|u| u.split('/').next())
                .unwrap_or("news.ycombinator.com")
                .to_string();

            items.push(FetchedItem {
                source: "HackerNews".into(),
                category: "news".into(),
                title: item.title,
                url: item.url.unwrap_or_else(|| format!("https://news.ycombinator.com/item?id={}", id)),
                thumbnail_url: None,
                thumbnail_data: None,
                description: Some(domain),
            });
        }

        items
    }
}
