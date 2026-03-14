pub mod scoring;
pub mod provider;
pub mod reddit;
pub mod dadjoke;
pub mod news;
pub mod meme;
pub mod video;
pub mod gossip;
pub mod util;
pub mod jokeapi;
pub mod uselessfacts;
pub mod chucknorris;
pub mod hackernews;
pub mod bbcnews;

use crate::commands::THROTTLE_LEVEL;
use std::time::Instant;

pub fn providers_per_cycle() -> usize {
    let level = THROTTLE_LEVEL.load(std::sync::atomic::Ordering::Relaxed);
    if level >= 7 {
        13
    } else if level >= 4 {
        (6 + (level as usize - 4) * 2).min(13)
    } else {
        (3 + level as usize).min(13)
    }
}

pub async fn execute_crawl(db: &crate::db::Database, providers: Vec<Box<dyn provider::ContentProvider>>) -> u32 {
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            let _ = db.log_diagnostic_event("crawl_error", "warn", &format!("Failed to build HTTP client: {}", e), None, None);
            return 0;
        }
    };

    let mut items_added = 0u32;

    for provider in &providers {
        let provider_name = provider.name().to_string();
        let start = Instant::now();
        let items = provider.fetch(&client).await;
        let elapsed_ms = start.elapsed().as_millis() as u64;

        // Track provider metrics
        let success = !items.is_empty();
        if let Err(e) = db.upsert_provider_stat(&provider_name, items.len(), success) {
            let _ = db.log_diagnostic_event(
                "metric_error", "warn",
                &format!("Failed to upsert provider stat: {}", e),
                None, None,
            );
        }

        // Log fetch timing
        let _ = db.log_diagnostic_event(
            "crawl_timing", "debug",
            &format!("{}: fetched in {}ms", provider_name, elapsed_ms),
            None, None,
        );

        let item_count = items.len();

        for item in items {
            let crawl_item = item.into_crawl_item();
            match db.insert_item(&crawl_item) {
                Ok(true) => items_added += 1,
                Ok(false) => {}
                Err(e) => {
                    let _ = db.log_diagnostic_event(
                        "insert_error", "warn",
                        &format!("Failed to insert item: {}", e),
                        None, None,
                    );
                }
            }
        }

        if item_count > 0 {
            let _ = db.log_diagnostic_event(
                "crawl_success", "info",
                &format!("{}: fetched {} items", provider_name, item_count),
                None, None,
            );
        } else {
            let _ = db.log_diagnostic_event(
                "crawl_error", "warn",
                &format!("{}: no items fetched", provider_name),
                None, None,
            );
        }
    }

    let _ = db.log_diagnostic_event(
        "crawl_complete", "info",
        &format!("Crawl complete: {} new items", items_added),
        None, None,
    );

    items_added
}
