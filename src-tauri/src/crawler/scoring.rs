use crate::db::models::ProviderScore;

const MAX_EXPECTED_ITEMS: f64 = 10.0;
const RECENCY_FULL_BONUS_MINUTES: f64 = 30.0;
const RECENCY_ZERO_BONUS_MINUTES: f64 = 360.0; // 6 hours
const MIN_SCORE: f64 = 0.1;

/// Compute a health score [0.1, 1.0] for a provider based on its stats.
/// Score = (success_rate * 0.4) + (yield_normalized * 0.4) + (recency_bonus * 0.2)
pub fn compute_provider_score(stats: &ProviderScore) -> f64 {
    let success_rate = if stats.total_fetches == 0 {
        0.5 // neutral for new providers
    } else {
        stats.successful_fetches as f64 / stats.total_fetches as f64
    };

    let yield_normalized = (stats.avg_items_per_fetch / MAX_EXPECTED_ITEMS).min(1.0);

    let recency_bonus = compute_recency_bonus(stats.last_success_at.as_deref());

    let score = (success_rate * 0.4) + (yield_normalized * 0.4) + (recency_bonus * 0.2);
    score.clamp(MIN_SCORE, 1.0)
}

fn compute_recency_bonus(last_success_at: Option<&str>) -> f64 {
    let Some(ts) = last_success_at else {
        return 0.0;
    };
    let Ok(last) = chrono::NaiveDateTime::parse_from_str(ts, "%Y-%m-%dT%H:%M:%S") else {
        return 0.0;
    };
    let now = chrono::Local::now().naive_local();
    let minutes_ago = (now - last).num_minutes() as f64;

    if minutes_ago <= RECENCY_FULL_BONUS_MINUTES {
        1.0
    } else if minutes_ago >= RECENCY_ZERO_BONUS_MINUTES {
        0.0
    } else {
        // Linear decay from 1.0 to 0.0
        1.0 - (minutes_ago - RECENCY_FULL_BONUS_MINUTES)
            / (RECENCY_ZERO_BONUS_MINUTES - RECENCY_FULL_BONUS_MINUTES)
    }
}

/// Compute scores for all providers. Returns (provider_name, score) pairs.
pub fn compute_all_scores(stats: &[ProviderScore]) -> Vec<(String, f64)> {
    stats
        .iter()
        .map(|s| (s.provider_name.clone(), compute_provider_score(s)))
        .collect()
}

/// Weighted random selection without replacement.
/// Probability of selecting provider_i = score_i / sum(all_scores).
/// Returns up to `count` provider names.
pub fn select_providers_weighted(scores: &[(String, f64)], count: usize) -> Vec<String> {
    if scores.is_empty() || count == 0 {
        return vec![];
    }

    let mut pool: Vec<(String, f64)> = scores.to_vec();
    let mut selected = Vec::new();

    for _ in 0..count {
        if pool.is_empty() {
            break;
        }

        let total: f64 = pool.iter().map(|(_, s)| s).sum();

        if total <= 0.0 {
            // Fallback: uniform random
            let idx = rand::random::<usize>() % pool.len();
            selected.push(pool.remove(idx).0);
            continue;
        }

        let mut r = rand::random::<f64>() * total;
        let mut chosen_idx = pool.len() - 1; // fallback to last
        for (i, (_, score)) in pool.iter().enumerate() {
            r -= score;
            if r <= 0.0 {
                chosen_idx = i;
                break;
            }
        }
        selected.push(pool.remove(chosen_idx).0);
    }

    selected
}
