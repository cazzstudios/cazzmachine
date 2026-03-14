use super::provider::{ContentProvider, FetchedItem};
use serde::Deserialize;

pub struct ChuckNorrisProvider;

#[derive(Deserialize)]
#[allow(dead_code)]
struct ChuckJoke {
    id: String,
    value: String,
    url: String,
}

#[async_trait::async_trait]
impl ContentProvider for ChuckNorrisProvider {
    fn name(&self) -> &str {
        "chucknorris"
    }

    fn category(&self) -> &str {
        "joke"
    }

    async fn fetch(&self, client: &reqwest::Client) -> Vec<FetchedItem> {
        let url = "https://api.chucknorris.io/jokes/random";
        let mut items = Vec::new();

        for _ in 0..3 {
            let response = match client
                .get(url)
                .header("Accept", "application/json")
                .header("User-Agent", "cazzmachine/0.2.0")
                .send()
                .await
            {
                Ok(r) => r,
                Err(_) => continue,
            };

            if !response.status().is_success() {
                continue;
            }

            let joke: ChuckJoke = match response.json().await {
                Ok(j) => j,
                Err(_) => continue,
            };

            items.push(FetchedItem {
                source: "chucknorris".into(),
                category: "joke".into(),
                title: joke.value.clone(),
                url: joke.url,
                thumbnail_url: None,
                thumbnail_data: None,
                description: Some(joke.value),
            });
        }

        items
    }
}
