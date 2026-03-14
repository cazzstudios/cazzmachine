use super::provider::{ContentProvider, FetchedItem};
use super::util::{fetch_json, strip_html};
use serde::Deserialize;

pub struct BbcNewsProvider;

#[derive(Deserialize)]
struct Rss2Json {
    #[allow(dead_code)]
    status: String,
    #[allow(dead_code)]
    feed: Option<RssFeed>,
    items: Option<Vec<RssItem>>,
}

#[derive(Deserialize)]
struct RssFeed {
    #[allow(dead_code)]
    title: Option<String>,
}

#[derive(Deserialize)]
struct RssItem {
    title: Option<String>,
    link: Option<String>,
    description: Option<String>,
    thumbnail: Option<String>,
}

#[async_trait::async_trait]
impl ContentProvider for BbcNewsProvider {
    fn name(&self) -> &str {
        "bbc-news"
    }

    fn category(&self) -> &str {
        "news"
    }

    async fn fetch(&self, client: &reqwest::Client) -> Vec<FetchedItem> {
        let rss_url = "https://feeds.bbci.co.uk/news/rss.xml";
        let url = format!("https://api.rss2json.com/v1/api.json?rss_url={}", rss_url);

        let response: Rss2Json = match fetch_json(client, &url).await {
            Ok(r) => r,
            Err(_) => return vec![],
        };

        let items_vec = match response.items {
            Some(i) => i,
            None => return vec![],
        };

        let items: Vec<FetchedItem> = items_vec
            .into_iter()
            .take(8)
            .filter_map(|item| {
                let title = item.title?;
                let link = item.link?;
                let desc = item.description.map(|d| strip_html(&d).chars().take(150).collect());
                
                Some(FetchedItem {
                    source: "BBC News".into(),
                    category: "news".into(),
                    title,
                    url: link,
                    thumbnail_url: item.thumbnail,
                    thumbnail_data: None,
                    description: desc,
                })
            })
            .collect();

        items
    }
}
