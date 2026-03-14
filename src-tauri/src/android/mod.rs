pub mod background_service;

use std::sync::Arc;
use tauri::State;
use crate::db::Database;
use crate::shared::lifecycle_manager::LifecycleManager;

#[tauri::command]
pub fn on_android_app_background(db: State<'_, Arc<Database>>) -> Result<(), String> {
    LifecycleManager::on_background();
    db.log_diagnostic_event("android_lifecycle", "info", "Android app entered background", None, None)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn on_android_app_foreground(db: State<'_, Arc<Database>>) -> Result<(), String> {
    LifecycleManager::on_foreground();
    db.log_diagnostic_event("android_lifecycle", "info", "Android app entered foreground", None, None)
        .map_err(|e| e.to_string())
}


