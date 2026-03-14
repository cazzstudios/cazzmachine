use super::provider::{ContentProvider, FetchedItem};
use serde::Deserialize;

pub struct UselessFactsProvider;

#[derive(Deserialize)]
#[allow(dead_code)]
struct FactResponse {
    text: String,
    source_url: String,
}

#[async_trait::async_trait]
impl ContentProvider for UselessFactsProvider {
    fn name(&self) -> &str {
        "uselessfacts"
    }

    fn category(&self) -> &str {
        "joke"
    }

    async fn fetch(&self, client: &reqwest::Client) -> Vec<FetchedItem> {
        let url = "https://uselessfacts.jsph.pl/random.json?language=en";
        let mut items = Vec::new();

        for _ in 0..5 {
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

            let fact: FactResponse = match response.json().await {
                Ok(f) => f,
                Err(_) => continue,
            };

            items.push(FetchedItem {
                source: "uselessfacts".into(),
                category: "fact".into(),
                title: fact.text.clone(),
                url: fact.source_url,
                thumbnail_url: None,
                thumbnail_data: None,
                description: Some(fact.text),
            });
        }

        items
    }
}
