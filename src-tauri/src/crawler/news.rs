use super::provider::{ContentProvider, FetchedItem};
use super::util::{urlencoded, strip_html};
use serde::Deserialize;

pub struct GoogleNewsRssProvider;

#[derive(Deserialize)]
struct RssResponse {
    items: Vec<RssItem>,
}

#[derive(Deserialize)]
struct RssItem {
    title: String,
    link: String,
    #[serde(default)]
    description: String,
}

#[async_trait::async_trait]
impl ContentProvider for GoogleNewsRssProvider {
    fn name(&self) -> &str {
        "google-news"
    }

    fn category(&self) -> &str {
        "news"
    }

    async fn fetch(&self, client: &reqwest::Client) -> Vec<FetchedItem> {
        let rss_url = "https://news.google.com/rss?hl=en-US&gl=US&ceid=US:en";
        let rss_to_json = format!(
            "https://api.rss2json.com/v1/api.json?rss_url={}",
            urlencoded(rss_url)
        );

        let rss: RssResponse = match super::util::fetch_json(client, &rss_to_json).await {
            Ok(r) => r,
            Err(_) => return vec![],
        };

        let items: Vec<FetchedItem> = rss.items
            .into_iter()
            .take(8)
            .map(|item| {
                let desc = if item.description.is_empty() {
                    None
                } else {
                    Some(strip_html(&item.description).chars().take(200).collect())
                };
                FetchedItem {
                    source: "Google News".into(),
                    category: "news".into(),
                    title: item.title,
                    url: item.link,
                    thumbnail_url: None,
                    thumbnail_data: None,
                    description: desc,
                }
            })
            .collect();

        items
    }
}

