use std::sync::atomic::{AtomicU8, AtomicBool, Ordering};
use std::sync::Arc;
use tauri::State;
use base64::{engine::general_purpose::STANDARD, Engine};

use crate::db::models::{
    ClearDiagnosticsResult, ConsumeResult, CrawlItem, DayStats, DaySummary, DiagnosticLog,
    DiagnosticSummary, ProviderStatus,
};
use crate::db::Database;
use crate::summary;
use crate::crawler;
use crate::crawler::provider::ContentProvider;

pub static THROTTLE_LEVEL: AtomicU8 = AtomicU8::new(5);
pub static THREAD_COUNT: AtomicU8 = AtomicU8::new(1);
pub static SKIP_NEXT_NOTIFICATION: AtomicBool = AtomicBool::new(false);

#[tauri::command]
pub fn get_throttle_level() -> u8 {
    THROTTLE_LEVEL.load(Ordering::Relaxed)
}

#[tauri::command]
pub async fn set_throttle_level(db: State<'_, Arc<Database>>, level: u8) -> Result<(), String> {
    let level = level.clamp(1, 9);
    THROTTLE_LEVEL.store(level, Ordering::Relaxed);
    db.log_diagnostic_event("setting_change", "info", &format!("Throttle level set to {}", level), None, None)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_consumption_threads() -> u8 {
    THREAD_COUNT.load(Ordering::Relaxed)
}

#[tauri::command]
pub async fn set_consumption_threads(db: State<'_, Arc<Database>>, count: u8) -> Result<(), String> {
    let count = count.clamp(1, 8);
    THREAD_COUNT.store(count, Ordering::Relaxed);
    db.log_diagnostic_event("setting_change", "info", &format!("Thread count set to {}", count), None, None)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn skip_next_notification() {
    SKIP_NEXT_NOTIFICATION.store(true, Ordering::Relaxed);
}

#[tauri::command]
pub async fn get_today_items(db: State<'_, Arc<Database>>) -> Result<Vec<CrawlItem>, String> {
    db.get_items_for_today().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_items_by_category(
    db: State<'_, Arc<Database>>,
    category: String,
) -> Result<Vec<CrawlItem>, String> {
    db.get_items_by_category(&category)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_today_stats(db: State<'_, Arc<Database>>) -> Result<DayStats, String> {
    db.get_today_stats().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_daily_summary(db: State<'_, Arc<Database>>) -> Result<DaySummary, String> {
    summary::generate_daily_summary(&db)
}

#[tauri::command]
pub async fn toggle_save_item(db: State<'_, Arc<Database>>, item_id: String) -> Result<bool, String> {
    db.toggle_item_saved(&item_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn mark_item_seen(db: State<'_, Arc<Database>>, item_id: String) -> Result<(), String> {
    db.mark_item_seen(&item_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn consume_pending_items(
    db: State<'_, Arc<Database>>,
    budget_minutes: f64,
) -> Result<ConsumeResult, String> {
    // Step 1: Check buffer health
    let pending_count = db.get_pending_count().map_err(|e| e.to_string())?;
    let thread_count = THREAD_COUNT.load(Ordering::Relaxed);
    let throttle_level = THROTTLE_LEVEL.load(Ordering::Relaxed);
    
    // Approximate total cost using avg cost per item (0.5 minutes)
    let total_cost = pending_count as f64 * 0.5;
    let health = Database::compute_buffer_health(pending_count, total_cost, thread_count, throttle_level);
    
    // Step 2: If buffer is critical or low, trigger crawl to refill
    if health == "critical" || health == "low" || (throttle_level >= 7 && health == "moderate") {
        let _ = db.log_diagnostic_event("buffer_refill", "info", &format!("Buffer health: {}, triggering refill crawl", health), None, None);
        
        let mut items_added = 0u32;
        let mut attempts = 0;
        let max_retries = 2;

        while attempts < max_retries {
            // Use ALL 13 providers to ensure diversity - weighted selection
            // causes starvation when only some providers have scores
            let selected: Vec<Box<dyn ContentProvider>> = (0..13).map(|i| make_provider(i)).collect();
            
            items_added = crawler::execute_crawl(&db, selected).await;
            
            if items_added > 0 {
                break;
            }
            
            attempts += 1;
        }
        
        let _ = db.log_diagnostic_event("buffer_refill", "info", &format!("Buffer refill crawl added {} items", items_added), None, None);
    }

    
    // Step 3: Then call the sync db.consume_pending_items
    db.consume_pending_items(budget_minutes)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_url(url: String) -> Result<(), String> {
    open::that(&url).map_err(|e| format!("Failed to open URL: {}", e))
}

#[tauri::command]
pub async fn prune_old_items(db: State<'_, Arc<Database>>) -> Result<(i64, i64), String> {
    db.prune_old_items().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_pending_count(db: State<'_, Arc<Database>>) -> Result<i64, String> {
    db.get_pending_count().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_last_active_timestamp(db: State<'_, Arc<Database>>) -> Result<i64, String> {
    let timestamp = db.get_last_active_timestamp().map_err(|e| e.to_string())?;
    Ok(timestamp.timestamp_millis())
}

#[tauri::command]
pub async fn set_last_active_timestamp(
    db: State<'_, Arc<Database>>,
    timestamp: i64,
) -> Result<(), String> {
    let datetime = chrono::DateTime::from_timestamp_millis(timestamp).ok_or("Invalid timestamp")?;
    db.set_last_active_timestamp(datetime)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_diagnostic_summary(db: State<'_, Arc<Database>>) -> Result<DiagnosticSummary, String> {
    db.get_diagnostic_summary().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_provider_status(db: State<'_, Arc<Database>>) -> Result<Vec<ProviderStatus>, String> {
    db.get_provider_status().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_recent_diagnostics(
    db: State<'_, Arc<Database>>,
    limit: i64,
) -> Result<Vec<DiagnosticLog>, String> {
    db.get_recent_diagnostics(limit).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn clear_diagnostics(
    db: State<'_, Arc<Database>>,
    older_than_days: i64,
) -> Result<ClearDiagnosticsResult, String> {
    let deleted_count = db
        .clear_diagnostics(older_than_days)
        .map_err(|e| e.to_string())?;
    Ok(ClearDiagnosticsResult { deleted_count })
}

#[tauri::command]
pub async fn trigger_crawl(db: State<'_, Arc<Database>>) -> Result<u32, String> {
    let mut items_added = 0u32;
    let mut attempts = 0;


    while attempts < 3 {
        // Use ALL 13 providers to ensure diversity - weighted selection
        // causes starvation when only some providers have scores
        let selected: Vec<Box<dyn ContentProvider>> = (0..13).map(|i| make_provider(i)).collect();
        
        items_added = crawler::execute_crawl(&db, selected).await;
        
        if items_added > 0 {
            break;
        }
        
        attempts += 1;
    }
    
    Ok(items_added)
}

fn name_to_idx(name: &str) -> Option<usize> {
    match name {
        // Reddit providers
        "memes" => Some(0),
        "dad_jokes" => Some(1),
        "celebrity_gossip" => Some(2),
        // Other providers - accept multiple name variants
        "icanhazdadjoke" | "dadjoke" => Some(3),
        "reddit-memes" | "reddit_meme" | "reddit_memes" => Some(4),
        "reddit-videos" | "reddit_video" => Some(5),
        "gossip" => Some(6),
        "google-news" | "google_news" => Some(7),
        "jokeapi" => Some(8),
        "uselessfacts" => Some(9),
        "chucknorris" => Some(10),
        "hackernews" => Some(11),
        "bbc-news" | "bbcnews" => Some(12),
        _ => None,
    }
}


fn make_provider(idx: usize) -> Box<dyn ContentProvider> {
    match idx {
        0 => Box::new(crawler::reddit::RedditProvider::memes()),
        1 => Box::new(crawler::reddit::RedditProvider::dad_jokes()),
        2 => Box::new(crawler::reddit::RedditProvider::celebrity_gossip()),
        3 => Box::new(crawler::dadjoke::DadJokeProvider),
        4 => Box::new(crawler::meme::RedditMemeProvider),
        5 => Box::new(crawler::video::RedditVideoProvider),
        6 => Box::new(crawler::gossip::GossipProvider),
        7 => Box::new(crawler::news::GoogleNewsRssProvider),
        8 => Box::new(crawler::jokeapi::JokeApiProvider),
        9 => Box::new(crawler::uselessfacts::UselessFactsProvider),
        10 => Box::new(crawler::chucknorris::ChuckNorrisProvider),
        11 => Box::new(crawler::hackernews::HackerNewsProvider),
        _ => Box::new(crawler::bbcnews::BbcNewsProvider),
    }
}

#[tauri::command]
pub async fn log_diagnostic(
    db: State<'_, Arc<Database>>,
    event_type: String,
    severity: String,
    message: String,
    metadata: Option<String>,
) -> Result<(), String> {
    db.log_diagnostic_event(&event_type, &severity, &message, metadata.as_deref(), None)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn fetch_image(url: String) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .get(&url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .header("Referer", "https://www.reddit.com/")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch image: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Image fetch failed with status: {}", response.status()));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read image bytes: {}", e))?;

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
    Ok(format!("data:{};base64,{}", mime, base64))
}
