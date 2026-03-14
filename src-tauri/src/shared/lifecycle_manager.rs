use chrono::{DateTime, Utc};
use std::sync::atomic::{AtomicBool, Ordering};

static IS_BACKGROUND_MODE: AtomicBool = AtomicBool::new(false);

pub struct LifecycleManager;

impl LifecycleManager {
    /// Mark app as going to background
    pub fn on_background() {
        IS_BACKGROUND_MODE.store(true, Ordering::Relaxed);
    }

    /// Mark app as coming to foreground
    pub fn on_foreground() {
        IS_BACKGROUND_MODE.store(false, Ordering::Relaxed);
    }

    /// Check if app is in background mode
    pub fn is_background_mode() -> bool {
        IS_BACKGROUND_MODE.load(Ordering::Relaxed)
    }

    /// Calculate elapsed time since last active in minutes
    pub fn get_elapsed_minutes(last_active: DateTime<Utc>) -> f64 {
        let now = Utc::now();
        let elapsed = now.signed_duration_since(last_active);
        elapsed.num_milliseconds() as f64 / 60000.0
    }
}
