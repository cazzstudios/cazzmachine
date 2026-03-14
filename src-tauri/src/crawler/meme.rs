use super::provider::{ContentProvider, FetchedItem};
use serde::Deserialize;

pub struct RedditMemeProvider;

#[derive(Deserialize)]
struct RedditListing {
    data: RedditListingData,
}

#[derive(Deserialize)]
struct RedditListingData {
    children: Vec<RedditChild>,
}

#[derive(Deserialize)]
struct RedditChild {
    data: RedditPost,
}

#[derive(Deserialize)]
struct RedditPost {
    title: String,
    permalink: String,
    #[serde(default)]
    url: String,
    #[serde(default)]
    post_hint: Option<String>,
    #[serde(default)]
    over_18: bool,
    #[serde(default)]
    stickied: bool,
}

#[async_trait::async_trait]
impl ContentProvider for RedditMemeProvider {
    fn name(&self) -> &str {
        "reddit-memes"
    }

    fn category(&self) -> &str {
        "meme"
    }

    async fn fetch(&self, client: &reqwest::Client) -> Vec<FetchedItem> {
        let subreddits = ["dankmemes", "me_irl", "funny", "wholesomememes"];
        let sub = subreddits[rand::random::<usize>() % subreddits.len()];
        let url = format!("https://www.reddit.com/r/{}/hot.json?limit=10", sub);

        let listing: RedditListing = match super::util::fetch_json(client, &url).await {
            Ok(l) => l,
            Err(_) => return vec![],
        };

        let mut items = Vec::new();
        for c in listing.data.children.into_iter().take(6) {
            if c.data.over_18 || c.data.stickied || c.data.post_hint.as_deref() != Some("image") {
                continue;
            }
            let post = c.data;
            let image_url = if post.url.starts_with("http") {
                Some(post.url.clone())
            } else {
                None
            };

            let thumbnail_data = if let Some(ref url) = image_url {
                super::util::download_image(client, url).await
            } else {
                None
            };

            items.push(FetchedItem {
                source: format!("r/{}", sub),
                category: "meme".into(),
                title: post.title,
                url: format!("https://reddit.com{}", post.permalink),
                thumbnail_url: image_url,
                thumbnail_data,
                description: None,
            });
        }

        items
    }
}
