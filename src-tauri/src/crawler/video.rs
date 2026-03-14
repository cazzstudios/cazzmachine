use super::provider::{ContentProvider, FetchedItem};
use serde::Deserialize;

pub struct RedditVideoProvider;

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
    over_18: bool,
    #[serde(default)]
    stickied: bool,
    #[serde(default)]
    is_video: bool,
}

#[async_trait::async_trait]
impl ContentProvider for RedditVideoProvider {
    fn name(&self) -> &str {
        "reddit-videos"
    }

    fn category(&self) -> &str {
        "video"
    }

    async fn fetch(&self, client: &reqwest::Client) -> Vec<FetchedItem> {
        let subreddits = [
            "videos", "Unexpected", "WhatCouldGoWrong", "ContagiousLaughter",
            "WinStupidPrizes", "IdiotsInCars", "InstantKarma", "JusticeServed",
            "PublicFreakout", "StreetFights", "TikTokCringe", "facepalm",
            "AnimalsBeingDerps", "aww", "punny", "me_irl"
        ];
        let sub = subreddits[rand::random::<usize>() % subreddits.len()];
        let url = format!("https://www.reddit.com/r/{}/hot.json?limit=25", sub);

        let listing: RedditListing = match super::util::fetch_json(client, &url).await {
            Ok(l) => l,
            Err(_) => return vec![],
        };

        let mut items = Vec::new();
        for c in listing.data.children.into_iter().take(5) {
            if c.data.over_18 || c.data.stickied || (!c.data.is_video && !c.data.url.contains("youtu")) {
                continue;
            }
            let post = c.data;
            let thumb = if post.thumbnail.starts_with("http") {
                Some(post.thumbnail.clone())
            } else {
                None
            };

            let thumbnail_data = if let Some(ref url) = thumb {
                super::util::download_image(client, url).await
            } else {
                None
            };

            items.push(FetchedItem {
                source: format!("r/{}", sub),
                category: "video".into(),
                title: post.title,
                url: format!("https://reddit.com{}", post.permalink),
                thumbnail_url: thumb,
                thumbnail_data,
                description: None,
            });
        }

        items
    }
}
