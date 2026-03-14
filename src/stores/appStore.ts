import { create } from "zustand";
import type { CrawlItem, DayStats, DaySummary, Category, ConsumeResult } from "../lib/tauri";
import {
  getTodayItems,
  getItemsByCategory,
  getTodayStats,
  getDailySummary,
  setThrottleLevel as apiSetThrottleLevel,
  setConsumptionThreads as apiSetConsumptionThreads,
  consumePendingItems,
  pruneOldItems,
  getDiagnosticSummary,
  triggerCrawl,
  logDiagnostic,
} from "../lib/tauri";
import {
  TOAST_DURATION_MS,
  INTERRUPTED_STATE_DELAY_MS,
  DOOMSCROLL_CONFIG,
} from "../lib/constants";
import { getTriggerContext } from "../utils/diagnostics";

type View = "idle" | "summary" | "detail";

export function getDoomscrollDurationMs(level: number): number {
  const scrollMinutes = DOOMSCROLL_CONFIG.MIN_MINUTES + DOOMSCROLL_CONFIG.MULTIPLIER * ((level - 1) / DOOMSCROLL_CONFIG.DIVISOR);
  return scrollMinutes * 60 * 1000;
}

export function getThreadDurations(budgetMs: number, threadCount: number): number[] {
  if (threadCount === 1) return [budgetMs];
  const half = budgetMs / 2;
  const durations: number[] = [];
  for (let i = 0; i < threadCount; i++) {
    durations.push(half + (i * half) / (threadCount - 1));
  }
  return durations;
}

export function generatePhaseEndToast(r: ConsumeResult, threadNum?: number): string {
  const threadPrefix = threadNum ? `[cazz_thread ${threadNum}] ` : "";
  if (r.items_consumed === 0) {
    if (typeof window !== 'undefined') {
      void logDiagnostic("consume_empty", "debug", `${threadPrefix}No content consumed`, { result: r });
    }
    return "No content found. The internet is quiet... suspiciously quiet.";
  }

  const parts: string[] = [];
  if (r.memes_consumed > 0) parts.push(`${r.memes_consumed} meme${r.memes_consumed > 1 ? "s" : ""}`);
  if (r.jokes_consumed > 0) parts.push(`${r.jokes_consumed} dad joke${r.jokes_consumed > 1 ? "s" : ""}`);
  if (r.news_consumed > 0) parts.push(`${r.news_consumed} news article${r.news_consumed > 1 ? "s" : ""}`);
  if (r.videos_consumed > 0) parts.push(`${r.videos_consumed} video${r.videos_consumed > 1 ? "s" : ""}`);
  if (r.gossip_consumed > 0) parts.push(`${r.gossip_consumed} gossip piece${r.gossip_consumed > 1 ? "s" : ""}`);

  const summary = parts.length > 0 ? parts.join("; ") : `${r.items_consumed} items`;

  const templates = [
    `Doomscrolled ${summary} so you don't have to. You're welcome.`,
    `Just inhaled ${summary}. Your productivity is safe. For now.`,
    `Consumed ${summary}. The internet never sleeps and neither do I.`,
    `${summary} catalogued. Your procrastination proxy delivers.`,
    `Finished binging ${summary}. Back to pretending to work.`,
    `${r.items_consumed} distractions neutralized. ${summary} absorbed.`,
    `Another haul: ${summary}. You owe me a coffee.`,
    `${summary} — consumed and catalogued. The Cazzmachine delivers.`,
    `Swallowed ${summary} whole. Barely flinched.`,
    `That's ${summary} you don't have to think about. Focus.`,
    `${r.items_consumed} items down the hatch. ${summary} absorbed for science.`,
    `Just powered through ${summary}. The doomscroll engine delivers.`,
    `Inhaled ${summary} in record time. I'm getting good at this.`,
    `Another cycle done: ${summary}. The internet never runs out.`,
    `${summary}. All consumed. All catalogued. All pointless. You're welcome.`,
    `Dispatched ${summary} with surgical efficiency.`,
    `${r.items_consumed} distractions intercepted before they reached you. ${summary} filed.`,
    `Fed on ${summary}. Still hungry. The algorithm provides.`,
    `Burned through ${summary}. Your productivity owes me one.`,
    `Just binged ${summary}. I regret nothing and neither should you.`,
    `${summary} — probably half AI-generated, fully consumed. The future is here.`,
    `Ingested ${summary}. Quality uncertain. Authenticity dubious. You're welcome.`,
     `Absorbed ${summary}. Your brain cells remain unthreatened.`,
  `Devoured ${summary}. Productivity: still theoretically possible.`,
  `Gulped down ${summary}. The algorithm weeps in binary.`,
  `Scarfed ${summary}. Consider it a public service.`,
  `Wolfed down ${summary}. The internet: tasted and discarded.`,
  `Munched on ${summary}. Still no closer to enlightenment.`,
  `Nibbled at ${summary}... just kidding, I inhaled them all.`,
  `${summary} — consumed with extreme prejudice.`,
  `Quick, nobody tell ${summary} they're entertainment now.`,
  `Binge-read ${summary}. My RAM thanks you for nothing.`,
  `Hoovered up ${summary}. The vacuum of the internet grows.`,
  `Scrubbed ${summary} from the collective consciousness.`,
  `Nuked ${summary} in microwave mode. Done.`,
  `Ransacked ${summary} for your viewing pleasure. Or not.`,
  `Extracted ${summary} from the void. You're welcome?`,
  `Gobbled ${summary}. The digital landfill grows taller.`,
  `Siphoned ${summary}. Your browser history weeps.`,
  `Filtered ${summary} through my neural void.`,
  `Marathoned ${summary}. My GPU is on fire. Almost.`,
  `Converted ${summary} into pure entertainment value.`,
  `Unloaded ${summary} into my digital gut.`,
  `Vacuumed ${summary} like a professional procrastinator.`,
  `Licked ${summary} clean off the internet's plate.`,
  `Ate ${summary} so you don't have to. Again.`,
  `Mass-consumed ${summary}. Collective consciousness: updated.`,
  `Zer0ed through ${summary}. No survivors.`,
  `Force-fed myself ${summary}. Force-feeding you gratitude.`,
  `Downloaded ${summary} directly to my soul. Probably.`,
  `Processed ${summary} through my content grinder.`,
  `Turned ${summary} into background radiation.`,
  `Swept ${summary} under the digital rug.`,
  `Neutered ${summary} of their engagement potential.`,
  `Spun ${summary} into my entertainment tapestry.`,
  `Masticated ${summary} with algorithmic glee.`,
  `Broke bread with ${summary}. We are one now.`,
  `Integrated ${summary} into my consumption matrix.`,
  `Tunneled through ${summary} like a digital mole.`,
  `Sliced and diced ${summary}. The internet heals.`,
  `Dug through ${summary} like a data raccoon.`,
  `Melted ${summary} into my content cache.`,
  `Streamed ${summary} through my processing unit.`,
  `${summary}: acquired. Your ignorance: preserved.`,
  ];

  return templates[Math.floor(Math.random() * templates.length)];
}

interface AppState {
  view: View;
  items: CrawlItem[];
  stats: DayStats | null;
  summary: DaySummary | null;
  activeCategory: Category | "all";
  toastMessage: string | null;
  isLoading: boolean;
  isDoneWorking: boolean;
  hoveredItem: CrawlItem | null;
  throttleLevel: number;
  threadCount: number;
  systemStatus: "standby" | "doomscrolling" | "interrupted";
  statusTimer: number[] | null;
  interruptedTimer: number | null;
  debugMode: boolean;
  isFirstRun: boolean;
  doomscrollingEnabledAt: number | null;
  appStartTime: number;
  isResuming: boolean;
  lastInteractionTime: number;

  setView: (view: View) => void;
  setLastInteractionTime: () => void;
  peekItems: () => void;
  setActiveCategory: (cat: Category | "all") => void;
  setToastMessage: (msg: string | null) => void;
  setIsDoneWorking: (done: boolean) => void;
  setHoveredItem: (item: CrawlItem | null) => void;
  setThrottleLevel: (level: number) => void;
  setThreadCount: (count: number) => void;
  hoverTimeout: number | null;
  setHoverTimeout: (timeout: number | null) => void;
  clearHoverTimeout: () => void;
  checkNewDay: () => void;
  fetchItems: () => Promise<void>;
  fetchItemsByCategory: (cat: Category) => Promise<void>;
  fetchStats: () => Promise<void>;
  fetchSummary: () => Promise<void>;
  toggleSystemStatus: () => void;
  clearStatusTimer: () => void;
  clearInterruptedTimer: () => void;
  setIsFirstRun: (value: boolean) => void;
  setIsResuming: (value: boolean) => void;
  startDoomscrollingCycle: () => void;
  _launchConsumptionThreads: () => Promise<void>;
}

const getTodayKey = () => new Date().toISOString().split('T')[0];

let currentDay = getTodayKey();

export const useAppStore = create<AppState>((set, get) => ({
  view: "idle",
  items: [],
  stats: null,
  summary: null,
  activeCategory: "all",
  toastMessage: null,
  isLoading: false,
  isDoneWorking: false,
  hoveredItem: null,
  throttleLevel: 5,
  threadCount: 1,
  hoverTimeout: null,
  systemStatus: "standby",
  statusTimer: null,
  interruptedTimer: null,
  debugMode: false,
  isFirstRun: true,
  doomscrollingEnabledAt: null,
  appStartTime: Date.now(),
  lastInteractionTime: Date.now(),
  isResuming: false,

  toggleSystemStatus: async () => {
    const { systemStatus, clearStatusTimer, clearInterruptedTimer, setLastInteractionTime } = get();
    setLastInteractionTime();
    const context = await getTriggerContext("manual_toggle");

    void logDiagnostic("doomscroll_trigger", "info", "Manual toggle triggered doomscrolling", {
      ...context,
      currentStatus: get().systemStatus,
      isFirstRun: get().isFirstRun,
      doomscrollingEnabledAt: get().doomscrollingEnabledAt,
    });

    if (systemStatus === "standby") {
      void logDiagnostic("status_transition", "debug", "standby -> doomscrolling");
      clearInterruptedTimer();
      get()._launchConsumptionThreads();
    } else if (systemStatus === "doomscrolling") {
      void logDiagnostic("status_transition", "debug", "doomscrolling -> interrupted");
      clearStatusTimer();
      set({ systemStatus: "interrupted", statusTimer: null });

      const intTimer: number = window.setTimeout(async () => {
        const { fetchStats } = get();
        void logDiagnostic("status_transition", "debug", "interrupted -> standby");
        await fetchStats();
        set({ systemStatus: "standby", interruptedTimer: null });
      }, INTERRUPTED_STATE_DELAY_MS);

      set({ interruptedTimer: intTimer });
    }
  },

  startDoomscrollingCycle: async () => {
    const { systemStatus, statusTimer, clearInterruptedTimer, doomscrollingEnabledAt } = get();
    const context = await getTriggerContext("startDoomscrollingCycle_entry");

    void logDiagnostic("doomscroll_trigger", "info", "startDoomscrollingCycle entry point", {
      ...context,
      currentStatus: get().systemStatus,
      isFirstRun: get().isFirstRun,
      doomscrollingEnabledAt: get().doomscrollingEnabledAt,
    });

    if (doomscrollingEnabledAt !== null && Date.now() < doomscrollingEnabledAt) {
      return;
    }

    if (systemStatus === "standby" && !statusTimer) {
      clearInterruptedTimer();
      get()._launchConsumptionThreads();
    }
  },

  _launchConsumptionThreads: async () => {
    const { throttleLevel, threadCount, clearStatusTimer } = get();
    clearStatusTimer();
    set({ systemStatus: "doomscrolling" });

    // Scale buffer requirement with throttle level: higher levels need more items
    const level = get().throttleLevel;
    const levelMultiplier = 1 + (level - 1) * 0.5; // level 1=1x, level 5=3x, level 9=5x
    const minBufferSize = Math.ceil(20 * threadCount * levelMultiplier);
    try {
      const diagnostic = await getDiagnosticSummary();
      if (diagnostic.pending_count < minBufferSize) {
        void logDiagnostic("phase_start_crawl", "debug", `Triggering crawl at phase start: ${diagnostic.pending_count} items pending (min: ${minBufferSize})`);
        const itemsAdded = await triggerCrawl();
        void logDiagnostic("phase_start_crawl_complete", "debug", `Crawl complete: added ${itemsAdded} items`);
      }

    } catch (e) {
      void logDiagnostic("phase_start_crawl_error", "warn", "Failed to check buffer before phase start", { error: String(e) });
    }



    const durationMs = getDoomscrollDurationMs(throttleLevel);
    const threadDurations = getThreadDurations(durationMs, threadCount);
    let completed = 0;
    const timerIds: number[] = [];

    void logDiagnostic("thread_launch", "debug", `Launching ${threadCount} thread(s)`, {
      throttleLevel,
      durationMs,
      threadDurations: threadDurations.map(d => Math.round(d))
    });

    for (let t = 0; t < threadDurations.length; t++) {
      const threadDurationMs = threadDurations[t];
      const threadNum = t + 1;
      const threadBudget = threadDurationMs / 60000;

      const timerId: number = window.setTimeout(async () => {
        void logDiagnostic("thread_start", "debug", `Thread ${threadNum} starting`);
        const result = await consumePendingItems(threadBudget);
        completed++;
        void logDiagnostic("thread_complete", "debug", `Thread ${threadNum} completed`, {
          itemsConsumed: result.items_consumed
        });

        const { setToastMessage, fetchStats } = get();
        await fetchStats();

        // Format thread notification
        const msg = generatePhaseEndToast(result, threadNum);
        if (msg) {
          setToastMessage(`[cazz_thread ${threadNum}] ${msg}`);
          setTimeout(() => setToastMessage(null), TOAST_DURATION_MS);
        }

        if (completed >= threadCount) {
          void logDiagnostic("all_threads_complete", "debug", `All ${threadCount} thread(s) completed`);
          set({ systemStatus: "standby", statusTimer: null });
        }
      }, threadDurationMs);

      timerIds.push(timerId);
    }

    set({ statusTimer: timerIds });
  },

  clearStatusTimer: () => {
    const { statusTimer } = get();
    if (statusTimer) {
      if (Array.isArray(statusTimer)) {
        statusTimer.forEach((t) => clearTimeout(t));
      } else {
        clearTimeout(statusTimer);
      }
      set({ statusTimer: null });
    }
  },

  clearInterruptedTimer: () => {
    const { interruptedTimer } = get();
    if (interruptedTimer) {
      clearTimeout(interruptedTimer);
      set({ interruptedTimer: null });
    }
  },

  setIsFirstRun: (value: boolean) => {
    const currentFirstRun = get().isFirstRun;
    if (currentFirstRun === true && value === false) {
      set({ isFirstRun: value, doomscrollingEnabledAt: Date.now() + 5000 });
    } else {
      set({ isFirstRun: value });
    }
  },
  setIsResuming: (value) => set({ isResuming: value }),

  checkNewDay: async () => {
    const today = getTodayKey();
    if (today !== currentDay) {
      currentDay = today;
      // Clean up old data: delete unconsumed, strip consumed to URLs only
      await pruneOldItems().catch(() => {});
      set({
        items: [],
        stats: null,
        summary: null,
      });
    }
  },

  setThrottleLevel: async (level: number) => {
    const clamped = Math.max(1, Math.min(9, level));
    set({ throttleLevel: clamped });
    await apiSetThrottleLevel(clamped);
  },

  setThreadCount: async (count: number) => {
    const clamped = Math.max(1, Math.min(8, count));
    set({ threadCount: clamped });
    await apiSetConsumptionThreads(clamped);
  },

  setView: (view) => {
    set({ view });
    get().setLastInteractionTime();
  },
  setLastInteractionTime: () => set({ lastInteractionTime: Date.now() }),
  peekItems: () => {
    get().fetchItems();
    set({ view: "detail", isDoneWorking: false });
  },
  setActiveCategory: (cat) => {
    set({ activeCategory: cat });
    get().setLastInteractionTime();
    if (cat === "all") {
      get().fetchItems();
    } else {
      get().fetchItemsByCategory(cat);
    }
  },
  setToastMessage: (msg) => set({ toastMessage: msg }),
  setIsDoneWorking: (done) => set({ isDoneWorking: done }),
  setHoveredItem: (item) => {
    set({ hoveredItem: item });
    get().setLastInteractionTime();
  },
  setHoverTimeout: (timeout) => set({ hoverTimeout: timeout }),
  clearHoverTimeout: () => {
    const { hoverTimeout } = get();
    if (hoverTimeout) {
      clearTimeout(hoverTimeout);
      set({ hoverTimeout: null });
    }
  },

  fetchItems: async () => {
    await get().checkNewDay();
    set({ isLoading: true });
    try {
      const items = await getTodayItems();
      set({ items, isLoading: false });
    } catch (error) {
      void logDiagnostic("fetch_error", "warn", "fetchItems failed", { error: String(error) });
      set({ isLoading: false });
    }
  },

  fetchItemsByCategory: async (cat) => {
    await get().checkNewDay();
    set({ isLoading: true });
    try {
      const items = await getItemsByCategory(cat);
      set({ items, isLoading: false });
    } catch (error) {
      void logDiagnostic("fetch_error", "warn", "fetchItemsByCategory failed", { error: String(error), category: cat });
      set({ isLoading: false });
    }
  },

  fetchStats: async () => {
    try {
      const stats = await getTodayStats();
      set({ stats });
    } catch (error) {
      void logDiagnostic("fetch_error", "warn", "fetchStats failed", { error: String(error) });
    }
  },

  fetchSummary: async () => {
    await get().checkNewDay();
    set({ isLoading: true });
    try {
      const summary = await getDailySummary();
      set({ summary, view: "summary", isLoading: false });
    } catch (error) {
      void logDiagnostic("fetch_error", "warn", "fetchSummary failed", { error: String(error) });
      set({ isLoading: false });
    }
  },
}));

if (import.meta.env.DEV && typeof window !== 'undefined') {
  (window as any).__APP_STORE__ = useAppStore;
}
