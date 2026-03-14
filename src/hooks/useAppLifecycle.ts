import { useEffect, useCallback } from 'react';
import { useAppStore } from '../stores/appStore';
import { getLastActiveTimestamp, setLastActiveTimestamp, consumePendingItems, triggerCrawl, type ConsumeResult, logDiagnostic, isAndroid, skipNextNotification, onAndroidAppBackground, onAndroidAppForeground } from '../lib/tauri';
import { onResume, onPause } from 'tauri-plugin-app-events-api';

// localStorage persists across WebView restarts (survives app background/foreground cycles)
// This avoids the issue where sessionStorage is cleared when WebView process is killed
// Fallback: AndroidPrefs JavaScript interface reads from SharedPreferences
const getBackgroundedAt = (): number | null => {
  // First try localStorage (written by Kotlin's onPause)
  const stored = localStorage.getItem('backgroundedAtMs');
  if (stored) {
    return parseInt(stored, 10);
  }
  
  // Fallback: try AndroidPrefs (reads from SharedPreferences)
  // This is available on Android and persists even if localStorage is cleared
  if (typeof window !== 'undefined' && (window as any).AndroidPrefs) {
    const ts = (window as any).AndroidPrefs.getBackgroundTimestamp();
    if (ts > 0) {
      return ts;
    }
  }
  
  return null;
};

const setBackgroundedAt = (ts: number): void => {
  localStorage.setItem('backgroundedAtMs', ts.toString());
};

const clearBackgroundedAt = (): void => {
  localStorage.removeItem('backgroundedAtMs');
};

// Calculate active percentage based on knob level (1-9)
// Formula: activePercent = 0.02 + (level - 1) * 0.89 / 8
// Level 1 = 2%, Level 5 = 47%, Level 9 = 91%
const getActivePercent = (level: number): number => {
  return 0.02 + (level - 1) * 0.89 / 8;
};


export function useAppLifecycle() {
  const {
    threadCount,
    throttleLevel,
    clearStatusTimer,
    setToastMessage,
    setIsResuming,
  } = useAppStore();

  const handleAppResume = useCallback(async () => {
    useAppStore.setState({ isResuming: true });
    try {
      const now = Date.now();
      // Prefer the in-memory value to avoid DB write race condition on quick resume
      const lastActive = getBackgroundedAt() ?? await getLastActiveTimestamp();
      const elapsedMinutes = (now - lastActive) / 60000;

      const state = useAppStore.getState();
      void logDiagnostic("doomscroll_trigger", "info", "App resume consumption", {
        source: "app_resume",
        currentStatus: state.systemStatus,
        isFirstRun: state.isFirstRun,
        elapsedSinceLastActive: elapsedMinutes,
        usedInMemoryTs: getBackgroundedAt() !== null,
        throttleLevel: state.throttleLevel,
      });

      await setLastActiveTimestamp(now);
      clearBackgroundedAt();

      if (isAndroid()) {
        await skipNextNotification();
        await onAndroidAppForeground();
      }

      if (elapsedMinutes < 1) {
        void logDiagnostic("app_resume", "info", "Resume skipped: elapsed < 1 min", { elapsedMinutes });
        return;
      }

      // Account for active percentage based on knob level
      // At level 1: 2% active, at level 5: ~47% active, at level 9: 91% active
      const activePercent = getActivePercent(throttleLevel);
      const budgetMinutes = elapsedMinutes * threadCount * activePercent;

      // Fire crawl in background — don't await, it blocks for up to 15s and we have items already
      void triggerCrawl().catch((e) => {
        void logDiagnostic("app_resume_crawl_error", "warn", "Background crawl on resume failed", { error: String(e) });
      });

      const result = await consumePendingItems(budgetMinutes);

      if (result.items_consumed > 0) {
        const msg = formatResumeMessage(result);
        setToastMessage(msg);
        setTimeout(() => setToastMessage(null), 8000);
      }

      void logDiagnostic("app_resume", "info", `App resume: consumed ${result.items_consumed} items`, { elapsedMinutes: elapsedMinutes.toFixed(1) });
    } catch (error) {
      void logDiagnostic("app_resume_error", "error", "Failed to handle app resume", { error: String(error) });
    } finally {
      setTimeout(() => useAppStore.setState({ isResuming: false }), 3000);
    }
  }, [threadCount, throttleLevel, setToastMessage, setIsResuming]);

  const handleAppBackground = useCallback(async () => {
    // Set synchronously — DB write below is async and may not finish before onResume fires
    setBackgroundedAt(Date.now());

    try {
      const state = useAppStore.getState();
      void logDiagnostic("app_background", "info", "App background: clearing status and saving timestamp", {
        source: "app_background",
        currentStatus: state.systemStatus,
      });

      await setLastActiveTimestamp(getBackgroundedAt() ?? Date.now());

      clearStatusTimer();

      const currentState = useAppStore.getState();
      if (currentState.systemStatus !== "standby") {
        useAppStore.setState({ systemStatus: "standby" });
        void logDiagnostic("app_background", "info", "System status reset to standby on background", {});
      }

      if (isAndroid()) {
        await skipNextNotification();
        await onAndroidAppBackground();
      }

      void logDiagnostic("app_background", "info", `App background: saved timestamp ${getBackgroundedAt()}, timer cleared`, {});
    } catch (error) {
      void logDiagnostic("app_background_error", "error", "Failed to handle app background", { error: String(error) });
    }
  }, [clearStatusTimer]);

  useEffect(() => {
    if (!isAndroid()) {
      const handleVisibilityChange = async () => {
        if (document.visibilityState === 'visible') {
          await handleAppResume();
        } else {
          await handleAppBackground();
        }
      };
      document.addEventListener('visibilitychange', handleVisibilityChange);
      return () => {
        document.removeEventListener('visibilitychange', handleVisibilityChange);
      };
    }

    const setup = async () => {
      try {
        await getLastActiveTimestamp();
      } catch {
        await setLastActiveTimestamp(Date.now());
      }
    };

    setup();

    onResume(handleAppResume);
    onPause(handleAppBackground);

    return () => {
      onResume(() => {});
      onPause(() => {});
    };
  }, [handleAppResume, handleAppBackground]);
}

function formatResumeMessage(result: ConsumeResult): string {
  const parts: string[] = [];
  if (result.memes_consumed > 0) parts.push(`${result.memes_consumed} meme${result.memes_consumed > 1 ? 's' : ''}`);
  if (result.jokes_consumed > 0) parts.push(`${result.jokes_consumed} dad joke${result.jokes_consumed > 1 ? 's' : ''}`);
  if (result.news_consumed > 0) parts.push(`${result.news_consumed} news article${result.news_consumed > 1 ? 's' : ''}`);
  if (result.videos_consumed > 0) parts.push(`${result.videos_consumed} video${result.videos_consumed > 1 ? 's' : ''}`);
  if (result.gossip_consumed > 0) parts.push(`${result.gossip_consumed} gossip piece${result.gossip_consumed > 1 ? 's' : ''}`);

  const summary = parts.length > 0 ? parts.join('; ') : `${result.items_consumed} items`;

  const templates = [
    `While you were away, I doomscrolled through ${summary}. You're welcome.`,
    `Welcome back. In your absence I consumed ${summary}. Someone had to.`,
    `While you were out living your life, I devoured ${summary}.`,
    `Back already? I barely had time to finish — but I got through ${summary}.`,
    `You missed nothing. Or rather, I made sure of it: ${summary}, handled.`,
    `Absence noted. Doomscrolling continued. ${summary} consumed on your behalf.`,
    `Oh, you're back. I was just wrapping up ${summary}.`,
    `While you were away the internet kept producing garbage. I processed ${summary} of it.`,
    `You were gone. The internet kept generating slop. I duly processed it: ${summary}.`,
  ];
  return templates[Math.floor(Math.random() * templates.length)];
}
