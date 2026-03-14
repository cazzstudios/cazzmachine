import { useAppStore } from "../stores/appStore";
import { getDiagnosticSummary, getRecentDiagnostics } from "../lib/tauri";
import type { DiagnosticLog } from "../lib/tauri";

export interface TriggerContext {
  source: string;
  appUptime: number;
  timeSinceLastInteraction: number;
  itemsAvailable: number;
  throttleLevel: number;
  threadCount: number;
}

export async function getTriggerContext(source: string): Promise<TriggerContext> {
  const state = useAppStore.getState();
  const now = Date.now();
  
  let itemsAvailable = 0;
  try {
    const diagnostic = await getDiagnosticSummary();
    itemsAvailable = diagnostic.pending_count;
  } catch {
    itemsAvailable = state.items.length;
  }

  return {
    source,
    appUptime: now - state.appStartTime,
    timeSinceLastInteraction: now - state.lastInteractionTime,
    itemsAvailable,
    throttleLevel: state.throttleLevel,
    threadCount: state.threadCount,
  };
}

/**
 * Exports doomscroll_trigger diagnostic logs to JSON.
 * @param since Optional timestamp in ms to filter from. Defaults to last 24 hours.
 * @returns JSON string of trigger logs
 */
export async function exportTriggerLogs(since?: number): Promise<string> {
  const sinceTime = since ?? (Date.now() - 24 * 60 * 60 * 1000);
  
  // Get a reasonable number of recent logs (more than needed, we'll filter)
  const allLogs = await getRecentDiagnostics(1000);
  
  // Filter by event type and timestamp
  const triggerLogs = allLogs.filter(
    (log: DiagnosticLog) => 
      log.event_type === "doomscroll_trigger" && 
      new Date(log.timestamp).getTime() >= sinceTime
  );
  
  return JSON.stringify(triggerLogs, null, 2);
}
