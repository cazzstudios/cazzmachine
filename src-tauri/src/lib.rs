mod commands;
pub mod crawler;
pub mod db;
mod notifications;
mod summary;
#[cfg(target_os = "android")] mod shared;

#[cfg(target_os = "android")]
mod android;

use std::sync::Arc;
use tauri::Manager;
use tokio::sync::watch;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    #[cfg(mobile)]
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_app_events::init());

    #[cfg(not(mobile))]
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_http::init());

    builder
        .setup(move |app| {
            let app_dir = app
                .path()
                .app_data_dir()
                .expect("failed to get app data dir");

            let database =
                Arc::new(db::Database::new(app_dir).expect("failed to initialize database"));

            // Clear diagnostic logs from previous sessions on startup
            let _ = database.clear_diagnostics(0);

            app.manage(database.clone());

            let notif_db = database.clone();
            let notif_shutdown = shutdown_rx.clone();
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let engine = notifications::NotificationEngine::new(notif_db, notif_shutdown);
                engine.run(app_handle).await;
            });

            #[cfg(target_os = "android")]
            {
                let bg_db = database.clone();
                let bg_shutdown = shutdown_rx.clone();

                tauri::async_runtime::spawn(async move {
                    let service = android::background_service::AndroidBackgroundService::new(bg_db, bg_shutdown);
                    service.run().await;
                });
            }

            Ok(())
        })
        .on_window_event(move |_window, event| {
            if let tauri::WindowEvent::Destroyed = event {
                let _ = shutdown_tx.send(true);
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_today_items,
            commands::get_items_by_category,
            commands::get_today_stats,
            commands::get_daily_summary,
            commands::toggle_save_item,
            commands::mark_item_seen,
            commands::consume_pending_items,
            commands::open_url,
            commands::get_throttle_level,
            commands::set_throttle_level,
            commands::get_consumption_threads,
            commands::set_consumption_threads,
            commands::get_pending_count,
            commands::prune_old_items,
            commands::get_last_active_timestamp,
            commands::set_last_active_timestamp,
            commands::get_diagnostic_summary,
            commands::get_provider_status,
            commands::get_recent_diagnostics,
            commands::clear_diagnostics,
            commands::trigger_crawl,
            commands::log_diagnostic,
            commands::fetch_image,
            commands::skip_next_notification,
            #[cfg(target_os = "android")]
            android::on_android_app_background,
            #[cfg(target_os = "android")]
            android::on_android_app_foreground,
        ])
        .run(tauri::generate_context!())
        .expect("error while running cazzmachine");
}
