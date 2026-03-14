pub mod models;

use crate::crawler::scoring;
use rusqlite::{Connection, Result as SqlResult};
use std::path::PathBuf;
use std::sync::Mutex;

pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    pub fn new(app_dir: PathBuf) -> SqlResult<Self> {
        std::fs::create_dir_all(&app_dir).ok();
        let db_path = app_dir.join("cazzmachine.db");
        let conn = Connection::open(db_path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;

        let db = Database {
            conn: Mutex::new(conn),
        };
        db.run_migrations()?;
        Ok(db)
    }

    fn run_migrations(&self) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute_batch(r#"
            CREATE TABLE IF NOT EXISTS crawl_items (
                id TEXT PRIMARY KEY,
                source TEXT NOT NULL,
                category TEXT NOT NULL,
                title TEXT NOT NULL,
                url TEXT NOT NULL UNIQUE,
                thumbnail_url TEXT,
                thumbnail_data TEXT,
                description TEXT,
                fetched_at TEXT NOT NULL DEFAULT (datetime('now')),
                is_seen INTEGER NOT NULL DEFAULT 0,
                is_saved INTEGER NOT NULL DEFAULT 0,
                is_consumed INTEGER NOT NULL DEFAULT 0,
                session_date TEXT NOT NULL DEFAULT (date('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_crawl_items_session ON crawl_items(session_date);
            CREATE INDEX IF NOT EXISTS idx_crawl_items_category ON crawl_items(category);
            CREATE INDEX IF NOT EXISTS idx_crawl_items_consumed ON crawl_items(is_consumed);
            
            CREATE TABLE IF NOT EXISTS session_stats (
                id TEXT PRIMARY KEY,
                date TEXT NOT NULL UNIQUE,
                started_at TEXT NOT NULL,
                memes_found INTEGER NOT NULL DEFAULT 0,
                jokes_found INTEGER NOT NULL DEFAULT 0,
                news_checked INTEGER NOT NULL DEFAULT 0,
                videos_found INTEGER NOT NULL DEFAULT 0,
                gossip_found INTEGER NOT NULL DEFAULT 0,
                total_items INTEGER NOT NULL DEFAULT 0,
                estimated_time_saved_minutes REAL NOT NULL DEFAULT 0.0
            );
            
            CREATE TABLE IF NOT EXISTS notification_log (
                id TEXT PRIMARY KEY,
                message TEXT NOT NULL,
                sent_at TEXT NOT NULL DEFAULT (datetime('now')),
                related_item_id TEXT,
                FOREIGN KEY (related_item_id) REFERENCES crawl_items(id)
            );
            
            CREATE TABLE IF NOT EXISTS app_state (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            INSERT OR IGNORE INTO app_state (key, value) VALUES ('last_active_timestamp', strftime('%s', 'now') * 1000);
            
            CREATE TABLE IF NOT EXISTS diagnostic_logs (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                event_type TEXT NOT NULL,
                severity TEXT NOT NULL,
                message TEXT NOT NULL,
                related_item_id TEXT,
                metadata TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_diagnostic_logs_severity ON diagnostic_logs(severity);
            CREATE INDEX IF NOT EXISTS idx_diagnostic_logs_event ON diagnostic_logs(event_type);
            CREATE INDEX IF NOT EXISTS idx_diagnostic_logs_event ON diagnostic_logs(event_type);
            
            CREATE TABLE IF NOT EXISTS provider_stats (
                provider_name TEXT PRIMARY KEY,
                total_fetches INTEGER NOT NULL DEFAULT 0,
                successful_fetches INTEGER NOT NULL DEFAULT 0,
                total_items_fetched INTEGER NOT NULL DEFAULT 0,
                last_fetch_at TEXT,
                last_success_at TEXT,
                consecutive_failures INTEGER NOT NULL DEFAULT 0,
                avg_items_per_fetch REAL NOT NULL DEFAULT 0.0
            );
        "#)?;

        Ok(())
    }

    pub fn insert_item(&self, item: &models::CrawlItem) -> SqlResult<bool> {
        let conn = self.conn.lock().unwrap();
        let result = conn.execute(
            "INSERT OR IGNORE INTO crawl_items (id, source, category, title, url, thumbnail_url, thumbnail_data, description, fetched_at, session_date)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                item.id,
                item.source,
                item.category,
                item.title,
                item.url,
                item.thumbnail_url,
                item.thumbnail_data,
                item.description,
                item.fetched_at,
                item.session_date,
            ],
        );
        match result {
            Ok(rows) => Ok(rows > 0),
            Err(e) => Err(e),
        }
    }

    fn row_to_item(row: &rusqlite::Row) -> rusqlite::Result<models::CrawlItem> {
        Ok(models::CrawlItem {
            id: row.get(0)?,
            source: row.get(1)?,
            category: row.get(2)?,
            title: row.get(3)?,
            url: row.get(4)?,
            thumbnail_url: row.get(5)?,
            thumbnail_data: row.get(6)?,
            description: row.get(7)?,
            fetched_at: row.get(8)?,
            is_seen: row.get(9)?,
            is_saved: row.get(10)?,
            is_consumed: row.get(11)?,
            session_date: row.get(12)?,
        })
    }

    const ITEM_COLUMNS: &'static str =
        "id, source, category, title, url, thumbnail_url, thumbnail_data, description, fetched_at, is_seen, is_saved, is_consumed, session_date";

    pub fn get_items_for_today(&self) -> SqlResult<Vec<models::CrawlItem>> {
        let conn = self.conn.lock().unwrap();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let sql = format!(
            "SELECT {} FROM crawl_items WHERE session_date = ?1 AND is_consumed = 1 ORDER BY fetched_at DESC",
            Self::ITEM_COLUMNS
        );
        let mut stmt = conn.prepare(&sql)?;
        let items = stmt
            .query_map(rusqlite::params![today], Self::row_to_item)?
            .collect::<SqlResult<Vec<_>>>()?;
        Ok(items)
    }

    pub fn get_items_by_category(&self, category: &str) -> SqlResult<Vec<models::CrawlItem>> {
        let conn = self.conn.lock().unwrap();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let sql = format!(
            "SELECT {} FROM crawl_items WHERE session_date = ?1 AND category = ?2 AND is_consumed = 1 ORDER BY fetched_at DESC",
            Self::ITEM_COLUMNS
        );
        let mut stmt = conn.prepare(&sql)?;
        let items = stmt
            .query_map(rusqlite::params![today, category], Self::row_to_item)?
            .collect::<SqlResult<Vec<_>>>()?;
        Ok(items)
    }

    pub fn get_today_stats(&self) -> SqlResult<models::DayStats> {
        let conn = self.conn.lock().unwrap();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let mut stmt = conn.prepare(
            "SELECT category, COUNT(*) FROM crawl_items WHERE session_date = ?1 AND is_consumed = 1 GROUP BY category",
        )?;
        let mut stats = models::DayStats::default();
        let rows = stmt.query_map(rusqlite::params![today], |row| {
            let category: String = row.get(0)?;
            let count: i64 = row.get(1)?;
            Ok((category, count))
        })?;
        for row in rows {
            let (category, count) = row?;
            match category.as_str() {
                "meme" => stats.memes_found = count,
                "joke" => stats.jokes_found = count,
                "news" => stats.news_checked = count,
                "video" => stats.videos_found = count,
                "gossip" => stats.gossip_found = count,
                _ => {}
            }
            stats.total_items += count;
        }
        stats.estimated_time_saved_minutes = Self::estimate_time_saved(&stats);
        Ok(stats)
    }

    pub fn consume_pending_items(&self, budget_minutes: f64) -> SqlResult<models::ConsumeResult> {
        let conn = self.conn.lock().unwrap();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();

        let mut stmt = conn.prepare(
            "SELECT id, category FROM crawl_items WHERE session_date = ?1 AND is_consumed = 0 ORDER BY fetched_at ASC",
        )?;
        let pending: Vec<(String, String)> = stmt
            .query_map(rusqlite::params![today], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?
            .collect::<SqlResult<Vec<_>>>()?;

        let pending_count = pending.len();
        Self::log_diagnostic_with_conn(
            &conn,
            "consume_start",
            "info",
            &format!(
                "Starting consumption: budget={:.2}min, pending_items={}",
                budget_minutes, pending_count
            ),
            None,
            None,
        )?;

        let total_pending_cost: f64 = pending
            .iter()
            .map(|(_, cat)| Self::category_cost(cat))
            .sum();
        let min_item_cost = pending
            .iter()
            .map(|(_, cat)| Self::category_cost(cat))
            .fold(f64::INFINITY, f64::min);
        let estimated_max_items = if min_item_cost.is_finite() {
            (budget_minutes / min_item_cost).floor() as i64
        } else {
            0
        };

        Self::log_diagnostic_with_conn(
            &conn,
            "budget_analysis",
            "info",
            &format!(
                "Budget analysis: total_pending_cost={:.2}min, min_item_cost={:.2}min, estimated_max_items={}",
                total_pending_cost,
                if min_item_cost.is_finite() { min_item_cost } else { 0.0 },
                estimated_max_items
            ),
            None,
            None,
        )?;

        if pending_count == 0 {
            Self::log_diagnostic_with_conn(
                &conn,
                "consume_empty",
                "warn",
                "items_consumed=0, reason='empty_buffer' - No pending items in database",
                None,
                None,
            )?;
        } else if budget_minutes < 0.3 {
            Self::log_diagnostic_with_conn(
                &conn,
                "consume_empty",
                "warn",
                &format!("items_consumed=0, reason='budget_too_small' - Budget ({:.2}min) smaller than minimum category cost (0.3min)", budget_minutes),
                None,
                None,
            )?;
        } else if pending
            .iter()
            .all(|(_, cat)| Self::category_cost(cat) > budget_minutes)
        {
            Self::log_diagnostic_with_conn(
                &conn,
                "consume_empty",
                "warn",
                &format!("items_consumed=0, reason='all_items_too_expensive' - {} pending items exist but all cost more than budget ({:.2}min)", pending_count, budget_minutes),
                None,
                None,
            )?;
        }

        let mut remaining_budget = budget_minutes;
        let mut consumed_ids: Vec<String> = Vec::new();
        let mut time_consumed = 0.0_f64;
        let mut memes: i64 = 0;
        let mut jokes: i64 = 0;
        let mut news: i64 = 0;
        let mut videos: i64 = 0;
        let mut gossip: i64 = 0;

        for (id, category) in &pending {
            let cost = Self::category_cost(category);
            if remaining_budget >= cost {
                consumed_ids.push(id.clone());
                remaining_budget -= cost;
                time_consumed += cost;
                match category.as_str() {
                    "meme" => memes += 1,
                    "joke" => jokes += 1,
                    "news" => news += 1,
                    "video" => videos += 1,
                    "gossip" => gossip += 1,
                    _ => {}
                }
            }
            // Unconsumed items stay in buffer for next phase - no deletion
        }

        for id in &consumed_ids {
            conn.execute(
                "UPDATE crawl_items SET is_consumed = 1 WHERE id = ?1",
                rusqlite::params![id],
            )?;
        }

        let unconsumed_count = pending.len() as i64 - consumed_ids.len() as i64;

        Ok(models::ConsumeResult {
            items_consumed: consumed_ids.len() as i64,
            items_discarded: unconsumed_count,
            time_consumed_minutes: time_consumed,
            memes_consumed: memes,
            jokes_consumed: jokes,
            news_consumed: news,
            videos_consumed: videos,
            gossip_consumed: gossip,
        })
    }

    fn category_cost(category: &str) -> f64 {
        match category {
            "meme" => 0.5,
            "joke" => 0.3,
            "news" => 2.0,
            "video" => 3.0,
            "gossip" => 1.5,
            _ => 1.0,
        }
    }

    pub fn mark_item_seen(&self, item_id: &str) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE crawl_items SET is_seen = 1 WHERE id = ?1",
            rusqlite::params![item_id],
        )?;
        Ok(())
    }

    pub fn toggle_item_saved(&self, item_id: &str) -> SqlResult<bool> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE crawl_items SET is_saved = CASE WHEN is_saved = 0 THEN 1 ELSE 0 END WHERE id = ?1",
            rusqlite::params![item_id],
        )?;
        let saved: bool = conn.query_row(
            "SELECT is_saved FROM crawl_items WHERE id = ?1",
            rusqlite::params![item_id],
            |row| row.get(0),
        )?;
        Ok(saved)
    }

    pub fn get_latest_unseen_item(&self) -> SqlResult<Option<models::CrawlItem>> {
        let conn = self.conn.lock().unwrap();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let sql = format!(
            "SELECT {} FROM crawl_items WHERE session_date = ?1 AND is_seen = 0 AND is_consumed = 1 ORDER BY fetched_at DESC LIMIT 1",
            Self::ITEM_COLUMNS
        );
        let mut stmt = conn.prepare(&sql)?;
        let mut items = stmt.query_map(rusqlite::params![today], Self::row_to_item)?;
        match items.next() {
            Some(Ok(item)) => Ok(Some(item)),
            _ => Ok(None),
        }
    }

    fn estimate_time_saved(stats: &models::DayStats) -> f64 {
        // Average time a human spends per content type (in minutes)
        let meme_time = 0.5; // 30 sec per meme
        let joke_time = 0.3; // 20 sec per joke
        let news_time = 2.0; // 2 min per news article
        let video_time = 3.0; // 3 min per video
        let gossip_time = 1.5; // 1.5 min per gossip item

        (stats.memes_found as f64 * meme_time)
            + (stats.jokes_found as f64 * joke_time)
            + (stats.news_checked as f64 * news_time)
            + (stats.videos_found as f64 * video_time)
            + (stats.gossip_found as f64 * gossip_time)
    }

    /// Internal helper that logs a diagnostic event using an already-acquired connection.
    /// Use this when you already hold the mutex lock to avoid self-deadlock.
    fn log_diagnostic_with_conn(
        conn: &Connection,
        event_type: &str,
        severity: &str,
        message: &str,
        metadata: Option<&str>,
        related_item_id: Option<&str>,
    ) -> SqlResult<()> {
        let id = uuid::Uuid::new_v4().to_string();
        let timestamp = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        conn.execute(
            "INSERT INTO diagnostic_logs (id, timestamp, event_type, severity, message, metadata, related_item_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![id, timestamp, event_type, severity, message, metadata, related_item_id],
        )?;
        Ok(())
    }

    pub fn log_diagnostic_event(
        &self,
        event_type: &str,
        severity: &str,
        message: &str,
        metadata: Option<&str>,
        related_item_id: Option<&str>,
    ) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        Self::log_diagnostic_with_conn(
            &conn,
            event_type,
            severity,
            message,
            metadata,
            related_item_id,
        )
    }

    pub fn get_pending_count(&self) -> SqlResult<i64> {
        let conn = self.conn.lock().unwrap();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM crawl_items WHERE session_date = ?1 AND is_consumed = 0",
            rusqlite::params![today],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    pub fn prune_old_items(&self) -> SqlResult<(i64, i64)> {
        let conn = self.conn.lock().unwrap();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();

        let deleted = conn.execute(
            "DELETE FROM crawl_items WHERE session_date < ?1 AND is_consumed = 0",
            rusqlite::params![today],
        )?;

        let stripped = conn.execute(
            "UPDATE crawl_items SET title = '[ARCHIVED]', description = NULL, thumbnail_url = NULL, is_seen = 1 WHERE session_date < ?1 AND is_consumed = 1",
            rusqlite::params![today],
        )?;
        
        conn.execute("DELETE FROM provider_stats", [])?;

        Ok((deleted as i64, stripped as i64))
    }

    pub fn get_last_active_timestamp(&self) -> SqlResult<chrono::DateTime<chrono::Utc>> {
        let conn = self.conn.lock().unwrap();

        // Try to get from app_state table first
        let result: Option<i64> = conn
            .query_row(
                "SELECT value FROM app_state WHERE key = 'last_active_timestamp'",
                [],
                |row| row.get(0),
            )
            .ok();

        if let Some(timestamp) = result {
            Ok(chrono::DateTime::from_timestamp_millis(timestamp).unwrap_or_else(chrono::Utc::now))
        } else {
            // Return current time if not set
            Ok(chrono::Utc::now())
        }
    }

    pub fn set_last_active_timestamp(
        &self,
        timestamp: chrono::DateTime<chrono::Utc>,
    ) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        let timestamp_millis = timestamp.timestamp_millis();

        conn.execute(
            "INSERT OR REPLACE INTO app_state (key, value) VALUES ('last_active_timestamp', ?1)",
            rusqlite::params![timestamp_millis],
        )?;

        Ok(())
    }

    pub fn get_diagnostic_summary(&self) -> SqlResult<models::DiagnosticSummary> {
        let conn = self.conn.lock().unwrap();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();

        let pending_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM crawl_items WHERE session_date = ?1 AND is_consumed = 0",
            rusqlite::params![today],
            |row| row.get(0),
        )?;

        let mut stmt = conn.prepare(
            "SELECT category FROM crawl_items WHERE session_date = ?1 AND is_consumed = 0",
        )?;
        let categories: Vec<String> = stmt
            .query_map(rusqlite::params![today], |row| row.get::<_, String>(0))?
            .collect::<SqlResult<Vec<_>>>()?;

        let mut total_cost: f64 = 0.0;
        let mut min_cost = f64::INFINITY;
        let mut max_cost: f64 = 0.0;

        for category in &categories {
            let cost = Self::category_cost(category);
            total_cost += cost;
            min_cost = min_cost.min(cost);
            max_cost = max_cost.max(cost);
        }

        if min_cost.is_infinite() {
            min_cost = 0.0;
        }

        let thread_count = crate::commands::THREAD_COUNT.load(std::sync::atomic::Ordering::Relaxed);
        let throttle_level = crate::commands::THROTTLE_LEVEL.load(std::sync::atomic::Ordering::Relaxed);
        let health = Self::compute_buffer_health(pending_count, total_cost, thread_count, throttle_level);

        // Get provider stats and compute scores
        let stats = Self::get_provider_stats_with_conn(&conn)?;
        let scores = scoring::compute_all_scores(&stats);
        let provider_scores: Vec<models::ProviderScoreSummary> = stats.iter().map(|s| {
            let score = scores.iter()
                .find(|(name, _)| name == &s.provider_name)
                .map(|(_, score)| *score)
                .unwrap_or(0.1);
            let success_rate = if s.total_fetches > 0 {
                s.successful_fetches as f64 / s.total_fetches as f64
            } else {
                0.0
            };
            models::ProviderScoreSummary {
                provider_name: s.provider_name.clone(),
                score,
                total_fetches: s.total_fetches,
                success_rate,
                avg_yield: s.avg_items_per_fetch,
            }
        }).collect();

        Ok(models::DiagnosticSummary {
            pending_count,
            estimated_buffer_health: health,
            budget_analysis: models::BudgetAnalysis {
                min_cost_per_item: min_cost,
                max_cost_per_item: max_cost,
                estimated_buffer_minutes: total_cost,
                total_pending_cost_minutes: total_cost,
            },
            provider_scores,
        })
    }

    pub fn compute_buffer_health(pending_count: i64, total_cost: f64, thread_count: u8, throttle_level: u8) -> String {
        if pending_count == 0 {
            return "empty".to_string();
        }
        
        let scroll_minutes = 1.0 + 4.0 * ((throttle_level as f64 - 1.0) / 8.0);
        let demand_minutes = scroll_minutes * thread_count as f64;
        
        // Avoid division by zero for edge cases
        let demand_minutes = demand_minutes.max(1.0);
        
        if total_cost < demand_minutes * 0.25 {
            "critical".to_string()
        } else if total_cost < demand_minutes * 0.50 {
            "low".to_string()
        } else if total_cost < demand_minutes * 1.0 {
            "moderate".to_string()
        } else {
            "healthy".to_string()
        }
    }

    pub fn get_recent_diagnostics(&self, limit: i64) -> SqlResult<Vec<models::DiagnosticLog>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, event_type, severity, message, metadata, related_item_id 
             FROM diagnostic_logs 
             ORDER BY timestamp DESC 
             LIMIT ?1",
        )?;

        let logs = stmt
            .query_map(rusqlite::params![limit], |row| {
                Ok(models::DiagnosticLog {
                    id: row.get(0)?,
                    timestamp: row.get(1)?,
                    event_type: row.get(2)?,
                    severity: row.get(3)?,
                    message: row.get(4)?,
                    metadata: row.get(5)?,
                    related_item_id: row.get(6)?,
                })
            })?
            .collect::<SqlResult<Vec<_>>>()?;

        Ok(logs)
    }

    pub fn clear_diagnostics(&self, older_than_days: i64) -> SqlResult<i64> {
        let conn = self.conn.lock().unwrap();
        let deleted = if older_than_days <= 0 {
            // 0 or negative means delete all logs
            conn.execute("DELETE FROM diagnostic_logs", [])?
        } else {
            conn.execute(
                "DELETE FROM diagnostic_logs WHERE timestamp < datetime('now', '-' || ?1 || ' days')",
                rusqlite::params![older_than_days],
            )?
        };
        Ok(deleted as i64)
    }

    pub fn upsert_provider_stat(&self, provider_name: &str, items_fetched: usize, success: bool) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        
        // Get current row or defaults
        let (total_fetches, successful_fetches, total_items_fetched, consecutive_failures) = 
            conn.query_row(
                "SELECT total_fetches, successful_fetches, total_items_fetched, consecutive_failures FROM provider_stats WHERE provider_name = ?1",
                rusqlite::params![provider_name],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?, row.get::<_, i64>(2)?, row.get::<_, i64>(3)?))
            ).unwrap_or((0, 0, 0, 0));
        
        let new_total_fetches = total_fetches + 1;
        let new_successful_fetches = if success { successful_fetches + 1 } else { successful_fetches };
        let new_total_items = total_items_fetched + items_fetched as i64;
        let new_consecutive_failures = if success { 0 } else { consecutive_failures + 1 };
        let new_avg = if new_total_fetches > 0 {
            new_total_items as f64 / new_total_fetches as f64
        } else {
            0.0
        };
        
        let last_success_at = if success { Some(now.clone()) } else { None };
        
        conn.execute(
            "INSERT OR REPLACE INTO provider_stats (provider_name, total_fetches, successful_fetches, total_items_fetched, last_fetch_at, last_success_at, consecutive_failures, avg_items_per_fetch) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![provider_name, new_total_fetches, new_successful_fetches, new_total_items, now, last_success_at, new_consecutive_failures, new_avg],
        )?;
        
        Ok(())
    }
    
    pub fn get_provider_stats(&self) -> SqlResult<Vec<models::ProviderScore>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT provider_name, total_fetches, successful_fetches, total_items_fetched, last_fetch_at, last_success_at, consecutive_failures, avg_items_per_fetch FROM provider_stats"
        )?;
        
        let scores = stmt.query_map([], |row| {
            Ok(models::ProviderScore {
                provider_name: row.get(0)?,
                total_fetches: row.get(1)?,
                successful_fetches: row.get(2)?,
                total_items_fetched: row.get(3)?,
                last_fetch_at: row.get(4)?,
                last_success_at: row.get(5)?,
                consecutive_failures: row.get(6)?,
                avg_items_per_fetch: row.get(7)?,
            })
        })?.collect::<SqlResult<Vec<_>>>()?;
        
        Ok(scores)
    }

    fn get_provider_stats_with_conn(conn: &Connection) -> SqlResult<Vec<models::ProviderScore>> {
        let mut stmt = conn.prepare(
            "SELECT provider_name, total_fetches, successful_fetches, total_items_fetched, last_fetch_at, last_success_at, consecutive_failures, avg_items_per_fetch FROM provider_stats"
        )?;
        
        let scores = stmt.query_map([], |row| {
            Ok(models::ProviderScore {
                provider_name: row.get(0)?,
                total_fetches: row.get(1)?,
                successful_fetches: row.get(2)?,
                total_items_fetched: row.get(3)?,
                last_fetch_at: row.get(4)?,
                last_success_at: row.get(5)?,
                consecutive_failures: row.get(6)?,
                avg_items_per_fetch: row.get(7)?,
            })
        })?.collect::<SqlResult<Vec<_>>>()?;
        
        Ok(scores)
    }
    
    pub fn reset_provider_stats(&self) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM provider_stats", [])?;
        Ok(())
    }

    pub fn get_provider_status(&self) -> SqlResult<Vec<models::ProviderStatus>> {
        let conn = self.conn.lock().unwrap();
        let events = Self::fetch_recent_crawl_events(&conn)?;
        let statuses = Self::derive_provider_statuses(&conn, &events)?;
        Ok(statuses)
    }

    fn fetch_recent_crawl_events(
        conn: &Connection,
    ) -> SqlResult<Vec<(String, String, String, String)>> {
        let sql = "SELECT
                event_type,
                message,
                timestamp,
                severity
             FROM diagnostic_logs
             WHERE event_type IN ('crawl_start', 'crawl_success', 'crawl_error', 'provider_fetch')
             AND timestamp > datetime('now', '-1 day')
             ORDER BY timestamp DESC";

        let mut stmt = conn.prepare(sql)?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })?;

        let events: SqlResult<Vec<_>> = rows.collect();
        events
    }

    fn derive_provider_statuses(
        conn: &Connection,
        events: &[(String, String, String, String)],
    ) -> SqlResult<Vec<models::ProviderStatus>> {
        let provider_names = vec![
            ("google-news", "news"),
            ("dadjokes", "joke"),
            ("reddit-memes", "meme"),
            ("entertainment", "gossip"),
            ("memes", "meme"),
            ("gossip", "gossip"),
            ("reddit-videos", "video"),
            ("icanhazdadjoke", "joke"),
            ("jokeapi", "joke"),
            ("uselessfacts", "joke"),
            ("chucknorris", "joke"),
            ("hackernews", "news"),
            ("bbc-news", "news"),
        ];

        let mut statuses: Vec<models::ProviderStatus> = Vec::new();

        for (name, category) in provider_names {
            let status = Self::derive_single_provider_status(conn, events, name, category)?;
            statuses.push(status);
        }

        Ok(statuses)
    }

    fn derive_single_provider_status(
        conn: &Connection,
        events: &[(String, String, String, String)],
        name: &str,
        category: &str,
    ) -> SqlResult<models::ProviderStatus> {
        let mut last_status = "unknown".to_string();
        let mut last_timestamp: Option<String> = None;
        let mut error_count = 0;

        for (event_type, message, timestamp, severity) in events {
            if message.to_lowercase().contains(name) {
                if last_timestamp.is_none() {
                    last_timestamp = Some(timestamp.clone());
                }

                if event_type == "crawl_success" {
                    last_status = "ok".to_string();
                    break;
                } else if severity == "error" {
                    error_count += 1;
                    if last_status != "ok" {
                        last_status = "error".to_string();
                    }
                }
            }
        }

        if last_status == "unknown" {
            let today = chrono::Local::now().format("%Y-%m-%d").to_string();
            let has_items: bool = conn.query_row(
                "SELECT EXISTS(SELECT 1 FROM crawl_items WHERE session_date = ?1 AND category = ?2 LIMIT 1)",
                rusqlite::params![today, category],
                |row| row.get(0),
            ).unwrap_or(false);

            if has_items {
                last_status = "ok".to_string();
                last_timestamp = Some(today);
            }
        }

        // Compute score from provider_stats
        let score = conn.query_row(
            "SELECT total_fetches, successful_fetches, total_items_fetched, last_success_at FROM provider_stats WHERE provider_name = ?1",
            rusqlite::params![name],
            |row| {
                let total_fetches: i64 = row.get(0)?;
                let successful_fetches: i64 = row.get(1)?;
                let total_items: i64 = row.get(2)?;
                let last_success_at: Option<String> = row.get(3)?;
                
                let stats = models::ProviderScore {
                    provider_name: name.to_string(),
                    total_fetches,
                    successful_fetches,
                    total_items_fetched: total_items,
                    last_fetch_at: None,
                    last_success_at,
                    consecutive_failures: 0,
                    avg_items_per_fetch: if total_fetches > 0 { total_items as f64 / total_fetches as f64 } else { 0.0 },
                };
                Ok(scoring::compute_provider_score(&stats))
            },
        ).unwrap_or(0.1); // Default score if no stats yet

        Ok(models::ProviderStatus {
            provider_name: name.to_string(),
            category: category.to_string(),
            last_fetch_status: last_status,
            last_fetch_timestamp: last_timestamp,
            recent_error_count: error_count,
            score,
        })

    }
}
