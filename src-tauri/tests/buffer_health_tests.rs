//! Tests for buffer health computation
//!
//! Tests the compute_buffer_health function with various thread/level combinations

use cazzmachine_lib::db::Database;

#[test]
fn test_zero_pending_empty() {
    let health = Database::compute_buffer_health(0, 0.0, 1, 5);
    assert_eq!(health, "empty");
}

#[test]
fn test_one_thread_level_one_healthy() {
    // Level 1: scroll_minutes = 1 + 4*(0/8) = 1
    // 1 thread: demand = 1 * 1 = 1 minute
    // Need >= 1 minute of content to be healthy
    let health = Database::compute_buffer_health(5, 2.5, 1, 1); // 5 items * 0.5 min = 2.5 min > 1 min demand
    assert_eq!(health, "healthy");
}

#[test]
fn test_eight_threads_level_nine_critical() {
    // Level 9: scroll_minutes = 1 + 4*(8/8) = 5
    // 8 threads: demand = 5 * 8 = 40 minutes
    // Need >= 10 min (25% of 40) to not be critical
    let health = Database::compute_buffer_health(10, 5.0, 8, 9); // 10 items * 0.5 = 5 min < 10 min (25% of 40)
    assert_eq!(health, "critical");
}

#[test]
fn test_same_pending_different_health() {
    // Same 20 items = 10 minutes of content
    let health_low = Database::compute_buffer_health(20, 10.0, 1, 1); // demand = 1 min, 10 > 1 = healthy
    let health_high = Database::compute_buffer_health(20, 10.0, 8, 9); // demand = 40 min, 10 < 20 (50%) = low
    assert_eq!(health_low, "healthy");
    assert_eq!(health_high, "low");
}

#[test]
fn test_one_thread_level_five_moderate() {
    // Level 5: scroll_minutes = 1 + 4*(4/8) = 3
    // 1 thread: demand = 3 * 1 = 3 minutes
    // 2.5 min is between 50% (1.5) and 100% (3) = moderate
    let health = Database::compute_buffer_health(5, 2.5, 1, 5);
    assert_eq!(health, "moderate");
}

#[test]
fn test_four_threads_level_five_low() {
    // Level 5: scroll_minutes = 1 + 4*(4/8) = 3
    // 4 threads: demand = 3 * 4 = 12 minutes
    // 5 min is between 25% (3) and 50% (6) = low
    let health = Database::compute_buffer_health(10, 5.0, 4, 5);
    assert_eq!(health, "low");
}

#[test]
fn test_boundary_25_percent_critical() {
    // Level 1: scroll_minutes = 1
    // 4 threads: demand = 1 * 4 = 4 minutes
    // At exactly 25% (1 min): should be low, so 0.99 min should be critical
    let health = Database::compute_buffer_health(2, 0.99, 4, 1); // 0.99 < 1.0 (25% of 4)
    assert_eq!(health, "critical");
}

#[test]
fn test_boundary_50_percent_low() {
    // Level 1: scroll_minutes = 1
    // 4 threads: demand = 1 * 4 = 4 minutes
    // At exactly 50% (2 min): should be moderate, so 1.99 min should be low
    let health = Database::compute_buffer_health(4, 1.99, 4, 1); // 1.99 < 2.0 (50% of 4)
    assert_eq!(health, "low");
}

#[test]
fn test_boundary_100_percent_healthy() {
    // Level 1: scroll_minutes = 1
    // 4 threads: demand = 1 * 4 = 4 minutes
    // At exactly 100% (4 min): should be healthy
    let health = Database::compute_buffer_health(8, 4.0, 4, 1);
    assert_eq!(health, "healthy");
}

#[test]
fn test_empty_cost_at_level_one_returns_critical() {
    // Even with pending items, zero cost means critical
    // Level 1: scroll_minutes = 1
    // 1 thread: demand = 1 minute
    // 0 cost < 0.25 * 1 = 0.25
    let health = Database::compute_buffer_health(10, 0.0, 1, 1);
    assert_eq!(health, "critical");
}

#[test]
fn test_eight_threads_level_one_low() {
    // Level 1: scroll_minutes = 1
    // 8 threads: demand = 1 * 8 = 8 minutes
    // 3 min is between 25% (2) and 50% (4) = low
    let health = Database::compute_buffer_health(6, 3.0, 8, 1);
    assert_eq!(health, "low");
}

#[test]
fn test_one_thread_level_nine_moderate() {
    // Level 9: scroll_minutes = 5
    // 1 thread: demand = 5 * 1 = 5 minutes
    // 4 min is between 50% (2.5) and 100% (5) = moderate
    let health = Database::compute_buffer_health(8, 4.0, 1, 9);
    assert_eq!(health, "moderate");
}
