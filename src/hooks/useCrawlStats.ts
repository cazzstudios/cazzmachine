import { useEffect } from "react";
import { useAppStore } from "../stores/appStore";

export function useCrawlStats() {
  const fetchStats = useAppStore((s) => s.fetchStats);
  const checkNewDay = useAppStore((s) => s.checkNewDay);
  const stats = useAppStore((s) => s.stats);

  useEffect(() => {
    checkNewDay();
    fetchStats();
    const interval = setInterval(() => {
      checkNewDay();
      fetchStats();
    }, 30000);
    return () => clearInterval(interval);
  }, [fetchStats, checkNewDay]);

  return stats;
}
