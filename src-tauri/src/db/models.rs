use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlItem {
    pub id: String,
    pub source: String,
    pub category: String,
    pub title: String,
    pub url: String,
    pub thumbnail_url: Option<String>,
    pub thumbnail_data: Option<String>,
    pub description: Option<String>,
    pub fetched_at: String,
    pub is_seen: bool,
    pub is_saved: bool,
    pub is_consumed: bool,
    pub session_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DayStats {
    pub memes_found: i64,
    pub jokes_found: i64,
    pub news_checked: i64,
    pub videos_found: i64,
    pub gossip_found: i64,
    pub total_items: i64,
    pub estimated_time_saved_minutes: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumeResult {
    pub items_consumed: i64,
    pub items_discarded: i64,
    pub time_consumed_minutes: f64,
    pub memes_consumed: i64,
    pub jokes_consumed: i64,
    pub news_consumed: i64,
    pub videos_consumed: i64,
    pub gossip_consumed: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaySummary {
    pub stats: DayStats,
    pub summary_text: String,
    pub highlights: Vec<CrawlItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticLog {
    pub id: String,
    pub timestamp: String,
    pub event_type: String,
    pub severity: String,
    pub message: String,
    pub metadata: Option<String>,
    pub related_item_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticSummary {
    pub pending_count: i64,
    pub estimated_buffer_health: String,
    pub budget_analysis: BudgetAnalysis,
    pub provider_scores: Vec<ProviderScoreSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetAnalysis {
    pub min_cost_per_item: f64,
    pub max_cost_per_item: f64,
    pub estimated_buffer_minutes: f64,
    pub total_pending_cost_minutes: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderStatus {
    pub provider_name: String,
    pub category: String,
    pub last_fetch_status: String,
    pub last_fetch_timestamp: Option<String>,
    pub recent_error_count: i64,
    pub score: f64,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClearDiagnosticsResult {
    pub deleted_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderScore {
    pub provider_name: String,
    pub total_fetches: i64,
    pub successful_fetches: i64,
    pub total_items_fetched: i64,
    pub last_fetch_at: Option<String>,
    pub last_success_at: Option<String>,
    pub consecutive_failures: i64,
    pub avg_items_per_fetch: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderScoreSummary {
    pub provider_name: String,
    pub score: f64,
    pub total_fetches: i64,
    pub success_rate: f64,
    pub avg_yield: f64,
}
