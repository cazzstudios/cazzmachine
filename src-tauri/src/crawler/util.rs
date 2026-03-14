use base64::{engine::general_purpose::STANDARD, Engine};
use serde::de::DeserializeOwned;

pub async fn fetch_json<T: DeserializeOwned>(
    client: &reqwest::Client,
    url: &str,
) -> Result<T, ()> {
    let response = client
        .get(url)
        .header("User-Agent", "cazzmachine/0.1.0")
        .send()
        .await
        .map_err(|_| ())?;
    
    if !response.status().is_success() {
        return Err(());
    }
    
    response.json::<T>().await.map_err(|_| ())
}

pub async fn download_image(client: &reqwest::Client, url: &str) -> Option<String> {
    let response = client
        .get(url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .header("Referer", "https://www.reddit.com/")
        .send()
        .await
        .ok()?;

    if !response.status().is_success() {
        return None;
    }

    let bytes = response.bytes().await.ok()?;
    let mime = if url.contains(".png") {
        "image/png"
    } else if url.contains(".gif") {
        "image/gif"
    } else if url.contains(".webp") {
        "image/webp"
    } else {
        "image/jpeg"
    };

    let base64 = STANDARD.encode(&bytes);
    Some(format!("data:{};base64,{}", mime, base64))
}

pub(crate) fn urlencoded(s: &str) -> String {
    s.replace(':', "%3A")
        .replace('/', "%2F")
        .replace('?', "%3F")
        .replace('=', "%3D")
        .replace('&', "%26")
}

pub(crate) fn strip_html(s: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }
    result
}
