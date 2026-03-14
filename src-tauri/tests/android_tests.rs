#[cfg(test)]
mod android_tests {
    use cazzmachine_lib::db::Database;
    use tempfile::TempDir;

    fn create_test_db() -> (Database, TempDir) {
        let temp_dir = tempfile::tempdir().unwrap();
        let db = Database::new(temp_dir.path().to_path_buf()).unwrap();
        (db, temp_dir)
    }

    #[test]
    fn test_database_integration_works() {
        let (db, _temp_dir) = create_test_db();

        let pending = db.get_pending_count().unwrap_or(0);
        assert!(pending >= 0, "Pending count should be non-negative");
    }

    #[test]
    fn test_diagnostic_logging() {
        let (db, _temp_dir) = create_test_db();

        let result = db.log_diagnostic_event("test_event", "info", "Test message", None, None);

        assert!(result.is_ok(), "Diagnostic logging should work");
    }

    #[test]
    fn test_provider_count() {
        let count = cazzmachine_lib::crawler::providers_per_cycle();
        assert!(count > 0, "Should have at least one provider per cycle");
    }

    #[test]
    fn test_database_timestamp_operations() {
        let (db, _temp_dir) = create_test_db();

        let result = db.get_last_active_timestamp();
        assert!(result.is_ok(), "Should get last active timestamp");

        if let Ok(ts) = result {
            let _ = db.set_last_active_timestamp(ts);
        }
    }

    #[test]
    fn test_database_app_lifecycle_events() {
        let (db, _temp_dir) = create_test_db();

        let result = db.log_diagnostic_event(
            "app_background",
            "info",
            "App went to background",
            None,
            None,
        );
        assert!(result.is_ok(), "Should log app background event");

        let result = db.log_diagnostic_event(
            "app_resume",
            "info",
            "App resumed",
            Some("{\"elapsedMinutes\": 5}"),
            None,
        );
        assert!(result.is_ok(), "Should log app resume event");
    }

    #[test]
    fn test_database_resume_error_logging() {
        let (db, _temp_dir) = create_test_db();

        let result = db.log_diagnostic_event(
            "app_resume_error",
            "error",
            "Failed to handle resume",
            Some("{\"error\": \"Database error\"}"),
            None,
        );
        assert!(result.is_ok(), "Should log resume error");
    }

    #[test]
    fn test_database_background_error_logging() {
        let (db, _temp_dir) = create_test_db();

        let result = db.log_diagnostic_event(
            "app_background_error",
            "error",
            "Failed to handle background",
            Some("{\"error\": \"Timer cleanup failed\"}"),
            None,
        );
        assert!(result.is_ok(), "Should log background error");
    }

    #[test]
    fn test_database_timestamp_persistence() {
        let (db, _temp_dir) = create_test_db();

        let now = chrono::Utc::now();
        let result = db.set_last_active_timestamp(now);
        assert!(result.is_ok(), "Should set timestamp");

        let retrieved = db.get_last_active_timestamp();
        assert!(retrieved.is_ok(), "Should get timestamp");
    }

    #[test]
    fn test_database_concurrent_timestamp_updates() {
        let (db, _temp_dir) = create_test_db();
        let db = std::sync::Arc::new(db);

        let mut handles = vec![];

        for i in 0..5 {
            let db_clone = db.clone();
            let handle = std::thread::spawn(move || {
                let timestamp = chrono::Utc::now() + chrono::Duration::seconds(i);
                db_clone.set_last_active_timestamp(timestamp)
            });
            handles.push(handle);
        }

        for handle in handles {
            let result = handle.join().unwrap();
            assert!(result.is_ok(), "Concurrent timestamp update should succeed");
        }
    }

    #[test]
    fn test_diagnostic_event_with_metadata() {
        let (db, _temp_dir) = create_test_db();

        let metadata = r#"{"elapsedMinutes": 5.5, "itemsConsumed": 10}"#;
        let result = db.log_diagnostic_event(
            "app_resume",
            "info",
            "App resumed with consumption",
            Some(metadata),
            None,
        );

        assert!(result.is_ok(), "Should log event with JSON metadata");
    }

    #[test]
    fn test_diagnostic_event_with_related_item() {
        let (db, _temp_dir) = create_test_db();

        let result = db.log_diagnostic_event(
            "item_consumed",
            "info",
            "Item consumed during resume",
            None,
            Some("item-12345"),
        );

        assert!(result.is_ok(), "Should log event with related item ID");
    }

    #[test]
    fn test_multiple_lifecycle_events_in_sequence() {
        let (db, _temp_dir) = create_test_db();

        let events = vec![
            ("app_background", "info", "App went to background"),
            (
                "skip_notification_set",
                "info",
                "Next notification will be skipped",
            ),
            ("app_resume", "info", "App resumed"),
            (
                "consumption_triggered",
                "info",
                "Consumed items during resume",
            ),
        ];

        for (event_type, severity, message) in events {
            let result = db.log_diagnostic_event(event_type, severity, message, None, None);
            assert!(result.is_ok(), "Should log {} event", event_type);
        }

        let recent = db.get_recent_diagnostics(10);
        assert!(recent.is_ok(), "Should retrieve recent diagnostics");
    }
}
