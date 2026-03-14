import { invoke, isTauri } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-shell";

export interface CrawlItem {
  id: string;
  source: string;
  category: string;
  title: string;
  url: string;
  thumbnail_url: string | null;
  thumbnail_data: string | null;
  description: string | null;
  fetched_at: string;
  is_seen: boolean;
  is_saved: boolean;
  session_date: string;
}

export interface DayStats {
  memes_found: number;
  jokes_found: number;
  news_checked: number;
  videos_found: number;
  gossip_found: number;
  total_items: number;
  estimated_time_saved_minutes: number;
}

export interface DaySummary {
  stats: DayStats;
  summary_text: string;
  highlights: CrawlItem[];
}

export interface ConsumeResult {
  items_consumed: number;
  items_discarded: number;
  time_consumed_minutes: number;
  memes_consumed: number;
  jokes_consumed: number;
  news_consumed: number;
  videos_consumed: number;
  gossip_consumed: number;
}

export interface DiagnosticLog {
  id: string;
  timestamp: string;
  event_type: string;
  severity: string;
  message: string;
  metadata: string | null;
  related_item_id: string | null;
}

export interface BudgetAnalysis {
  min_cost_per_item: number;
  max_cost_per_item: number;
  estimated_buffer_minutes: number;
  total_pending_cost_minutes: number;
}

export interface DiagnosticSummary {
  pending_count: number;
  estimated_buffer_health: string;
  budget_analysis: BudgetAnalysis;
}

export interface ProviderStatus {
  provider_name: string;
  category: string;
  last_fetch_status: string;
  last_fetch_timestamp: string | null;
  recent_error_count: number;
  score: number;
}

export interface ClearDiagnosticsResult {
  deleted_count: number;
}

export type Category = "meme" | "joke" | "news" | "video" | "gossip";

export async function getTodayItems(): Promise<CrawlItem[]> {
  return invoke<CrawlItem[]>("get_today_items");
}

export async function getItemsByCategory(
  category: Category,
): Promise<CrawlItem[]> {
  return invoke<CrawlItem[]>("get_items_by_category", { category });
}

export async function getTodayStats(): Promise<DayStats> {
  return invoke<DayStats>("get_today_stats");
}

export async function getDailySummary(): Promise<DaySummary> {
  return invoke<DaySummary>("get_daily_summary");
}

export async function toggleSaveItem(itemId: string): Promise<boolean> {
  return invoke<boolean>("toggle_save_item", { itemId });
}

export async function markItemSeen(itemId: string): Promise<void> {
  return invoke<void>("mark_item_seen", { itemId });
}

export async function fetchImage(url: string): Promise<string> {
  return invoke<string>("fetch_image", { url });
}

declare global {
  interface Window {
    Tauri?: any;
  }
}

export async function openUrl(url: string): Promise<void> {
  try {
    await open(url);
  } catch (e) {
    void logDiagnosticInternal("open_url_error", "warn", "Failed to open URL with shell plugin", { error: String(e), url });

    if (typeof window !== 'undefined' && window.Tauri) {
      try {
        await window.Tauri.invoke('open_url', { url });
      } catch (backendError) {
        void logDiagnosticInternal("open_url_fallback_error", "error", "Failed to open URL with backend command", { error: String(backendError), url });

        if (url.startsWith('http')) {
          window.open(url, '_blank');
        }
      }
    } else {
      if (url.startsWith('http')) {
        window.open(url, '_blank');
      }
    }
  }
}

export async function getThrottleLevel(): Promise<number> {
  return invoke<number>("get_throttle_level");
}

export async function setThrottleLevel(level: number): Promise<void> {
  return invoke<void>("set_throttle_level", { level });
}

export async function getConsumptionThreads(): Promise<number> {
  return invoke<number>("get_consumption_threads");
}

export async function setConsumptionThreads(count: number): Promise<void> {
  return invoke<void>("set_consumption_threads", { count });
}

export async function getPendingCount(): Promise<number> {
  return invoke<number>("get_pending_count");
}

export async function consumePendingItems(budgetMinutes: number): Promise<ConsumeResult> {
  return invoke<ConsumeResult>("consume_pending_items", { budgetMinutes });
}

export async function pruneOldItems(): Promise<{ deleted: number; stripped: number }> {
  return invoke<[number, number]>("prune_old_items").then(([deleted, stripped]) => ({ deleted, stripped }));
}

export async function getLastActiveTimestamp(): Promise<number> {
  return invoke<number>("get_last_active_timestamp");
}

export async function setLastActiveTimestamp(timestamp: number): Promise<void> {
  return invoke<void>("set_last_active_timestamp", { timestamp });
}

export async function onAndroidAppBackground(): Promise<void> {
  if (isAndroid()) {
    return invoke<void>("on_android_app_background");
  }
}

export async function onAndroidAppForeground(): Promise<void> {
  if (isAndroid()) {
    return invoke<void>("on_android_app_foreground");
  }
}

export function isAndroid(): boolean {
  return navigator.userAgent.includes('Android');
}

export async function getDiagnosticSummary(): Promise<DiagnosticSummary> {
  return invoke<DiagnosticSummary>("get_diagnostic_summary");
}

export async function getProviderStatus(): Promise<ProviderStatus[]> {
  return invoke<ProviderStatus[]>("get_provider_status");
}

export async function getRecentDiagnostics(limit: number): Promise<DiagnosticLog[]> {
  return invoke<DiagnosticLog[]>("get_recent_diagnostics", { limit });
}

export async function clearDiagnostics(olderThanDays: number): Promise<ClearDiagnosticsResult> {
  return invoke<ClearDiagnosticsResult>("clear_diagnostics", { olderThanDays });
}

export async function triggerCrawl(): Promise<number> {
  return invoke<number>("trigger_crawl");
}

export async function skipNextNotification(): Promise<void> {
  return invoke<void>("skip_next_notification");
}

export async function logDiagnostic(
  eventType: string,
  severity: "info" | "warn" | "error" | "debug",
  message: string,
  metadata?: Record<string, unknown>
): Promise<void> {
  return invoke<void>("log_diagnostic", {
    eventType,
    severity,
    message,
    metadata: metadata ? JSON.stringify(metadata) : null,
  });
}

async function logDiagnosticInternal(
  eventType: string,
  severity: "info" | "warn" | "error" | "debug",
  message: string,
  metadata?: Record<string, unknown>
): Promise<void> {
  if (!isTauri()) return;
  try {
    await logDiagnostic(eventType, severity, message, metadata);
  } catch (e) {
    if (import.meta.env.DEV) {
      console.error("[logDiagnosticInternal] Failed to log:", e);
    }
  }
}
