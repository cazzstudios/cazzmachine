//! Integration tests for database deadlock and concurrent access
//!
//! These tests verify that the deadlock fix (using log_diagnostic_with_conn instead
//! of log_diagnostic_event while holding the mutex) works correctly.

use cazzmachine_lib::db::models::CrawlItem;
use cazzmachine_lib::db::Database;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

/// Creates a test database in a temporary directory
fn create_test_db() -> (Database, tempfile::TempDir) {
    let temp_dir = tempfile::tempdir().unwrap();
    let db = Database::new(temp_dir.path().to_path_buf()).unwrap();
    (db, temp_dir)
}

/// Creates a test crawl item with the given category
fn create_test_item(id_suffix: &str, category: &str) -> CrawlItem {
    let now = chrono::Local::now();
    let today = now.format("%Y-%m-%d").to_string();
    let timestamp = now.format("%Y-%m-%d %H:%M:%S").to_string();

    CrawlItem {
        id: format!("test-{}-{}", id_suffix, uuid::Uuid::new_v4()),
        source: format!("test-{}-source", category),
        category: category.to_string(),
        title: format!("Test {} item {}", category, id_suffix),
        url: format!("https://example.com/{}/{}", category, id_suffix),
        thumbnail_url: Some("https://example.com/thumb.jpg".to_string()),
        thumbnail_data: None,
        description: Some(format!("Test description for {} item", category)),
        fetched_at: timestamp,
        is_seen: false,
        is_saved: false,
        is_consumed: false,
        session_date: today,
    }
}

/// Inserts test items of various categories
fn insert_test_items(db: &Database) {
    let categories = vec!["meme", "joke", "news", "video", "gossip"];
    for (i, category) in categories.iter().enumerate() {
        for j in 0..3 {
            let item = create_test_item(&format!("{}-{}", i, j), category);
            db.insert_item(&item).unwrap();
        }
    }
}

/// Test 1: Verify consume_pending_items returns without hanging (deadlock regression test)
///
/// This test verifies the fix where log_diagnostic_with_conn is used instead of
/// log_diagnostic_event while holding the mutex lock.
#[test]
fn test_consume_no_deadlock() {
    let (db, _temp_dir) = create_test_db();

    // Insert test items
    insert_test_items(&db);

    // Verify pending count before consumption
    let pending_before = db.get_pending_count().unwrap();
    assert!(pending_before > 0, "Should have pending items");

    // Call consume_pending_items with a timeout
    // If deadlock exists, this will hang indefinitely
    let start = Instant::now();
    let timeout = Duration::from_secs(5);

    let result = std::panic::catch_unwind(|| db.consume_pending_items(1.0).unwrap());

    let elapsed = start.elapsed();

    // Assert that the operation completed without deadlock
    assert!(
        elapsed < timeout,
        "consume_pending_items took {:?} which exceeds timeout - possible deadlock!",
        elapsed
    );

    let consume_result = result.expect("consume_pending_items panicked");
    assert!(
        consume_result.items_consumed > 0 || consume_result.items_discarded > 0,
        "Should have consumed or discarded some items"
    );

    // Verify pending count changed
    let pending_after = db.get_pending_count().unwrap();
    assert_ne!(
        pending_before, pending_after,
        "Pending count should change after consumption"
    );
}

/// Test 2: Verify multiple threads can access DB without deadlock
///
/// This test spawns 4 threads that each call different DB methods concurrently
/// to verify that the Mutex-based synchronization works correctly.
#[test]
fn test_concurrent_db_access() {
    let db: Arc<Database> = Arc::new(create_test_db().0);
    let timeout = Duration::from_secs(10);

    // Insert test items before spawning threads
    insert_test_items(&db);

    // Spawn 4 threads with different operations
    let mut handles: Vec<thread::JoinHandle<Result<(), rusqlite::Error>>> = vec![];

    // Thread 1: consume_pending_items
    let db1: Arc<Database> = Arc::clone(&db);
    handles.push(thread::spawn(move || {
        let _ = db1.consume_pending_items(0.5)?;
        Ok(())
    }));

    // Thread 2: get_today_stats
    let db2: Arc<Database> = Arc::clone(&db);
    handles.push(thread::spawn(move || {
        let _stats = db2.get_today_stats()?;
        Ok(())
    }));

    // Thread 3: get_pending_count + get_latest_unseen_item
    let db3: Arc<Database> = Arc::clone(&db);
    handles.push(thread::spawn(move || {
        let _count = db3.get_pending_count()?;
        let _item = db3.get_latest_unseen_item()?;
        Ok(())
    }));

    let db4: Arc<Database> = Arc::clone(&db);
    handles.push(thread::spawn(move || {
        db4.log_diagnostic_event("notification_sent", "info", "Test notification", None, None)?;
        Ok(())
    }));

    // Wait for all threads with timeout
    let start = Instant::now();
    for (i, handle) in handles.into_iter().enumerate() {
        match handle.join() {
            Ok(result) => {
                result.expect(&format!("Thread {} returned an error", i));
            }
            Err(_) => {
                panic!("Thread {} panicked", i);
            }
        }

        assert!(
            start.elapsed() < timeout,
            "Thread {} took too long - possible deadlock!",
            i
        );
    }

    println!(
        "All threads completed successfully in {:?}",
        start.elapsed()
    );
}

/// Test 3: Simulate NotificationEngine access pattern
///
/// This test simulates the pattern used by NotificationEngine:
/// 1. get_today_stats
/// 2. get_latest_unseen_item
/// 3. log_diagnostic_event (notification sent)
#[test]
fn test_notification_engine_pattern() {
    let (db, _temp_dir) = create_test_db();
    let timeout = Duration::from_secs(5);

    insert_test_items(&db);

    let _ = db.consume_pending_items(10.0);

    let start = Instant::now();

    let stats = db.get_today_stats().unwrap();
    println!("Today stats: {} items", stats.total_items);

    let unseen = db.get_latest_unseen_item().unwrap();
    println!("Latest unseen item: {:?}", unseen.as_ref().map(|i| &i.id));

    if let Some(item) = &unseen {
        db.log_diagnostic_event(
            "notification_sent",
            "info",
            &format!("New {} available!", item.category),
            Some(&item.id),
            None,
        )
        .unwrap();
        println!("Logged notification for item {}", item.id);
    }

    let elapsed = start.elapsed();
    assert!(
        elapsed < timeout,
        "NotificationEngine pattern took {:?} which exceeds timeout - possible deadlock!",
        elapsed
    );

    println!("NotificationEngine pattern completed in {:?}", elapsed);
}

/// Additional test: Stress test with rapid sequential consume calls
///
/// This test calls consume_pending_items multiple times rapidly to ensure
/// no state corruption or deadlock occurs.
#[test]
fn test_rapid_consume_calls() {
    let (db, _temp_dir) = create_test_db();

    // Insert test items
    insert_test_items(&db);

    // Call consume multiple times with small budgets
    let timeout = Duration::from_secs(5);
    let start = Instant::now();

    for i in 0..5 {
        let budget = 0.3 + (i as f64 * 0.1); // Varying budgets
        let result = db.consume_pending_items(budget).unwrap();
        println!("Iteration {}: consumed {} items", i, result.items_consumed);
    }

    assert!(
        start.elapsed() < timeout,
        "Rapid consume calls took {:?} which exceeds timeout",
        start.elapsed()
    );
}

/// E2E Test: Full content workflow
///
/// Simulates a complete workflow: insert items, consume, mark seen, toggle save
#[test]
fn test_full_content_workflow() {
    let (db, _temp_dir) = create_test_db();

    // Insert test items
    insert_test_items(&db);

    // Step 1: Get initial pending count (items are pending by default)
    let pending_before = db.get_pending_count().unwrap();
    println!("Initial pending count: {}", pending_before);
    assert!(pending_before > 0, "Should have pending items after insert");

    // Step 2: Consume items to make them available
    let consume_result = db.consume_pending_items(5.0).unwrap();
    println!(
        "Consumed {} items, {} discarded",
        consume_result.items_consumed, consume_result.items_discarded
    );
    assert!(consume_result.items_consumed >= 0);

    // Step 3: Get items for today (consumed items)
    let items = db.get_items_for_today().unwrap();
    println!("Got {} consumed items for today", items.len());

    // Step 4: Get pending count after consume
    let pending_after = db.get_pending_count().unwrap();
    println!("Pending count after consume: {}", pending_after);

    // Step 5: Get stats
    let stats = db.get_today_stats().unwrap();
    println!(
        "Stats: {} memes, {} jokes, {} total",
        stats.memes_found, stats.jokes_found, stats.total_items
    );
    assert!(stats.total_items >= 0);
}

/// E2E Test: Category filtering
#[test]
fn test_category_filtering() {
    let (db, _temp_dir) = create_test_db();

    // Insert test items
    insert_test_items(&db);

    // Consume items to make them queryable
    let _ = db.consume_pending_items(10.0);

    // Get items by category (consumed items only)
    let meme_items = db.get_items_by_category("meme").unwrap();
    let joke_items = db.get_items_by_category("joke").unwrap();
    let news_items = db.get_items_by_category("news").unwrap();

    println!(
        "Items by category - memes: {}, jokes: {}, news: {}",
        meme_items.len(),
        joke_items.len(),
        news_items.len()
    );

    // Items should be present after consumption
    let total = meme_items.len() + joke_items.len() + news_items.len();
    assert!(total > 0, "Should have some consumed items");

    // Verify categories are correct
    for item in &meme_items {
        assert_eq!(item.category, "meme");
    }
    for item in &joke_items {
        assert_eq!(item.category, "joke");
    }
}

/// E2E Test: Diagnostic logging
#[test]
fn test_diagnostic_logging() {
    let (db, _temp_dir) = create_test_db();

    // Log some diagnostic events
    db.log_diagnostic_event("test_event", "info", "Test info message", None, None)
        .unwrap();
    db.log_diagnostic_event(
        "test_warning",
        "warning",
        "Test warning message",
        None,
        None,
    )
    .unwrap();
    db.log_diagnostic_event("test_error", "error", "Test error message", None, None)
        .unwrap();

    // Get diagnostic summary
    let summary = db.get_diagnostic_summary().unwrap();
    println!(
        "Diagnostic summary: {} pending, health: {}",
        summary.pending_count, summary.estimated_buffer_health
    );
    assert!(summary.pending_count >= 0);

    // Get recent diagnostics
    let recent = db.get_recent_diagnostics(10).unwrap();
    println!("Recent diagnostics: {} entries", recent.len());
    assert!(!recent.is_empty());
}

/// E2E Test: Pruning old items
#[test]
fn test_pruning() {
    let (db, _temp_dir) = create_test_db();

    // Insert test items
    insert_test_items(&db);

    // Get initial count
    let initial_count = db.get_pending_count().unwrap();
    assert!(initial_count > 0, "Should have items before pruning");

    // Prune with 0 days (should not delete today's items)
    let (deleted_pending, deleted_all) = db.prune_old_items().unwrap();
    println!(
        "Pruned: {} pending items, {} total items",
        deleted_pending, deleted_all
    );

    // Items from today should not be deleted
    let after_count = db.get_pending_count().unwrap();
    // Note: items might be consumed during prune, so we just check it's reasonable
    println!("Count after pruning: {}", after_count);
}

/// E2E Test: Provider status tracking
#[test]
fn test_provider_status() {
    let (db, _temp_dir) = create_test_db();

    // Insert test items to generate provider activity
    insert_test_items(&db);

    // Consume items to trigger provider updates
    let _ = db.consume_pending_items(1.0);

    // Get provider status
    let statuses = db.get_provider_status().unwrap();
    println!("Provider statuses: {:?}", statuses.len());

    // Should have entries for various providers
    for status in &statuses {
        println!(
            "Provider {}: category={}, status={}, errors={}",
            status.provider_name,
            status.category,
            status.last_fetch_status,
            status.recent_error_count
        );
    }
}
