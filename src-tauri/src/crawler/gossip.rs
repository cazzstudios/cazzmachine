use super::provider::{ContentProvider, FetchedItem};

pub struct GossipProvider;

fn get_preview_image_url(preview: &serde_json::Value) -> Option<String> {
    let images = preview.get("images")?.as_array()?;
    let first = images.first()?;
    let source = first.get("source")?;
    let url = source.get("url")?;
    url.as_str().map(|s| s.to_string())
}

#[async_trait::async_trait]
impl ContentProvider for GossipProvider {
    fn name(&self) -> &str {
        "gossip"
    }

    fn category(&self) -> &str {
        "gossip"
    }

    async fn fetch(&self, client: &reqwest::Client) -> Vec<FetchedItem> {
        let feed_url = "https://www.reddit.com/r/popculturechat/hot.json";

        let listing: serde_json::Value = match super::util::fetch_json(client, feed_url).await {
            Ok(v) => v,
            Err(_) => return vec![],
        };

        let children = match listing["data"]["children"].as_array() {
            Some(c) => c,
            None => {
                return vec![];
            }
        };

        let mut items = Vec::new();
        for child in children.iter().take(6) {
            let data = &child["data"];
            let over_18 = data["over_18"].as_bool().unwrap_or(false);
            let stickied = data["stickied"].as_bool().unwrap_or(false);
            if over_18 || stickied {
                continue;
            }
            let title = match data["title"].as_str() {
                Some(t) => t.to_string(),
                None => continue,
            };
            let permalink = match data["permalink"].as_str() {
                Some(p) => p.to_string(),
                None => continue,
            };
            let thumbnail = data["thumbnail"].as_str().unwrap_or("").to_string();
            let thumb = if thumbnail.starts_with("http") {
                Some(thumbnail)
            } else {
                get_preview_image_url(&data["preview"])
            };

            let thumbnail_data = if let Some(ref url) = thumb {
                super::util::download_image(client, url).await
            } else {
                None
            };

            items.push(FetchedItem {
                source: "r/popculturechat".into(),
                category: "gossip".into(),
                title,
                url: format!("https://reddit.com{}", permalink),
                thumbnail_url: thumb,
                thumbnail_data,
                description: None,
            });
        }

        items
    }
}
