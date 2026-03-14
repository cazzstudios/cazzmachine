use crate::commands::THROTTLE_LEVEL;

pub struct ConsumptionCalculator;

impl ConsumptionCalculator {
    /// Calculate scroll duration in minutes based on throttle level (1-9)
    /// Formula: S(level) = 1 + 4 × (level-1)/8 minutes
    pub fn get_scroll_duration_minutes(level: u8) -> f64 {
        let level_f = level as f64;
        1.0 + 4.0 * ((level_f - 1.0) / 8.0)
    }

    /// Calculate active percentage based on throttle level (1-9)
    /// Formula: A(level) = 0.02 + 0.89 × (level-1)/8
    pub fn get_active_percentage(level: u8) -> f64 {
        let level_f = level as f64;
        0.02 + 0.89 * ((level_f - 1.0) / 8.0)
    }

    /// Calculate standby time in minutes
    /// Formula: W(level) = S × ((1/A) - 1)
    pub fn get_standby_minutes(level: u8) -> f64 {
        let scroll = Self::get_scroll_duration_minutes(level);
        let active_pct = Self::get_active_percentage(level);
        scroll * ((1.0 / active_pct) - 1.0)
    }

    /// Calculate total cycle time (scroll + standby) in minutes
    pub fn get_total_cycle_minutes(level: u8) -> f64 {
        Self::get_scroll_duration_minutes(level) + Self::get_standby_minutes(level)
    }

    /// Calculate items consumed per hour based on level
    /// This is an estimate based on active time percentage
    pub fn get_items_per_hour(level: u8) -> f64 {
        let scroll = Self::get_scroll_duration_minutes(level);
        let total_cycle = Self::get_total_cycle_minutes(level);
        let active_ratio = scroll / total_cycle;
        active_ratio * 60.0 // Items per hour if consuming continuously
    }

    /// Calculate items consumed per minute
    pub fn get_items_per_minute(level: u8) -> f64 {
        Self::get_items_per_hour(level) / 60.0
    }

    /// Calculate items consumed based on elapsed time and thread count
    /// Formula: items = elapsed_minutes × items_per_minute × thread_count
    pub fn calculate_consumption(elapsed_minutes: f64, thread_count: u8) -> f64 {
        let level = THROTTLE_LEVEL.load(std::sync::atomic::Ordering::Relaxed);
        let items_per_minute = Self::get_items_per_minute(level);
        elapsed_minutes * items_per_minute * (thread_count as f64)
    }

    /// Calculate download buffer requirements
    /// Returns (base_buffer, download_buffer) in items
    pub fn calculate_buffer_requirements(thread_count: u8, download_hours: f64) -> (i64, i64) {
        let level = THROTTLE_LEVEL.load(std::sync::atomic::Ordering::Relaxed);
        let items_per_hour = Self::get_items_per_hour(level);
        let threads = thread_count as i64;

        let base_buffer = 20 * threads; // 20 items per thread
        let download_target = (items_per_hour * download_hours * (threads as f64)) as i64;

        (base_buffer, download_target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scroll_duration_level_1() {
        let duration = ConsumptionCalculator::get_scroll_duration_minutes(1);
        assert!((duration - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_scroll_duration_level_9() {
        let duration = ConsumptionCalculator::get_scroll_duration_minutes(9);
        assert!((duration - 5.0).abs() < 0.02);
    }

    #[test]
    fn test_scroll_duration_level_5() {
        let duration = ConsumptionCalculator::get_scroll_duration_minutes(5);
        assert!((duration - 3.0).abs() < 0.02);
    }

    #[test]
    fn test_active_percentage_level_1() {
        let active = ConsumptionCalculator::get_active_percentage(1);
        assert!((active - 0.02).abs() < 0.02);
    }

    #[test]
    fn test_active_percentage_level_9() {
        let active = ConsumptionCalculator::get_active_percentage(9);
        assert!((active - 0.91).abs() < 0.02);
    }

    #[test]
    fn test_items_per_hour_positive() {
        let items = ConsumptionCalculator::get_items_per_hour(5);
        assert!(items > 0.0);
        assert!(items <= 60.0);
    }

    #[test]
    fn test_buffer_calculation() {
        let (base, download) = ConsumptionCalculator::calculate_buffer_requirements(4, 2.0);
        assert!(base > 0);
        assert!(download > 0);
        assert_eq!(base, 80); // 20 * 4 threads
    }
}
