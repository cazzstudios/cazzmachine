import { useEffect, useState } from "react";
import { useAppStore } from "./stores/appStore";
import { useNotifications } from "./hooks/useNotifications";
import { useAppLifecycle } from "./hooks/useAppLifecycle";
import { useAndroidBackButton } from "./hooks/useAndroidBackButton";
import { Toast } from "./components/Toast";
import { HoverPreview } from "./components/HoverPreview";
import { IdleView } from "./components/IdleView";
import { Summary } from "./components/Summary";
import { DetailView } from "./components/DetailView";
import { ErrorBoundary } from "./components/ErrorBoundary";
import { DiagnosticsPanel } from "./components/DiagnosticsPanel";
import { SplashScreen } from "./components/SplashScreen";
import { getThrottleLevel, triggerCrawl } from "./lib/tauri";

function App() {
  useNotifications();
  useAppLifecycle();
  const view = useAppStore((s) => s.view);
  const setView = useAppStore((s) => s.setView);
  const setIsDoneWorking = useAppStore((s) => s.setIsDoneWorking);

  useAndroidBackButton(view, () => {
    setIsDoneWorking(false);
    setView("idle");
  });
  const debugMode = useAppStore((s) => s.debugMode);
  const isFirstRun = useAppStore((s) => s.isFirstRun);
  const setThrottleLevel = useAppStore((s) => s.setThrottleLevel);
  const [showDiagnostics, setShowDiagnostics] = useState(false);

  useEffect(() => {
    getThrottleLevel().then(async (level) => {
      const currentLevel = useAppStore.getState().throttleLevel;
      if (level !== currentLevel) {
        await setThrottleLevel(level);
      }
      const firstRun = useAppStore.getState().isFirstRun;
      if (!firstRun) {
        await triggerCrawl().catch((e) => {
          console.error("[App] triggerCrawl failed:", e);
        });
      }
    });
  }, [setThrottleLevel]);

  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    if (params.get("diagnostics") === "true" || debugMode) {
      setShowDiagnostics(true);
    }
  }, [debugMode]);

  return (
    <ErrorBoundary>
      {isFirstRun && <SplashScreen />}
      <div className="min-h-screen bg-cazz-bg text-cazz-text">
        {showDiagnostics && (
          <DiagnosticsPanel onClose={() => setShowDiagnostics(false)} />
        )}
        <Toast />
        <HoverPreview />
        <div className="max-w-4xl mx-auto px-4 py-4">
          {view === "idle" && <IdleView onShowDiagnostics={() => setShowDiagnostics(true)} />}
          {view === "summary" && <Summary />}
          {view === "detail" && <DetailView />}
        </div>
      </div>
    </ErrorBoundary>
  );
}

export default App;
