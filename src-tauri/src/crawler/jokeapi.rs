use super::provider::{ContentProvider, FetchedItem};
use serde::Deserialize;

pub struct JokeApiProvider;

#[derive(Deserialize)]
#[allow(dead_code)]
struct JokeApiResponse {
    error: bool,
    jokes: Vec<Joke>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct Joke {
    #[serde(rename = "type")]
    joke_type: String,
    joke: Option<String>,
    setup: Option<String>,
    delivery: Option<String>,
    id: i64,
}

#[async_trait::async_trait]
impl ContentProvider for JokeApiProvider {
    fn name(&self) -> &str {
        "jokeapi"
    }

    fn category(&self) -> &str {
        "joke"
    }

    async fn fetch(&self, client: &reqwest::Client) -> Vec<FetchedItem> {
        let url = "https://v2.jokeapi.dev/joke/Any?type=single&amount=10&blacklistFlags=nsfw,religious,political,racist,sexist,explicit";

        let response = match client
            .get(url)
            .header("Accept", "application/json")
            .header("User-Agent", "cazzmachine/0.2.0")
            .send()
            .await
        {
            Ok(r) => r,
            Err(_) => return vec![],
        };

        if !response.status().is_success() {
            return vec![];
        }

        let data: JokeApiResponse = match response.json().await {
            Ok(d) => d,
            Err(_) => return vec![],
        };

        if data.error {
            return vec![];
        }

        let items: Vec<FetchedItem> = data
            .jokes
            .into_iter()
            .filter_map(|joke| {
                let title = joke.joke.clone().or_else(|| {
                    joke.setup.clone().map(|s| {
                        format!("{} {}", s, joke.delivery.clone().unwrap_or_default())
                    })
                })?;

                Some(FetchedItem {
                    source: "jokeapi".into(),
                    category: "joke".into(),
                    title: title.clone(),
                    url: format!("https://sv443.net/jokeapi/v2/joke/Any?idRange={}", joke.id),
                    thumbnail_url: None,
                    thumbnail_data: None,
                    description: Some(title),
                })
            })
            .collect();

        items
    }
}
