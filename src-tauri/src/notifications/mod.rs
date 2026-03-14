pub mod templates;

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;

use crate::commands::THROTTLE_LEVEL;
use crate::db::Database;
#[cfg(target_os = "android")]
use crate::shared::lifecycle_manager::LifecycleManager;
#[cfg(target_os = "android")]
use crate::commands::SKIP_NEXT_NOTIFICATION;
use templates::generate_teaser;

pub struct NotificationEngine {
    db: Arc<Database>,
    shutdown_rx: watch::Receiver<bool>,
}

impl NotificationEngine {
    pub fn new(db: Arc<Database>, shutdown_rx: watch::Receiver<bool>) -> Self {
        Self { db, shutdown_rx }
    }

    fn get_cycle_interval(&self) -> Duration {
        let level = THROTTLE_LEVEL.load(std::sync::atomic::Ordering::Relaxed);
        // Formulas for linear active% from 2% to 91%:
        // S(level) = 1 + 4 × (level-1)/8          (scroll: 1→5 min)
        // A(level) = 0.02 + 0.89 × (level-1)/8   (active%: 2%→91%)
        // W(level) = S × ((1/A) - 1)             (standby: convex curve)
        let level_f = level as f64;
        let scroll_minutes = 1.0 + 4.0 * ((level_f - 1.0) / 8.0);
        let active_pct = 0.02 + 0.89 * ((level_f - 1.0) / 8.0);
        let standby_minutes = scroll_minutes * ((1.0 / active_pct) - 1.0);
        let total_cycle_minutes = scroll_minutes + standby_minutes;
        Duration::from_secs((total_cycle_minutes * 60.0) as u64)
    }

    pub async fn run(&self, app_handle: tauri::AppHandle) {
        let _ = self.db.log_diagnostic_event("notification_engine", "info", "NotificationEngine started", None, None);
        let regular_interval = self.get_cycle_interval();
        let _ = self.db.log_diagnostic_event("notification_engine", "info", &format!("Regular notification interval: {} seconds", regular_interval.as_secs()), None, None);

        let first_trigger_delay = Duration::from_secs(10);
        let _ = self.db.log_diagnostic_event("notification_engine", "info", &format!("First trigger in {} seconds", first_trigger_delay.as_secs()), None, None);
        
        tokio::time::sleep(first_trigger_delay).await;
        self.send_teaser(&app_handle).await;

        loop {
            let interval = self.get_cycle_interval();
            let _ = self.db.log_diagnostic_event("notification_engine", "info", &format!("Next notification in {} seconds", interval.as_secs()), None, None);

            let mut rx = self.shutdown_rx.clone();
            tokio::select! {
                _ = tokio::time::sleep(interval) => {},
                _ = rx.changed() => {
                    if *rx.borrow() {
                        let _ = self.db.log_diagnostic_event("notification_engine", "info", "NotificationEngine shutting down", None, None);
                        break;
                    }
                }
            }
            
            self.send_teaser(&app_handle).await;
        }
    }

    async fn send_teaser(&self, app_handle: &tauri::AppHandle) {
        #[cfg(target_os = "android")]
        {
            if SKIP_NEXT_NOTIFICATION.load(std::sync::atomic::Ordering::Relaxed) {
                SKIP_NEXT_NOTIFICATION.store(false, std::sync::atomic::Ordering::Relaxed);
                let _ = self.db.log_diagnostic_event("notification_engine", "debug", "Skipping notification: explicitly skipped by frontend", None, None);
                return;
            }
            if LifecycleManager::is_background_mode() {
                let _ = self.db.log_diagnostic_event("notification_engine", "debug", "Skipping notification: app is in background", None, None);
                return;
            }
        }

        let stats = match self.db.get_today_stats() {
            Ok(s) => s,
            Err(e) => {
                let _ = self.db.log_diagnostic_event("notification_error", "warn", &format!("Failed to get stats for notification: {}", e), None, None);
                return;
            }
        };

        let latest_item = self.db.get_latest_unseen_item().ok().flatten();
        let message = generate_teaser(&stats, latest_item.as_ref());

        // Emit event to trigger frontend doomscrolling cycle (no OS notification)
        if let Err(e) = tauri::Emitter::emit(app_handle, "cazz-notification", &message) {
            let _ = self.db.log_diagnostic_event("notification_error", "warn", &format!("Failed to emit notification event: {}", e), None, None);
        }

        if let Some(ref item) = latest_item {
            let _ = self.db.mark_item_seen(&item.id);
        }
        let _ = self.db.log_diagnostic_event(
            "notification_sent",
            "info",
            &message,
            latest_item.as_ref().map(|i| i.id.as_str()),
            None
        );
    }
}
