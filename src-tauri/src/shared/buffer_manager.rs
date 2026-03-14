use std::sync::Arc;
use crate::db::Database;
use crate::shared::consumption_calculator::ConsumptionCalculator;
use crate::commands::THREAD_COUNT;

pub struct BufferManager {
    db: Arc<Database>,
}

impl BufferManager {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Check if buffer needs replenishment and download if needed
    /// Returns the number of items downloaded
    pub async fn replenish_buffer_if_needed(&self) -> Result<i64, String> {
        let thread_count = THREAD_COUNT.load(std::sync::atomic::Ordering::Relaxed);
        let (base_buffer, download_target) = ConsumptionCalculator::calculate_buffer_requirements(thread_count, 2.0);
        let total_target = base_buffer + download_target;

        let pending_count = self.db.get_pending_count().map_err(|e| e.to_string())?;

        if pending_count < total_target {
            let items_to_download = total_target - pending_count;
            self.db.log_diagnostic_event(
                "buffer_manager",
                "info",
                &format!("Buffer low: {} items (target: {}), downloading {} more", pending_count, total_target, items_to_download),
                None,
                None
            ).map_err(|e| e.to_string())?;

            // Trigger download
            self.download_items(items_to_download).await?;

            let new_count = self.db.get_pending_count().map_err(|e| e.to_string())?;
            Ok(new_count - pending_count)
        } else {
            self.db.log_diagnostic_event(
                "buffer_manager",
                "info",
                &format!("Buffer sufficient: {} items available", pending_count),
                None,
                None
            ).map_err(|e| e.to_string())?;
            Ok(0)
        }
    }

    async fn download_items(&self, _count: i64) -> Result<(), String> {
        // This would integrate with the crawler/scheduler to download content
        self.db.log_diagnostic_event(
            "buffer_manager",
            "info",
            &format!("Would download {} items", _count),
            None,
            None
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Calculate how many items to consume based on elapsed time
    pub fn calculate_consumption_budget(elapsed_minutes: f64) -> f64 {
        let thread_count = THREAD_COUNT.load(std::sync::atomic::Ordering::Relaxed);
        ConsumptionCalculator::calculate_consumption(elapsed_minutes, thread_count)
    }
}
