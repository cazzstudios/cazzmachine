pub mod consumption_calculator;
pub mod buffer_manager;
pub mod lifecycle_manager;

use std::sync::atomic::{AtomicI64, AtomicBool};
use std::sync::OnceLock;
use chrono::Utc;

pub struct SharedState {
    pub last_active_timestamp: AtomicI64,
    pub is_background_mode: AtomicBool,
    pub download_interval_minutes: i64,
}

impl Default for SharedState {
    fn default() -> Self {
        Self {
            last_active_timestamp: AtomicI64::new(Utc::now().timestamp_millis()),
            is_background_mode: AtomicBool::new(false),
            download_interval_minutes: 30,
        }
    }
}

static SHARED_STATE: OnceLock<SharedState> = OnceLock::new();

pub fn get_shared_state() -> &'static SharedState {
    SHARED_STATE.get_or_init(SharedState::default)
}
