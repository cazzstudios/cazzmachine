import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { isPermissionGranted, requestPermission } from "@tauri-apps/plugin-notification";
import { useAppStore } from "../stores/appStore";
import { logDiagnostic, isAndroid } from "../lib/tauri";

export function useNotifications() {
  const startDoomscrollingCycle = useAppStore((s) => s.startDoomscrollingCycle);
  const clearStatusTimer = useAppStore((s) => s.clearStatusTimer);
  const systemStatus = useAppStore((s) => s.systemStatus);

  useEffect(() => {
    async function setupNotifications() {
      const hasPermission = await isPermissionGranted();
      if (!hasPermission) {
        await requestPermission();
      }
    }
    setupNotifications().catch((e) => logDiagnostic("notification_setup_error", "warn", "Failed to setup notifications", { error: String(e) }));

    const unlisten = listen<string>("cazz-notification", () => {
      const state = useAppStore.getState();
      
      if (isAndroid()) {
        if (document.hidden || document.visibilityState === 'hidden') {
          void logDiagnostic("doomscroll_trigger", "debug", "Ignoring notification: app not visible", {
            source: "notification_event",
            visibilityState: document.visibilityState,
          });
          return;
        }
        if (state.isResuming) {
          void logDiagnostic("doomscroll_trigger", "debug", "Ignoring notification: app is resuming", {
            source: "notification_event",
          });
          return;
        }
      }
      
      void logDiagnostic("doomscroll_trigger", "info", "Notification event triggered doomscrolling", {
        source: "notification_event",
        currentStatus: state.systemStatus,
        isFirstRun: state.isFirstRun,
        doomscrollingEnabledAt: state.doomscrollingEnabledAt,
      });
      startDoomscrollingCycle();
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [startDoomscrollingCycle, clearStatusTimer, systemStatus]);
}
