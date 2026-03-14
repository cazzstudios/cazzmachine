use super::provider::{ContentProvider, FetchedItem};
use serde::Deserialize;

fn is_image_url(url: &str) -> bool {
    url.ends_with(".jpg")
        || url.ends_with(".jpeg")
        || url.ends_with(".png")
        || url.ends_with(".gif")
        || url.ends_with(".webp")
        || url.contains("i.redd.it")
        || url.contains("i.imgur.com")
        || url.contains("preview.redd.it")
}

pub struct RedditProvider {
    subreddit: String,
    category: String,
}

impl RedditProvider {
    pub fn memes() -> Self {
        Self {
            subreddit: "memes".into(),
            category: "meme".into(),
        }
    }

    pub fn dad_jokes() -> Self {
        Self {
            subreddit: "dadjokes".into(),
            category: "joke".into(),
        }
    }

    pub fn celebrity_gossip() -> Self {
        Self {
            subreddit: "entertainment".into(),
            category: "gossip".into(),
        }
    }
}

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
    thumbnail: String,
    #[serde(default)]
    selftext: String,
    #[serde(default)]
    over_18: bool,
    #[serde(default)]
    stickied: bool,
    #[serde(default)]
    is_self: bool,
    #[serde(default)]
    preview: serde_json::Value,
}

fn get_preview_image_url(post: &RedditPost) -> Option<String> {
    let preview = post.preview.get("images")?;
    let images = preview.as_array()?;
    let first = images.first()?;
    let source = first.get("source")?;
    let url = source.get("url")?;
    url.as_str().map(|s| s.to_string())
}

#[async_trait::async_trait]
impl ContentProvider for RedditProvider {
    fn name(&self) -> &str {
        &self.subreddit
    }

    fn category(&self) -> &str {
        &self.category
    }

    async fn fetch(&self, client: &reqwest::Client) -> Vec<FetchedItem> {
        let url = format!(
            "https://www.reddit.com/r/{}/hot.json?limit=10",
            self.subreddit
        );

        let listing: RedditListing = match super::util::fetch_json(client, &url).await {
            Ok(l) => l,
            Err(_) => return vec![],
        };

        let mut items = Vec::new();
        for c in listing.data.children.into_iter().take(8) {
            if c.data.over_18 || c.data.stickied {
                continue;
            }
            let post = c.data;
            let thumbnail = if !post.is_self && is_image_url(&post.url) {
                Some(post.url.clone())
            } else if post.thumbnail.starts_with("http") && post.thumbnail != "default" && post.thumbnail != "self" && post.thumbnail != "nsfw" {
                Some(post.thumbnail.clone())
            } else {
                get_preview_image_url(&post)
            };

            let thumbnail_data = if let Some(ref url) = thumbnail {
                super::util::download_image(client, url).await
            } else {
                None
            };

            let description = if post.selftext.is_empty() {
                None
            } else {
                Some(post.selftext.chars().take(200).collect())
            };

            items.push(FetchedItem {
                source: format!("r/{}", self.subreddit),
                category: self.category.clone(),
                title: post.title,
                url: format!("https://reddit.com{}", post.permalink),
                thumbnail_url: thumbnail,
                thumbnail_data,
                description,
            });
        }

        items
    }
}
