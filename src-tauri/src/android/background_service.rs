use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;
use crate::db::Database;
use crate::shared::lifecycle_manager::LifecycleManager;
use crate::crawler::providers_per_cycle;
use crate::crawler::provider::ContentProvider;

pub struct AndroidBackgroundService {
    db: Arc<Database>,
    shutdown_rx: watch::Receiver<bool>,
}

impl AndroidBackgroundService {
    pub fn new(db: Arc<Database>, shutdown_rx: watch::Receiver<bool>) -> Self {
        Self { db, shutdown_rx }
    }

    async fn crawl(&self) -> u32 {
        let client = match reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build() {
            Ok(c) => c,
            Err(e) => {
                let _ = self.db.log_diagnostic_event("android_crawl_error", "warn", &format!("Failed to create HTTP client: {}", e), None, None);
                return 0;
            }
        };

        let providers: Vec<Box<dyn ContentProvider>> = vec![
            Box::new(crate::crawler::reddit::RedditProvider::memes()),
            Box::new(crate::crawler::reddit::RedditProvider::dad_jokes()),
            Box::new(crate::crawler::reddit::RedditProvider::celebrity_gossip()),
            Box::new(crate::crawler::dadjoke::DadJokeProvider),
            Box::new(crate::crawler::meme::RedditMemeProvider),
            Box::new(crate::crawler::video::RedditVideoProvider),
            Box::new(crate::crawler::gossip::GossipProvider),
            Box::new(crate::crawler::news::GoogleNewsRssProvider),
        ];

        let count = providers_per_cycle();
        let provider_idx = rand::random::<usize>() % providers.len();
        let mut items_added = 0u32;

        for i in 0..count {
            let idx = (provider_idx + i) % providers.len();
            let provider = &providers[idx];
            let _ = self.db.log_diagnostic_event("android_crawl", "info", &format!("Android background crawl: {} ({})", provider.name(), provider.category()), None, None);

            let items = provider.fetch(&client).await;
            for item in items {
                let crawl_item = item.into_crawl_item();
                match self.db.insert_item(&crawl_item) {
                    Ok(true) => items_added += 1,
                    Ok(false) => {}
                    Err(e) => {
                        let _ = self.db.log_diagnostic_event("android_insert_error", "warn", &format!("Failed to insert item: {}", e), None, None);
                    }
                }
            }
        }

        let _ = self.db.log_diagnostic_event("android_crawl", "info", &format!("Android background crawl complete: {} new items", items_added), None, None);
        items_added
    }

    pub async fn run(&self) {
        let _ = self.db.log_diagnostic_event("android_service", "info", "AndroidBackgroundService started", None, None);
        
        // Initial crawl on startup (after short delay to let app stabilize)
        tokio::time::sleep(Duration::from_secs(5)).await;
        let _ = self.db.log_diagnostic_event("android_service", "info", "AndroidBackgroundService: performing initial crawl", None, None);
        let initial_count = self.crawl().await;
        let _ = self.db.log_diagnostic_event("android_service", "info", &format!("AndroidBackgroundService: initial crawl added {} items", initial_count), None, None);

        loop {
            let mut rx = self.shutdown_rx.clone();
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(30 * 60)) => {
                    let _ = self.db.log_diagnostic_event("android_service", "info", "Android background service: 30 minute check", None, None);

                    // Always crawl on Android, regardless of foreground/background state
                    // The service itself indicates we're in a background context
                    let pending = match self.db.get_pending_count() {
                        Ok(count) => count,
                        Err(e) => {
                            let _ = self.db.log_diagnostic_event("android_service_error", "warn", &format!("Failed to get pending count: {}", e), None, None);
                            0
                        }
                    };
                    
                    let thread_count = crate::commands::THREAD_COUNT.load(std::sync::atomic::Ordering::Relaxed);
                    let target = 20 * thread_count as i64;

                    let _ = self.db.log_diagnostic_event("android_service", "info", &format!("Android background check: pending={}, target={}, thread_count={}", pending, target, thread_count), None, None);

                    if pending < target {
                        let _ = self.db.log_diagnostic_event("android_service", "info", &format!("Buffer low ({} < {}), crawling in background", pending, target), None, None);
                        let added = self.crawl().await;
                        let _ = self.db.log_diagnostic_event("android_service", "info", &format!("Background crawl complete: {} new items", added), None, None);
                    } else {
                        let _ = self.db.log_diagnostic_event("android_service", "info", &format!("Buffer sufficient: {} items (target: {})", pending, target), None, None);
                    }
                }
                _ = rx.changed() => {
                    if *rx.borrow() {
                        let _ = self.db.log_diagnostic_event("android_service", "info", "AndroidBackgroundService shutting down", None, None);
                        break;
                    }
                }
            }
        }
    }
}
