use cazzmachine_lib::crawler::scoring::{
    compute_all_scores, compute_provider_score, select_providers_weighted,
};
use cazzmachine_lib::db::models::ProviderScore;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fresh_provider_score() {
        let stats = ProviderScore {
            provider_name: "test".to_string(),
            total_fetches: 0,
            successful_fetches: 0,
            total_items_fetched: 0,
            last_fetch_at: None,
            last_success_at: None,
            consecutive_failures: 0,
            avg_items_per_fetch: 0.0,
        };
        let score = compute_provider_score(&stats);
        assert!(
            (score - 0.2).abs() < 0.1,
            "Fresh provider score {} not near 0.2",
            score
        );
    }

    #[test]
    fn test_perfect_provider_score() {
        let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        let stats = ProviderScore {
            provider_name: "test".to_string(),
            total_fetches: 100,
            successful_fetches: 100,
            total_items_fetched: 1000,
            last_fetch_at: Some(now.clone()),
            last_success_at: Some(now),
            consecutive_failures: 0,
            avg_items_per_fetch: 10.0,
        };
        let score = compute_provider_score(&stats);
        assert!(score > 0.8, "Perfect provider score {} not > 0.8", score);
    }

    #[test]
    fn test_failed_provider_score() {
        let stats = ProviderScore {
            provider_name: "test".to_string(),
            total_fetches: 10,
            successful_fetches: 0,
            total_items_fetched: 0,
            last_fetch_at: Some("2024-01-01T00:00:00".to_string()),
            last_success_at: None,
            consecutive_failures: 10,
            avg_items_per_fetch: 0.0,
        };
        let score = compute_provider_score(&stats);
        assert!(
            (score - 0.1).abs() < 0.05,
            "Failed provider score {} not near 0.1",
            score
        );
    }

    #[test]
    fn test_recency_decay_score() {
        let seven_hours_ago = (chrono::Local::now() - chrono::Duration::hours(7))
            .format("%Y-%m-%dT%H:%M:%S")
            .to_string();
        let stats = ProviderScore {
            provider_name: "test".to_string(),
            total_fetches: 100,
            successful_fetches: 100,
            total_items_fetched: 500,
            last_fetch_at: Some(seven_hours_ago.clone()),
            last_success_at: Some(seven_hours_ago),
            consecutive_failures: 0,
            avg_items_per_fetch: 5.0,
        };
        let score = compute_provider_score(&stats);
        assert!(
            score < 0.7,
            "Decayed recency score {} should be < 0.7",
            score
        );
    }

    #[test]
    fn test_partial_success_rate() {
        let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        let stats = ProviderScore {
            provider_name: "test".to_string(),
            total_fetches: 100,
            successful_fetches: 50,
            total_items_fetched: 250,
            last_fetch_at: Some(now.clone()),
            last_success_at: Some(now),
            consecutive_failures: 0,
            avg_items_per_fetch: 2.5,
        };
        let score = compute_provider_score(&stats);
        assert!(
            score > 0.4 && score < 0.6,
            "Partial success score {} not in expected range",
            score
        );
    }

    #[test]
    fn test_recent_success_bonus() {
        let fifteen_min_ago = (chrono::Local::now() - chrono::Duration::minutes(15))
            .format("%Y-%m-%dT%H:%M:%S")
            .to_string();
        let stats = ProviderScore {
            provider_name: "test".to_string(),
            total_fetches: 50,
            successful_fetches: 50,
            total_items_fetched: 500,
            last_fetch_at: Some(fifteen_min_ago.clone()),
            last_success_at: Some(fifteen_min_ago),
            consecutive_failures: 0,
            avg_items_per_fetch: 10.0,
        };
        let score = compute_provider_score(&stats);
        assert!(
            score > 0.9,
            "Recent success score {} should be > 0.9",
            score
        );
    }

    #[test]
    fn test_score_clamped_max() {
        let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        let stats = ProviderScore {
            provider_name: "test".to_string(),
            total_fetches: 1000,
            successful_fetches: 1000,
            total_items_fetched: 10000,
            last_fetch_at: Some(now.clone()),
            last_success_at: Some(now),
            consecutive_failures: 0,
            avg_items_per_fetch: 100.0,
        };
        let score = compute_provider_score(&stats);
        assert!(score <= 1.0, "Score {} should be <= 1.0", score);
    }

    #[test]
    fn test_compute_all_scores() {
        let stats = vec![
            ProviderScore {
                provider_name: "a".to_string(),
                total_fetches: 10,
                successful_fetches: 10,
                total_items_fetched: 100,
                last_fetch_at: None,
                last_success_at: None,
                consecutive_failures: 0,
                avg_items_per_fetch: 10.0,
            },
            ProviderScore {
                provider_name: "b".to_string(),
                total_fetches: 10,
                successful_fetches: 0,
                total_items_fetched: 0,
                last_fetch_at: None,
                last_success_at: None,
                consecutive_failures: 10,
                avg_items_per_fetch: 0.0,
            },
        ];
        let scores = compute_all_scores(&stats);
        assert_eq!(scores.len(), 2, "Should return scores for all providers");
        assert!(
            scores[0].1 > scores[1].1,
            "High performer should have higher score"
        );
    }

    #[test]
    fn test_weighted_selection_empty() {
        let result = select_providers_weighted(&[], 5);
        assert!(result.is_empty(), "Empty input should return empty output");
    }

    #[test]
    fn test_weighted_selection_more_than_available() {
        let scores = vec![("a".to_string(), 0.5), ("b".to_string(), 0.5)];
        let result = select_providers_weighted(&scores, 10);
        assert_eq!(
            result.len(),
            2,
            "Should return all available when count > providers"
        );
    }

    #[test]
    fn test_weighted_selection_zero_scores() {
        let scores = vec![
            ("a".to_string(), 0.0),
            ("b".to_string(), 0.0),
            ("c".to_string(), 0.0),
        ];
        let mut counts = std::collections::HashMap::new();
        for _ in 0..300 {
            let result = select_providers_weighted(&scores, 1);
            if let Some(sel) = result.first() {
                *counts.entry(sel.clone()).or_insert(0) += 1;
            }
        }
        for (_, count) in counts {
            assert!(
                count > 50,
                "Each provider should be selected roughly equally, got {}",
                count
            );
        }
    }

    #[test]
    fn test_weighted_selection_distribution() {
        let scores = vec![("high".to_string(), 0.9), ("low".to_string(), 0.1)];
        let mut high_count = 0;
        for _ in 0..1000 {
            let result = select_providers_weighted(&scores, 1);
            if result.contains(&"high".to_string()) {
                high_count += 1;
            }
        }
        assert!(
            high_count > 600,
            "High-scored provider selected {} times, expected > 600",
            high_count
        );
    }

    #[test]
    fn test_weighted_selection_count_zero() {
        let scores = vec![("a".to_string(), 0.9), ("b".to_string(), 0.1)];
        let result = select_providers_weighted(&scores, 0);
        assert!(result.is_empty(), "Count=0 should return empty");
    }

    #[test]
    fn test_weighted_selection_without_replacement() {
        let scores = vec![
            ("a".to_string(), 0.33),
            ("b".to_string(), 0.33),
            ("c".to_string(), 0.34),
        ];
        let result = select_providers_weighted(&scores, 3);
        assert_eq!(result.len(), 3, "Should select exactly 3 providers");
        let unique: std::collections::HashSet<_> = result.iter().collect();
        assert_eq!(unique.len(), 3, "Should have no duplicates");
    }

    #[test]
    fn test_weighted_selection_probability_accuracy() {
        let scores = vec![
            ("a".to_string(), 0.6),
            ("b".to_string(), 0.3),
            ("c".to_string(), 0.1),
        ];

        let mut counts = std::collections::HashMap::new();
        counts.insert("a".to_string(), 0);
        counts.insert("b".to_string(), 0);
        counts.insert("c".to_string(), 0);

        for _ in 0..2000 {
            let result = select_providers_weighted(&scores, 1);
            if let Some(sel) = result.first() {
                *counts.get_mut(sel).unwrap() += 1;
            }
        }

        let a_pct = counts["a"] as f64 / 2000.0;
        let b_pct = counts["b"] as f64 / 2000.0;
        let c_pct = counts["c"] as f64 / 2000.0;

        assert!(
            (a_pct - 0.6).abs() < 0.15,
            "A selected {}% expected ~60%",
            a_pct * 100.0
        );
        assert!(
            (b_pct - 0.3).abs() < 0.15,
            "B selected {}% expected ~30%",
            b_pct * 100.0
        );
        assert!(
            (c_pct - 0.1).abs() < 0.10,
            "C selected {}% expected ~10%",
            c_pct * 100.0
        );
    }
}
